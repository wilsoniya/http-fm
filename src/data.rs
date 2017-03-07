use std::cmp::Ordering;
use std::fs::{File, metadata};
use std::path::PathBuf;

use rocket::http::hyper::header::ContentLength;
use rocket::response::{self, Responder, Response};
use rocket_contrib::Template;

#[derive(Serialize)]
pub struct DirContext {
    pub dpath: String,
    pub items: Vec<DirItem>,
    pub code: String,
}

#[derive(Serialize, Eq)]
pub struct DirItem {
    pub is_dir: bool,
    pub name: String,
    pub path: String,

}

impl Ord for DirItem {
    fn cmp(&self, other: &DirItem) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for DirItem {
    fn eq(&self, other: &DirItem) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for DirItem {
    fn partial_cmp(&self, other: &DirItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub enum CodeResponse {
    Blob(PathBuf),
    Directory(Template),
}

impl<'r> Responder<'r> for CodeResponse {
    fn respond(self) -> response::Result<'r> {
        match self {
            CodeResponse::Blob(ref path) => {
                File::open(path)
                .and_then(|file| {
                    metadata(path)
                    .map(|meta| {
                        // TODO: set content-disposition for filename
                        let length = meta.len();
                        Response::build()
                        .header(ContentLength(length))
                        .streamed_body(file)
                        .finalize()
                    })
                })
                .respond()
            },
            CodeResponse::Directory(template) => template.respond(),
        }
    }
}
