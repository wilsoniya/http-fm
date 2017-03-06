#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate serde_json;

mod utils;

#[macro_use] extern crate serde_derive;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::DirEntry;
use std::fs::File;
use std::io::Result;
use std::path::PathBuf;

use rocket::response::Stream;
use rocket_contrib::Template;

use utils::{absolutize, is_hidden};

#[derive(Serialize)]
struct DirContext {
    dpath: String,
    items: Vec<DirItem>
}

#[derive(Serialize, Eq)]
struct DirItem {
    is_dir: bool,
    item: String
}

impl Ord for DirItem {
    fn cmp(&self, other: &DirItem) -> Ordering {
        self.item.cmp(&other.item)
    }
}

impl PartialEq for DirItem {
    fn eq(&self, other: &DirItem) -> bool {
        self.item == other.item
    }
}

impl PartialOrd for DirItem {
    fn partial_cmp(&self, other: &DirItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[get("/dir/<path..>")]
fn dir(path: PathBuf) -> Option<Template> {
    let abs_path = absolutize(path);

    if abs_path.is_dir() {
        Some(abs_path.clone())
    } else {
        None
    }
    .map(|dp: PathBuf| {
        let mut dir_items = dp.read_dir().unwrap()
        .map(|dir_entry: Result<DirEntry>| {
            let dir_entry = dir_entry.unwrap();
            let path = dir_entry.path();
            let item = dir_entry.path().to_str().unwrap().to_owned();
            DirItem { is_dir: path.is_dir(), item: item }
        })
        .filter(|di| {
            !is_hidden(&abs_path, &PathBuf::from(&di.item.to_owned().clone()))
        })
        .collect::<Vec<DirItem>>();
        dir_items.sort();

        let dpath = dp.to_str().unwrap().to_owned();

        let context = DirContext { dpath: dpath, items: dir_items };
        Template::render("dir", &context)

    })
}

#[get("/blob/<path..>")]
fn blob(path: PathBuf) -> Option<Result<Stream<File>>> {
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

#[get("/")]
fn index() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("index", &context)
}

fn main() {
    rocket::ignite().mount("/", routes![dir, blob, index]).launch();
}
