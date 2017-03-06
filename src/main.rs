#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate serde_json;

mod data;
mod utils;
mod views;

#[macro_use] extern crate serde_derive;

fn main() {
    rocket::ignite()
        .mount("/", routes![views::dir, views::blob, views::index]).launch();
}
