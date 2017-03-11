extern crate chrono;
extern crate clap;
extern crate rusqlite;
extern crate serde_json;

mod data;
mod db;
mod utils;

#[macro_use] extern crate serde_derive;

use std::path::PathBuf;

use clap::{App, Arg};

use db::DB;
use utils::absolutize;


fn main() {
	let matches = App::new("http-fm CLI")
		.about("Manages files and directoreis")
		.arg(Arg::with_name("code")
			 .short("c")
			 .long("code")
			 .value_name("CODE")
			 .help("Sets a custom share code")
			 .takes_value(true))
		.arg(Arg::with_name("PATH")
			 .help("File or directory to share")
			 .required(true)
			 .index(1))
		.get_matches();

    let path = absolutize(PathBuf::from(matches.value_of("PATH").unwrap())).unwrap();
    let code = matches.value_of("CODE").unwrap_or(generate_code());

    if let Ok(db) = DB::open(None) {
        let result = db.insert_code_path(code, path.to_str().unwrap(), None);
    } else {
    }
}

fn generate_code<'a>() -> &'a str {
    "asdf"
}
