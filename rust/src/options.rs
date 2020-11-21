use crate::audit::Audit;
use crate::folder::Folder;
use crate::forwarded_identity::ForwardedIdentity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Default, Deserialize)]
pub struct Group {
    pub name: String,
    pub members_email: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OptionsInternal {
    pub log_level: Option<String>,
    pub access_control_allow_origin: Option<String>,
    pub thumb_folder_path: String,
    pub log_file: String,
    pub audit_file: Option<String>,
    pub static_site_path: String,
    pub groups: Vec<Group>,
    pub folders: Vec<Folder>,
    pub prometheus_metrics_port: Option<u16>,
}

#[derive(Clone, Debug)]
pub struct Options {
    pub log_level: log::LevelFilter,
    pub access_control_allow_origin: Option<String>,
    pub thumb_folder_path: String,
    pub log_file: String,
    pub audit_file: Option<String>,
    pub static_site_path: String,
    pub audit: Option<Audit>,
    pub groups: Vec<Group>,
    pub folders: Vec<Folder>,
    pub prometheus_metrics_port: Option<u16>,
    all_emails: HashSet<String>,
}

impl TryFrom<&str> for Options {
    type Error = toml::de::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut options: OptionsInternal = toml::from_str(s)?;
        // securities are sorted by folder, so they are easier to travel
        options.folders.sort();

        // calculate unique users
        let mut all_emails = HashSet::new();
        options.groups.iter().for_each(|group| {
            group.members_email.iter().for_each(|email| {
                all_emails.insert(email.to_owned());
            })
        });

        Ok(Options {
            log_level: match options.log_level {
                None => log::LevelFilter::Info,
                Some(log_level) => match log_level.as_ref() {
                    "Error" => log::LevelFilter::Error,
                    "Warn" => log::LevelFilter::Warn,
                    "Info" => log::LevelFilter::Info,
                    "Debug" => log::LevelFilter::Debug,
                    "Trace" => log::LevelFilter::Trace,
                    "Off" => log::LevelFilter::Off,
                    _ => log::LevelFilter::Info,
                },
            },
            access_control_allow_origin: options.access_control_allow_origin,
            thumb_folder_path: options.thumb_folder_path,
            log_file: options.log_file,
            audit: options
                .audit_file
                .as_ref()
                .map(|audit_file| Audit::new(audit_file)),
            audit_file: options.audit_file,
            static_site_path: options.static_site_path,
            groups: options.groups,
            folders: options.folders,
            prometheus_metrics_port: options.prometheus_metrics_port,
            all_emails,
        })
    }
}

impl OptionsInternal {
    pub fn serialize(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self)
    }
}

impl Options {
    pub fn audit(
        &self,
        email: &str,
        obj_type: &str,
        obj_name: &str,
        operation: &str,
        allowed: bool,
    ) {
        if let Some(audit) = &self.audit {
            audit.send_event(format!(
                "{}|{}|{}|{}|{}|{}",
                chrono::Local::now().format("%Y-%m-%d|%H:%M:%S"),
                email,
                obj_type,
                obj_name,
                operation,
                match allowed {
                    true => "ALLOWED",
                    false => "DENIED",
                }
            ));
        }
    }

    pub fn identity_allowed(&self, forwared_identity: &ForwardedIdentity) -> bool {
        forwared_identity.forced() || self.all_emails.contains(&forwared_identity.email)
    }

    pub fn calculate_ancestors(&self) -> Vec<(&Folder, &Folder)> {
        let mut ancestors = Vec::new();
        // for each folder, find the topmost one
        for folder_to_find_root in &self.folders {
            let mut tree: Vec<&Folder> = self
                .folders
                .iter()
                .filter(|folder| folder_to_find_root.path.starts_with(&folder.path))
                .collect::<_>();
            tree.sort_by(|a, b| a.path.len().cmp(&b.path.len()));

            let ancestor = match tree.first() {
                Some(s) => s,
                None => folder_to_find_root,
            };

            ancestors.push((folder_to_find_root, ancestor));
        }

        ancestors
    }

    /// calulates is the path from `from` to `to`
    /// is always browsable
    pub fn simplify_path(&self, user: &str, from: &Folder, to: &Folder) -> Vec<&Folder> {
        debug!("from == {}, to == {}", from.path, to.path);

        let path_folders: Vec<&Folder> = self
            .folders
            .iter()
            .filter(|folder| {
                folder.path.as_str().starts_with(from.path.as_str())
                    && to.path.as_str().starts_with(folder.path.as_str())
            })
            .collect::<_>();

        debug!("path_folders == {:#?}", path_folders);

        // check if each folder inherits and if it has access
        let mut inherited = false;
        let mut res = Vec::new();
        for folder in path_folders {
            debug!("checking {}", folder.path);
            if self.is_folder_allowed(&PathBuf::from(&folder.path), user) {
                debug!("allowed!");
                // if it's inherited do not store it
                // if it's not let's store it
                if !inherited {
                    res.push(folder);
                }
                inherited = folder.inheritable.unwrap_or(false);
            } else {
                inherited = false;
            }
        }

        //debug!("res == {:#?}", res);

        res
    }

    pub fn first_level_allowed_folders(&self, user: &str) -> HashSet<&str> {
        let ancestors = self.calculate_ancestors();
        let mut hs = std::collections::HashSet::new();
        for anc in ancestors {
            debug!("{} --> {}", anc.0.path, anc.1.path);
            let simplified_path = self.simplify_path(user, anc.1, anc.0);
            for sim in simplified_path {
                hs.insert(&sim.path as &str);
                debug!("\t{:?}", sim);
            }
        }
        debug!("{:#?}", hs);

        hs
    }

    pub fn calculate_first_level_folders_for_every_user(&self) -> HashMap<String, Vec<String>> {
        // now call  first_level_allowed_folders for every user and store it
        let mut hm = HashMap::with_capacity(self.all_emails.len());

        self.all_emails.iter().for_each(|user| {
            hm.insert(
                user.to_owned(),
                self.first_level_allowed_folders(&user)
                    .into_iter()
                    .map(|user| user.to_owned())
                    .collect::<_>(),
            );
        });

        hm
    }

    pub fn is_folder_allowed(&self, path_to_check: &PathBuf, user_to_check: &str) -> bool {
        // we need to traverse the path from root to here and collect the
        // resultant permissions
        debug!(
            "path_to_check == {:?}, is_dir() == {}",
            &path_to_check,
            path_to_check.is_dir()
        );

        let mut current_allowed: HashSet<String> = HashSet::new();
        let mut current_denied: HashSet<String> = HashSet::new();
        let mut current_path = "/";
        let mut current_inheritable = false;

        // first find all the paths (from "root" path)
        // in securities composing the given path
        // that is, every folder that is superfolder
        // of directory_to_check
        let subpaths: Vec<&Folder> = self
            .folders
            .iter()
            .filter(|path| {
                debug!(
                    "path == {}, path_to_check == {:?}",
                    &path.path, path_to_check
                );
                let to_add = path_to_check.to_str().unwrap().starts_with(&path.path);
                debug!("to_add == {}", to_add);
                to_add
            })
            .collect::<_>();

        debug!("subpaths == {:?}", subpaths);

        // since subpaths are sorted by construction we can enumerate one by one and
        // calculate the resultant set of permissions
        for subpath in subpaths {
            debug!("processing path {:?}", subpath);

            // if the path breaks inheritance reset the permissions!
            #[allow(unused_assignments)]
            if let Some(breaks_inheritance) = subpath.breaks_inheritance {
                if breaks_inheritance {
                    current_allowed = HashSet::new();
                    current_denied = HashSet::new();
                    current_inheritable = false;
                }
            }

            // if it's inheritable save the info
            current_inheritable = if let Some(inheritable) = subpath.inheritable {
                inheritable
            } else {
                false
            };

            // let's add the relevant items
            // TODO: Remove the unnecessary string
            // allocations here
            if let Some(allowed) = &subpath.allowed {
                allowed.iter().for_each(|allowed| {
                    current_allowed.insert(allowed.to_owned());
                });
            }
            if let Some(denied) = &subpath.denied {
                denied.iter().for_each(|denied| {
                    current_denied.insert(denied.to_owned());
                });
            }

            current_path = &subpath.path;

            debug!("current_path == {:#?}", current_path);
            debug!("current_allowed == {:#?}", current_allowed);
            debug!("current_denied == {:#?}", current_denied);
            debug!("current_inheritable == {:?}", current_inheritable);
        }

        // now we have the resultant policy, let's check it!
        // first let's explode the groups
        let current_allowed = self.explode_group(current_allowed);
        let current_denied = self.explode_group(current_denied);
        debug!(
            "after group explosion current_allowed == {:#?}",
            current_allowed
        );
        debug!(
            "after group explosion current_denied == {:#?}",
            current_denied
        );

        // If the directory to check is not the same as the
        // last checked path and inheritance is disabled
        // we return false
        let is_allowed = if path_to_check.to_str().unwrap() != current_path && !current_inheritable
        {
            false
        } else {
            // authorize the user if it's not in the denied list
            // and is in the authorized one
            current_denied
                .iter()
                .find(|user| user == &user_to_check)
                .is_none()
                && current_allowed.iter().any(|user| user == user_to_check)
        };

        self.audit(
            user_to_check,
            match path_to_check.is_dir() {
                true => "directory",
                false => "file",
            },
            path_to_check.to_str().unwrap(),
            "check",
            is_allowed,
        );

        is_allowed
    }

    fn explode_group(&self, hs: HashSet<String>) -> HashSet<String> {
        let mut tmp = HashSet::new();
        hs.iter().for_each(|item| {
            if item.starts_with('#') {
                // find the corresponding group
                if let Some(group) = self.groups.iter().find(|group| group.name == item[1..]) {
                    // if found, let's add it!
                    group.members_email.iter().for_each(|email| {
                        tmp.insert(email.to_owned());
                    });
                } else {
                    warn!(
                        "the group {} was not found, possible error in the securities.",
                        item
                    );
                }
            } else {
                // simple email, let's add it as it is
                tmp.insert(item.to_owned());
            }
        });

        tmp
    }
}
