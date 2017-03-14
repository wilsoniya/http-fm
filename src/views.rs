use std::collections::HashMap;
use std::fs::{File, metadata};
use std::path::PathBuf;

use rocket::http::hyper::header::ContentLength;
use rocket::response::{self, Responder, Response};
use rocket_contrib::Template;

use data::{DirItem, DirContext};
use db::DB;
use utils::{is_hidden, get_last_path_component};

#[get("/share/<code>/<path..>")]
pub fn share_dir(code: &str, path: PathBuf) -> Option<CodeResponse> {
    resolve_code_fpath(code)
    .and_then(|code_root| {
        // case: code resolves to an actual path
        let abs_path = code_root.join(path);

        if abs_path.is_dir() {
            abs_path.read_dir()
            .map(|dir_entries| {
                // case: success in reading stuff out of code_root dir
                let mut dir_items = dir_entries
                .map(|maybe_dir_entry| {
                    // for each item in in code_root dir
                    maybe_dir_entry.ok()
                    .and_then(|dir_entry| {
                        // case: item inside code_root is ok
                        let path = dir_entry.path();
                        path.strip_prefix(&code_root).ok()
                        .and_then(|rel_path| {
                            // case: successfully stripped code_root prefix from item
                            rel_path.to_str()
                            .and_then(|item| {
                                // case: rel_path successfully stringified
                                get_last_path_component(&rel_path.to_owned())
                                .map(|name| {
                                    // case: success got last path component
                                    let item = item.to_owned();
                                    let name = name.to_owned();
                                    DirItem { is_dir: path.is_dir(), path: item, name: name }
                                })
                            })
                        })
                    })
                })
                .filter_map(|item: Option<DirItem>| item)
                .filter(|dir_item: &DirItem| !is_hidden(&PathBuf::from(&dir_item.path)))
                .collect::<Vec<DirItem>>();

                dir_items.sort();

                dir_items
            }).ok()
            .and_then(|dir_items| {
                // case: DirItem Vec created successfully
                abs_path.strip_prefix(&code_root).ok()
                .and_then(|code_rel_path| {
                    // case: code_root relative path derived successfully
                    PathBuf::from(code).join(code_rel_path).to_str()
                    .map(|dpath| {
                        // case: code_root relative path sucessfully stringified
                        let dpath = dpath.to_owned();
                        let code_str = code.to_owned();
                        let context = DirContext {
                            dpath: dpath, items: dir_items, code: code_str
                        };
                        CodeResponse::Directory(Template::render("dir", &context))
                    })
                })
            })
        } else {
            Some(CodeResponse::Blob(abs_path))
        }
    })
}

#[get("/share/<code>")]
pub fn share(code: &str) -> Option<CodeResponse> {
    resolve_code_fpath(code).and_then(|fpath| share_dir(code, fpath))
}

#[get("/")]
pub fn index() -> Template {
    let context: HashMap<&str, &str> = HashMap::new(); Template::render("index", &context)
}

fn resolve_code_fpath(code: &str) -> Option<PathBuf> {
    // TODO: somehow save the db handle
    DB::open(None).ok()
    .and_then(|db| {
        db.get_code_path(code).ok()
        .and_then(|maybe_code_path| {
            maybe_code_path
            .map(|code_path| code_path.path)
        })
    })
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
