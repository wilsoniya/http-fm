use core::pin::Pin;
use std::convert::TryInto;
use std::fs;
use std::path::Path;

use actix_web::web::Bytes;
use async_std::fs::File;
use futures::stream::Stream;
use futures::task::{Context, Poll};

use serde::{
    Serialize,
    Deserialize,
};

use crate::errors;

// struct StreamedFile(File);
//
// impl StreamedFile {
//     pub fn new(file: File) -> Self {
//         Self(file)
//     }
// }
//
// impl Stream for StreamedFile {
//     type Item = Result<Bytes, errors::HFMError>;
//
//     fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
//
//     }
// }


pub fn ls(path: &Path) -> Result<DirectoryListing, errors::HFMError> {
    let abs_path = Path::new("/").join(path);
    let maybe_dir_entries = fs::read_dir(abs_path);
    maybe_dir_entries
        .map_err(std::io::Error::into)
        .and_then(|read_dir: fs::ReadDir| {
            read_dir
                .map(|maybe_dir_entry| {
                    maybe_dir_entry
                        .map_err(std::io::Error::into)
                        .and_then(|dir_entry| dir_entry.try_into())
                })
            .collect()
        })
        .map(|items| DirectoryListing { items })
}

#[derive(Serialize)]
pub struct DirectoryListing {
    pub items: Vec<FSItem>,
}

#[derive(Serialize)]
pub enum FSItem {
    File {
        path: String,
        size_bytes: u64,
    },
    Directory {
        path: String,
    }
}

impl std::convert::TryFrom<std::fs::DirEntry> for FSItem {
    type Error = errors::HFMError;

    fn try_from(dir_entry: std::fs::DirEntry) -> Result<Self, Self::Error> {
        dir_entry
            .path()
            .to_str()
            .ok_or(errors::HFMError::UnicodeError)
            .and_then(|path| {
                dir_entry.metadata()
                    .map_err(std::io::Error::into)
                    .and_then(|meta| {
                        if meta.is_file() {
                            Ok(Self::File {
                                path: path.into(),
                                size_bytes: meta.len(),
                            })
                        } else if meta.is_dir() {
                            Ok(Self::Directory {
                                path: path.into(),
                            })
                        } else {
                            Err(errors::HFMError::UnknownFileType)
                        }
                    })
            })
    }
}
