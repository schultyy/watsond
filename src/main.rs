#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate rocket;
extern crate uuid;
extern crate regex;
extern crate bincode;

mod http_server;
mod analyzer;
mod serializer;
mod state;

use state::WatsonState;

fn main() {
  let mount_point = "/api";
  if let Ok(watson_state) = serializer::read_from_disk() {
    http_server::rocket(mount_point, watson_state).launch();
  } else {
    http_server::rocket(mount_point, WatsonState::new()).launch();
  }

}
