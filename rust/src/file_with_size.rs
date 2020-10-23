use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FileWithSize {
    pub path: String,
    pub size: Option<u64>,
}

impl FileWithSize {
    pub fn with_size(path: String, size: u64) -> Self {
        Self {
            path,
            size: Some(size),
        }
    }

    pub fn without_size(path: String) -> Self {
        Self { path, size: None }
    }
}
