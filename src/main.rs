#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
mod http_server;

fn main() {
  http_server::rocket().launch();
}
