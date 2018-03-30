#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;
extern crate rocket;
extern crate uuid;
extern crate regex;

mod http_server;
mod analyzer;

fn main() {
  http_server::rocket().launch();
}
