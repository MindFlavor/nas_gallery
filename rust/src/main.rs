#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;
use rocket::http::Status;
use rocket::http::{ContentType, MediaType};
use rocket::response::NamedFile;
use rocket::{Response, State};
use snafu::{Backtrace, ResultExt, Snafu};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, RwLock};

mod audit;
mod file_type;
mod file_with_size;
mod folder;
mod forwarded_identity;
mod logging;
mod options;
mod statistics;
use file_type::FileType;
use file_with_size::FileWithSize;
use forwarded_identity::ForwardedIdentity;
use logging::setup_logger;
use options::*;
use statistics::*;

static IMAGE_EXTENSIONS: &[&str] = &["png", "bmp", "jpg", "gif"];
static VIDEO_EXTENSIONS: &[&str] = &["mkv", "mp4", "avi", "mov", "webm"];

#[get("/metrics")]
pub(crate) fn metrics<'r>(statistics: State<'_, Arc<RwLock<Statistics>>>) -> Response<'r> {
    let mut response = Response::new();
    response.set_status(Status::Ok);
    response.set_sized_body(Cursor::new(
        statistics.read().unwrap().render_to_prometheus(),
    ));
    response
}

fn get_file<'r>(path: &Path) -> Result<Response<'r>, Box<dyn std::error::Error>> {
    let file = std::fs::OpenOptions::new().read(true).open(&path)?;

    let content_type = ContentType::parse_flexible(&path.extension().unwrap().to_str().unwrap())
        .unwrap_or_else(|| {
            let extension = path.extension().unwrap().to_str().unwrap().to_lowercase();
            debug!("extension == {:?}", extension);

            let media_type =
                MediaType::from_extension(&extension).unwrap_or_else(|| match extension.as_ref() {
                    "mkv" => MediaType::new("video", "mp4"),
                    "mp4" => MediaType::new("video", "mp4"),
                    "avi" => MediaType::new("video", "x-msvideo"),
                    "webm" => MediaType::new("video", "webm"),
                    "webp" => MediaType::new("image", "webp"),
                    "ogv" => MediaType::new("video", "ogg"),
                    "mpeg" => MediaType::new("video", "mpeg"),
                    _ => {
                        warn!(
                            "unsuppored media type {}, returning application/octet-stream",
                            extension
                        );
                        MediaType::new("application", "octet-stream")
                    }
                });
            ContentType(media_type)
        });
    debug!("content_type == {:?}", content_type);

    let mut response = Response::new();
    response.set_status(Status::Ok);
    response.set_header(content_type);
    response.set_sized_body(file);
    Ok(response)
}

#[get("/", rank = 1)]
fn root<'a>(
    options: State<'_, Options>,
    statistics: State<'_, Arc<RwLock<Statistics>>>,
    forwarded_identity: ForwardedIdentity,
) -> Response<'a> {
    if !options.identity_allowed(&forwarded_identity) {
        track_unauthorized(&options, &statistics, "/");
        let mut response = Response::new();
        response.set_status(Status::Unauthorized);
        response
    } else {
        track_authorized_static(&options, &statistics, "/");
        let path = Path::new(&options.static_site_path).join("index.html");
        get_file(&path).unwrap()
    }
}

#[get("/<file..>", rank = 1)]
fn site<'r>(
    options: State<'_, Options>,
    statistics: State<'_, Arc<RwLock<Statistics>>>,
    forwarded_identity: ForwardedIdentity,
    file: PathBuf,
) -> Response<'r> {
    if !options.identity_allowed(&forwarded_identity) {
        track_unauthorized(&options, &statistics, file.to_str().unwrap());
        let mut response = Response::new();
        response.set_status(Status::Unauthorized);
        response
    } else {
        let complete_path = Path::new(&options.static_site_path).join(&file);
        trace!("requested: {:?}, mapped as {:?}", &file, &complete_path);
        if complete_path.exists() {
            track_authorized_static(&options, &statistics, complete_path.to_str().unwrap());
            get_file(&complete_path).unwrap()
        } else {
            // the file does not exists so let's call index.html and let
            // Angular sort out the path
            let path = Path::new(&options.static_site_path).join("index.html");
            debug!(
                "requested {:?} but not found in the site path as {:?}, defaulting to {:?}",
                &file, &complete_path, &path
            );
            if !path.exists() {
                error!("default path {:?} does not exists or is not accessible. Check the application configuration. Requested file = {:?}, mapped as non existing file = {:?}", path, &file, &complete_path);
                let mut response = Response::new();
                response.set_status(Status::NotFound);
                response
            } else {
                track_authorized_dynamic(&options, &statistics);
                get_file(&path).unwrap()
            }
        }
    }
}

#[get("/path/<path..>")]
fn path<'r>(
    options: State<'_, Options>,
    statistics: State<'_, Arc<RwLock<Statistics>>>,
    forwarded_identity: ForwardedIdentity,
    path: PathBuf,
) -> Response<'r> {
    let path = PathBuf::from("/").join(path);
    trace!("requesting: {:?}", &path);
    trace!("Authenticated as {}", &forwarded_identity);
    let is_folder_allowed = options.is_folder_allowed(&path, &forwarded_identity.email);
    trace!("is_folder_allowed == {}", is_folder_allowed);

    if !is_folder_allowed {
        track_unauthorized(&options, &statistics, path.to_str().unwrap());
        let mut response = Response::new();
        response.set_status(Status::Unauthorized);
        response
    } else {
        trace!("extension == {:?}", path.as_path().extension());
        let extension = match path.as_path().extension() {
            Some(ext) => ext.to_str().unwrap().to_lowercase(),
            None => {
                let mut response = Response::new();
                response.set_status(Status::NotFound);
                return response;
            }
        };

        if path.as_path().is_dir() {
            track_authorized_not_found(&options, &statistics);
            let mut response = Response::new();
            response.set_status(Status::NotFound);
            response
        } else if IMAGE_EXTENSIONS.iter().any(|&ext| ext == extension)
            || VIDEO_EXTENSIONS.iter().any(|&ext| ext == extension)
        {
            track_authorized_dynamic(&options, &statistics);
            options.audit(
                &forwarded_identity.email,
                "image/video",
                path.to_str().unwrap(),
                "get",
                true,
            );

            debug!("sending == {:?}", &path);
            match get_file(&path) {
                Ok(response) => response,
                Err(_err) => {
                    let mut response = Response::new();
                    response.set_status(Status::NotFound);
                    response
                }
            }
        } else {
            track_authorized_not_found(&options, &statistics);
            let mut response = Response::new();
            response.set_status(Status::NotFound);
            response
        }
    }
}

fn generate_thumb_folder_path(options: &Options, size: u64, original_path: &PathBuf) -> PathBuf {
    trace!("original_path == {:?}", &original_path);
    let path = Path::new(&options.thumb_folder_path).join(format!("{}x{}", size, size));
    trace!("generate_thumb_folder_path == {:?}", &path);
    let path = path.join(&original_path.parent().unwrap().to_str().unwrap()[1..]);
    trace!("generate_thumb_folder_path == {:?}", &path);

    std::fs::create_dir_all(&path).unwrap();

    path
}

fn generate_picture_thumb(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
    size: u64,
    original_path: &PathBuf,
    complete_path: &PathBuf,
) -> PathBuf {
    let output_file_name = generate_thumb_folder_path(options, size, &original_path).join(format!(
        "{}.jpg",
        original_path.file_name().unwrap().to_str().unwrap()
    ));
    trace!("output_file_name == {:#?}", output_file_name);
    track_picture_thumb_access(options, statistics);

    // if we already have a thumb, do not regenerate it
    if !output_file_name.exists() {
        track_picture_thumb_generation(options, statistics);

        let mut cmd = Command::new("convert");
        let cmd = cmd.args(&[
            complete_path.to_str().unwrap(),
            "-auto-orient",
            "-thumbnail",
            &format!("{}x{}>", size, size),
            "-background",
            "white",
            "-gravity",
            "center",
            "-extent",
            &format!("{}x{}", size, size),
            output_file_name.to_str().unwrap(),
        ]);
        trace!("{:#?}", cmd);
        let output = cmd.output().unwrap();
        trace!("{:?}", output);
    }

    output_file_name
}

fn generate_video_thumb(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
    size: u64,
    original_path: &PathBuf,
    complete_path: &PathBuf,
) -> PathBuf {
    let output_file_name = generate_thumb_folder_path(options, size, &original_path).join(format!(
        "{}.jpg",
        original_path.file_name().unwrap().to_str().unwrap()
    ));
    trace!("output_file_name == {:#?}", output_file_name);
    track_video_thumb_access(options, statistics);

    // if we already have a thumb, do not regenerate it
    if !output_file_name.exists() {
        track_video_thumb_generation(options, statistics);

        let mut cmd = Command::new("ffmpeg");
        let cmd = cmd.args(&[
            "-i",
            complete_path.to_str().unwrap(),
            "-vframes",
            "1",
            output_file_name.to_str().unwrap(),
            "-y",
        ]);
        trace!("about to send == {:#?}", cmd);
        let output = cmd.output().unwrap();
        trace!("{:?}", output);

        let mut cmd = Command::new("convert");
        let cmd = cmd.args(&[
            output_file_name.to_str().unwrap(),
            "-thumbnail",
            &format!("{}x{}>", size, size),
            "-background",
            "white",
            "-gravity",
            "center",
            "-extent",
            &format!("{}x{}", size, size),
            output_file_name.to_str().unwrap(),
        ]);
        trace!("{:#?}", cmd);
        let output = cmd.output().unwrap();
        trace!("{:?}", output);

        let mut cmd = Command::new("composite");
        let cmd = cmd.args(&[
            "-dissolve",
            "50",
            "-gravity",
            "Center",
            "play256.png",
            output_file_name.to_str().unwrap(),
            "-alpha",
            "Set",
            output_file_name.to_str().unwrap(),
        ]);
        trace!("{:#?}", cmd);
        let output = cmd.output().unwrap();
        trace!("{:?}", output);
    }

    output_file_name
}

#[get("/thumb/<max_size>/<path..>")]
fn thumb(
    options: State<'_, Options>,
    statistics: State<'_, Arc<RwLock<Statistics>>>,
    forwarded_identity: ForwardedIdentity,
    max_size: u64,
    path: PathBuf,
) -> Option<NamedFile> {
    let path = PathBuf::from("/").join(path);
    trace!("requesting: {:?}", &path);
    trace!("Authenticated as {}", &forwarded_identity);
    let is_folder_allowed = options.is_folder_allowed(&path, &forwarded_identity.email);
    trace!("is_folder_allowed == {}", is_folder_allowed);

    if !is_folder_allowed {
        track_unauthorized_thumb(&options, &statistics);
        None
    } else {
        trace!("{:?}", path);

        if path.as_path().is_dir() {
            None
        } else {
            track_authorized_thumb(&options, &statistics);
            trace!("extension == {:?}", path.as_path().extension());
            let extension = match path.as_path().extension() {
                Some(ext) => ext.to_str().unwrap().to_lowercase(),
                None => return None,
            };

            if IMAGE_EXTENSIONS.iter().any(|&ext| ext == extension) {
                NamedFile::open(generate_picture_thumb(
                    &options,
                    &statistics,
                    max_size,
                    &path,
                    &path,
                ))
                .ok()
            } else if VIDEO_EXTENSIONS.iter().any(|&ext| ext == extension) {
                NamedFile::open(generate_video_thumb(
                    &options,
                    &statistics,
                    max_size,
                    &path,
                    &path,
                ))
                .ok()
            } else {
                None
            }
        }
    }
}

fn is_previewable_file(file: &PathBuf) -> bool {
    let extension = match file.as_path().extension() {
        Some(ext) => ext.to_str().unwrap().to_lowercase(),
        None => return false,
    };

    IMAGE_EXTENSIONS.iter().any(|&ext| ext == extension)
        || VIDEO_EXTENSIONS.iter().any(|&ext| ext == extension)
}

#[get("/list/<file_type>/<path..>")]
fn list_files<'a>(
    options: State<'a, Options>,
    statistics: State<'a, Arc<RwLock<Statistics>>>,
    forwarded_identity: ForwardedIdentity,
    file_type: FileType,
    path: PathBuf,
) -> Response<'a> {
    let path = PathBuf::from("/").join(path);
    trace!("Authenticated as {}", &forwarded_identity);
    trace!("requested path == {:?}", &path);

    let is_folder_allowed = options.is_folder_allowed(&path, &forwarded_identity.email);
    trace!("is_folder_allowed == {}", is_folder_allowed);

    if !is_folder_allowed {
        track_unauthorized_list_files(&options, &statistics, file_type);
        let mut response = Response::new();
        response.set_status(Status::Unauthorized);
        return response;
    }

    track_authorized_list_files(&options, &statistics, file_type);

    let items = match file_type {
        FileType::Preview => {
            let a = path
                .read_dir()
                .unwrap()
                .map(|res| res.unwrap().path())
                .filter(|res| res.is_file())
                .filter(|res| is_previewable_file(&res))
                .map(|res| {
                    FileWithSize::with_size(
                        res.to_str().unwrap().to_owned(),
                        res.metadata().unwrap().len(),
                    )
                })
                .collect::<Vec<_>>();

            options.audit(
                &forwarded_identity.email,
                "preview",
                path.to_str().unwrap(),
                "list",
                true,
            );

            a
        }
        FileType::Extra => {
            let a = path
                .read_dir()
                .unwrap()
                .map(|res| res.unwrap().path())
                .filter(|res| res.is_file())
                .filter(|res| !is_previewable_file(&res))
                .map(|res| {
                    FileWithSize::with_size(
                        res.to_str().unwrap().to_owned(),
                        res.metadata().unwrap().len(),
                    )
                })
                .collect::<Vec<_>>();

            options.audit(
                &forwarded_identity.email,
                "extra",
                path.to_str().unwrap(),
                "list",
                true,
            );

            a
        }
        FileType::Folder => {
            let a = path
                .read_dir()
                .unwrap()
                .map(|res| res.unwrap().path())
                .filter(|res| res.is_dir())
                .filter(|res| options.is_folder_allowed(&res, &forwarded_identity.email))
                .map(|res| FileWithSize::without_size(res.to_str().unwrap().to_owned()))
                .collect::<Vec<_>>();

            options.audit(
                &forwarded_identity.email,
                "folder",
                path.to_str().unwrap(),
                "list",
                true,
            );

            a
        }
    };

    let mut response = Response::new();
    response.set_status(Status::Ok);
    add_access_control_allow_origin_if_needed(&mut response, &options);
    response.set_sized_body(Cursor::new(serde_json::to_string(&items).unwrap()));
    response
}

#[get("/allowed/<path..>")]
fn is_folder_allowed(
    options: State<'_, Options>,
    forwarded_identity: ForwardedIdentity,
    path: PathBuf,
) -> Response<'_> {
    trace!(
        "is_folder_allowed(forwared_identity = {:?}, path == {:?}",
        &forwarded_identity,
        &path
    );
    let path = PathBuf::from("/").join(path);

    let mut response = Response::new();
    response.set_status(Status::Ok);
    response.set_sized_body(Cursor::new(
        serde_json::to_string(&options.is_folder_allowed(&path, &forwarded_identity.email))
            .unwrap(),
    ));
    add_access_control_allow_origin_if_needed(&mut response, &options);
    response
}

#[get("/firstlevel")]
fn get_first_level_folders<'r>(
    options: State<'r, Options>,
    statistics: State<'_, Arc<RwLock<Statistics>>>,
    first_folder_by_email: State<'r, HashMap<String, Vec<String>>>,
    forwarded_identity: ForwardedIdentity,
) -> Response<'r> {
    trace!("{:#?}", first_folder_by_email);

    options.audit(
        &forwarded_identity.email,
        "first_level_folders",
        "",
        "list",
        true,
    );
    if !options.identity_allowed(&forwarded_identity) {
        track_unauthorized_first_level_folders(&options, &statistics);
        let mut response = Response::new();
        response.set_status(Status::Unauthorized);
        response
    } else {
        track_authorized_first_level_folders(&options, &statistics);
        let mut response = Response::new();
        response.set_status(Status::Ok);
        add_access_control_allow_origin_if_needed(&mut response, &options);
        response.set_sized_body(Cursor::new(
            serde_json::to_string(
                first_folder_by_email
                    .get(&forwarded_identity.email)
                    .unwrap(),
            )
            .unwrap(),
        ));
        response
    }
}

#[inline]
fn add_access_control_allow_origin_if_needed(response: &mut Response, options: &Options) {
    if let Some(access_control_allow_origin) = &options.access_control_allow_origin {
        response.adjoin_raw_header(
            "Access-Control-Allow-Origin",
            access_control_allow_origin.to_owned(),
        );
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could read config file {} error: {}", config_file.display(), source))]
    ReadConfig {
        config_file: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Could parse config file error: {}", source))]
    ParseConfig {
        options: String,
        source: toml::de::Error,
        backtrace: Backtrace,
    },
}

fn main() {
    let config_file = std::env::args()
        .nth(1)
        .expect("please pass the configuration file as first parameter");
    println!("reading configuration from {}", config_file);

    let config_file = PathBuf::from(config_file);
    if !config_file.exists() {
        eprintln!(
            "configuration file {:?} does not exists or is not accessible",
            config_file
        );
        return;
    }

    let options = std::fs::read_to_string(&config_file)
        .context(ReadConfig { config_file })
        .unwrap();
    let options: Options = (&options as &str)
        .try_into()
        .context(ParseConfig { options })
        .unwrap();

    setup_logger(&options).unwrap();

    let first_folders_by_email = options.calculate_first_level_folders_for_every_user();
    debug!("first_folders_by_email == {:#?}", first_folders_by_email);

    let statistics = Arc::new(RwLock::new(Statistics::default()));

    if let Some(prometheus_metrics_port) = options.prometheus_metrics_port {
        let statistics = statistics.clone();
        std::thread::spawn(move || {
            let cfg = rocket::config::Config::build(rocket::config::Environment::Production)
                .address("0.0.0.0")
                .port(prometheus_metrics_port)
                .workers(1) // only Prometheus will be calling it.
                .unwrap();
            rocket::custom(cfg)
                .mount("/", routes![metrics])
                .manage(statistics)
                .launch();
        });
    }

    rocket::ignite()
        .mount(
            "/",
            routes![
                path,
                thumb,
                list_files,
                get_first_level_folders,
                is_folder_allowed,
                site,
                root,
            ],
        )
        .manage(first_folders_by_email)
        .manage(options)
        .manage(statistics)
        .launch();
}
