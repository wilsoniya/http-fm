use std::fs;
use std::convert::TryInto;

use serde::{
    Serialize,
    Deserialize,
};

pub fn ls(path: &str) -> Result<DirectoryListing, String> {
    let maybe_dir_entries: Result<_, _> = fs::read_dir(path);
    maybe_dir_entries
        .map_err(|_| String::from("shits fucked"))
        .and_then(|read_dir: fs::ReadDir| {
            read_dir
                .map(|maybe_dir_entry| {
                    maybe_dir_entry
                        .map_err(|_| String::from("shits fucked"))
                        .and_then(|dir_entry| dir_entry.try_into())
                })
            .collect()
        })
        .map(|items| {
            DirectoryListing { items }
        })
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
    type Error = String;

    fn try_from(dir_entry: std::fs::DirEntry) -> Result<Self, Self::Error> {
        dir_entry
            .path()
            .to_str()
            .ok_or(String::from("file path contains unicode errors"))
            .and_then(|path| {
                dir_entry.metadata()
                    .map_err(|_| "shit's fucked".into())
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
                            Err("unknown file type".to_string())
                        }
                    })
            })
    }
}
