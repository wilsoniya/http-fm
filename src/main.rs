#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
extern crate rocket;
extern crate rocket_contrib;
extern crate rusqlite;
extern crate serde_json;

mod data;
mod db;
mod utils;
mod views;

#[macro_use] extern crate serde_derive;

fn main() {
    rocket::ignite()
        .mount("/", routes![views::index, views::share_dir, views::share])
        .launch();
}
