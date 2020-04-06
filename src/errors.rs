use std::convert;
use std::io;

use actix_web::http::StatusCode;

use actix_web::web::{
    HttpResponse
};

#[derive(Debug)]
pub enum HFMError {
    DiskError {
        inner: std::io::Error,
    },
    UnicodeError,
    UnknownFileType,
}

impl HFMError {
    fn get_message_status(&self) -> (&str, actix_web::http::StatusCode) {
        match self {
            HFMError::DiskError{inner} => {
                match inner.kind() {
                    io::ErrorKind::NotFound => ("File not found", StatusCode::NOT_FOUND),
                    _ => ("Disk error", StatusCode::INTERNAL_SERVER_ERROR)
                }
            },
            HFMError::UnicodeError{..} => ("Unicode error", StatusCode::INTERNAL_SERVER_ERROR),
            HFMError::UnknownFileType{..} => ("Unknown file type", StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

impl convert::From<io::Error> for HFMError {
    fn from(other: io::Error) -> Self {
        Self::DiskError{ inner: other }
    }
}

impl std::fmt::Display for HFMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_message_status().0)
    }
}

impl actix_web::ResponseError for HFMError {
    fn error_response(&self) -> HttpResponse {
        let (msg, status) = self.get_message_status();
        HttpResponse::build(status).body(String::from(msg))
    }
}
