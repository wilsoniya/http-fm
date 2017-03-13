extern crate chrono;
extern crate clap;
extern crate rand;
extern crate rusqlite;
extern crate serde_json;

mod data;
mod db;
mod utils;

#[macro_use] extern crate serde_derive;

use std::path::PathBuf;

use clap::{App, Arg};

use db::DB;
use utils::{absolutize, generate_code};


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
    let rand_code = generate_code();
    let code = matches.value_of("CODE").unwrap_or(rand_code.as_str());

    match DB::open(None) {
        Ok(db) => {
            match db.insert_code_path(code, path.to_str().unwrap(), None) {
                Ok(_) => {
                    println!("path: {:?}", path);
                    println!("is now associated with");
                    // TODO: make this URL prefix configurable
                    println!("url: http://localhost:8000/share/{}", code);
                },
                Err(err) => {
                    println!("Something went wrong: {:?}", err);
                }
            }
        },
        Err(err) => {
            println!("An error occurred opening the database: {:?}", err);
        }

    }
}
