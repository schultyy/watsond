#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
mod http_server;

fn main() {
  rocket::ignite().mount("/", routes![http_server::status]).launch();
}
