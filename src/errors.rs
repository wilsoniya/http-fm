use std::convert;
use std::io;

#[derive(Debug)]
pub enum HFMError {
    DiskError {
        inner: std::io::Error,
    },
    UnicodeError,
    UnknownFileType,
}

impl convert::From<io::Error> for HFMError {
    fn from(other: io::Error) -> Self {
        Self::DiskError{ inner: other }
    }
}

impl std::fmt::Display for HFMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HFMError::DiskError{inner} => {
                match inner.kind() {
                    io::ErrorKind::NotFound => write!(f, "File not found"),
                    _ => write!(f, "Disk error")
                }
            },
            HFMError::UnicodeError{..} => write!(f, "Unicode error"),
            HFMError::UnknownFileType{..} => write!(f, "Unknown file type"),
        }
    }
}

impl actix_web::ResponseError for HFMError { }
