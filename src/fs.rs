use std::fs;
use std::path::Path;
use tokio;

use serde::Serialize;

use crate::errors;

/// An item that appears in a filesystem; either a file or a directory.
pub enum FSItem {
    /// The contents of a directory
    Directory(DirectoryListing),
    /// An open file and its length
    File(tokio::fs::File, u64),
}

impl FSItem {
    pub async fn new(path: &Path) -> Result<Self, errors::HFMError> {
        match tokio::fs::metadata(path).await {
            Ok(metadata) => {
                if metadata.is_file() {
                    tokio::fs::File::open(path).await
                        .map_err(errors::HFMError::from)
                        .map(|file| Self::File(file, metadata.len()))
                } else if metadata.is_dir() {
                    fs::read_dir(path)
                        .map_err(errors::HFMError::from)
                        .and_then(|read_dir| {
                            read_dir
                                .map(|maybe_dir_entry| {
                                    maybe_dir_entry
                                        .map_err(errors::HFMError::from)
                                        .and_then(<DirItem as std::convert::TryFrom<std::fs::DirEntry>>::try_from)
                                })
                            .collect()
                        })
                    .map(|items| Self::Directory(DirectoryListing { items }))
                } else {
                    Err(errors::HFMError::UnknownFileType)
                }
            },
            Err(err) => Err(errors::HFMError::from(err)),
        }
    }
}

#[derive(Serialize)]
pub struct DirectoryListing {
    pub items: Vec<DirItem>,
}

#[derive(Serialize)]
pub enum DirItem {
    File {
        path: String,
        size_bytes: u64,
    },
    Directory {
        path: String,
    }
}

impl std::convert::TryFrom<std::fs::DirEntry> for DirItem {
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
