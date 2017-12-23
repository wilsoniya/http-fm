#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
extern crate clap;
extern crate rand;
extern crate rocket;
extern crate rocket_contrib;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use clap::{App, Arg};
use rocket::config::{Config, Environment};

pub mod data;
pub mod db;
pub mod utils;
pub mod views;


fn main() {
    let matches = App::new("http-fm daemon")
	.about("Serves files and directories")
        .arg(Arg::with_name("PORT")
             .help("http-fm server listening port")
             .short("p")
             .long("port")
             .default_value("8000"))
	.get_matches();
    let config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .port(matches.value_of("PORT").unwrap().parse().unwrap())
        .finalize().unwrap();

    rocket::custom(config, true)
        .mount("/", routes![views::index, views::share_dir, views::share])
        .launch();
}
