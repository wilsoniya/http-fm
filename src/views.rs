use std::collections::HashMap;
use std::fs::DirEntry;
use std::fs::File;
use std::io::Result;
use std::path::PathBuf;

use rocket::response::Stream;
use rocket_contrib::Template;

use data::{DirItem, DirContext, CodeResponse};
use utils::{absolutize, is_hidden, get_last_path_component};


static CODE: &'static str = "c0d3";
static FPATH: &'static str = "/home/wilsoniya/IMG_20160914_141827.jpg";
static CODE2: &'static str = "fart";
static FPATH2: &'static str = "/home/wilsoniya";


// #[get("/dir/<path..>")]
// pub fn dir(path: PathBuf) -> Option<Template> {
//     let abs_path = absolutize(path);
//
//     if abs_path.is_dir() {
//         Some(abs_path.clone())
//     } else {
//         None
//     }
//     .map(|dp: PathBuf| {
//         let mut dir_items = dp.read_dir().unwrap()
//         .map(|dir_entry: Result<DirEntry>| {
//             let dir_entry = dir_entry.unwrap();
//             let path = dir_entry.path();
//             let item = dir_entry.path().to_str().unwrap().to_owned();
//             DirItem { is_dir: path.is_dir(), item: item }
//         })
//         .filter(|di| {
//             !is_hidden(&PathBuf::from(&di.item))
//         })
//         .collect::<Vec<DirItem>>();
//         dir_items.sort();
//
//         let code = "foo".to_owned();
//         let dpath = dp.to_str().unwrap().to_owned();
//
//         let context = DirContext { dpath: dpath, items: dir_items, code: code.clone() };
//         Template::render("dir", &context)
//
//     })
// }

#[get("/blob/<path..>")]
pub fn blob(path: PathBuf) -> Option<Result<Stream<File>>> {
    let abs_path = absolutize(path);

    if abs_path.is_file() {
        Some(abs_path)
    } else {
        None
    }
    .map(|dp: PathBuf| {
        File::open(dp).map(|f| Stream::from(f))
    })
}


#[get("/share/<code>/<path..>")]
pub fn share_dir(code: &str, path: PathBuf) -> Option<CodeResponse> {
    resolve_code_fpath(code)
    .and_then(|code_root| {
        let abs_path = code_root.join(path);

        if abs_path.is_dir() {
            let mut dir_items = abs_path.read_dir().unwrap()
            .map(|dir_entry: Result<DirEntry>| {
                let dir_entry = dir_entry.unwrap();
                let path = dir_entry.path();
                let rel_path = path.strip_prefix(&code_root).unwrap();
                let item = rel_path.to_str().unwrap().to_owned();
                let name = get_last_path_component(&rel_path.to_owned())
                    .unwrap().to_owned();
                DirItem { is_dir: path.is_dir(), path: item, name: name}
            })
            .filter(|di| !is_hidden(&PathBuf::from(&di.path)))
            .collect::<Vec<DirItem>>();
            dir_items.sort();

            let dpath = PathBuf::from(code).join(
                abs_path.strip_prefix(&code_root).unwrap())
                .to_str().unwrap().to_owned();

            let code_str = code.to_owned();
            let context = DirContext {
                dpath: dpath, items: dir_items, code:
                    code_str.clone() };
            Some(CodeResponse::Directory(Template::render("dir", &context)))
        } else {
            Some(CodeResponse::Blob(abs_path))
        }
    })
}

#[get("/share/<code>")]
pub fn share(code: &str) -> Option<CodeResponse> {
    resolve_code_fpath(code)
    .and_then(|fpath| {
        match fpath.exists() {
            true => Some(fpath),
            false => None,
        }
    })
    .and_then(|fpath| {
        if fpath.is_file() {
            Some(CodeResponse::Blob(fpath))
        } else if fpath.is_dir() {
            share_dir(code, PathBuf::from(""))
        } else {
            // somehow file doesnt' exist
            None
        }
    })
}

#[get("/")]
pub fn index() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("index", &context)
}

fn resolve_code_fpath(code: &str) -> Option<PathBuf> {
    match code {
        c if c == CODE => Some(PathBuf::from(FPATH)),
        c if c == CODE2 => Some(PathBuf::from(FPATH2)),
        _ => None
    }
}
