use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Eq, Clone, Debug, Serialize, Default, Deserialize)]
pub struct Folder {
    pub path: String,
    pub inheritable: Option<bool>,
    pub breaks_inheritance: Option<bool>,
    pub allowed: Option<Vec<String>>,
    pub denied: Option<Vec<String>>,
}

impl PartialEq for Folder {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Ord for Folder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for Folder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
