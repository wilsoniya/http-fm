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

use clap::{App, Arg, SubCommand};

use db::DB;
use utils::{absolutize, generate_code};


fn main() {
	let matches = App::new("http-fm CLI")
        .about("Manages shares of files and dirs through httfp-fm-daemon.")
		.subcommand(
            SubCommand::with_name("create")
                .about("Creates a new path share")
                .arg(Arg::with_name("code")
                     .short("c")
                     .long("code")
                     .value_name("CODE")
                     .help("Sets a custom share code")
                     .takes_value(true))
                .arg(Arg::with_name("PATH")
                     .help("File or directory to share")
                     .required(true)
                     .index(1)))
        .subcommand(
            SubCommand::with_name("list")
                .about("Lists file and directory shares")
                .arg(Arg::with_name("code")
                     .short("c")
                     .long("code")
                     .value_name("CODE")
                     .help("Show only shares corresponding to this code")
                     .takes_value(true)))
        .subcommand(
            SubCommand::with_name("delete")
                .about("Deletes a path share")
                .arg(Arg::with_name("CODE")
                     .help("Code of share to delete")
                     .required(true)
                     .index(1)))
        .get_matches();

    let db = open_db();

    if let Some(matches) = matches.subcommand_matches("create") {
        let path = absolutize(PathBuf::from(matches.value_of("PATH").unwrap())).unwrap();
        let rand_code = generate_code();
        let code = matches.value_of("CODE").unwrap_or(rand_code.as_str());


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

    }

    if let Some(matches) = matches.subcommand_matches("list") {
        let code_paths = db.get_all_code_paths()
        .expect("An error occurred fetching shares");

        for code_path in code_paths.iter() {
            println!("{:?}", code_path);
        }
    }

    if let Some(matches) = matches.subcommand_matches("delete") {
        let code = matches.value_of("CODE")
        .expect("Unable to read code parameter");

        db.delete_code_path(code)
        .expect("An error occurred while deleting the given share");
    }
}

fn open_db() -> DB {
    DB::open(None).expect("An error occurred opening the database")
}
