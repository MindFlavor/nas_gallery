use rocket::http::RawStr;
use rocket::request::FromParam;

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
