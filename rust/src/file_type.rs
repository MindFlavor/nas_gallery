use rocket::http::RawStr;
use rocket::request::FromParam;
use std::fmt::Display;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub enum FileType {
    Preview,
    Extra,
    Folder,
}

impl<'a> FromParam<'a> for FileType {
    type Error = ();
    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        match param.as_str() {
            "Preview" => Ok(FileType::Preview),
            "Extra" => Ok(FileType::Extra),
            "Folder" => Ok(FileType::Folder),
            _ => Err(()),
        }
    }
}

impl Display for FileType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.as_str())
    }
}

impl FileType {
    pub fn as_str(&self) -> &str {
        match self {
            FileType::Preview => "Preview",
            FileType::Extra => "Extra",
            FileType::Folder => "Folder",
        }
    }
}
