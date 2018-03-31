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
  if let Ok(watson_state) = serializer::read_from_disk() {
    http_server::rocket(watson_state).launch();
  } else {
    http_server::rocket(WatsonState::new()).launch();
  }

}
