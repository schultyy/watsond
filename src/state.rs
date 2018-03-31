use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub type ID = Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct File {
  pub content: String,
  pub name: String
}

pub struct WatsonState {
  pub file_list: HashMap<ID, File>,
  pub analyzers: HashSet<String>
}

impl WatsonState {
  pub fn new() -> WatsonState {
    WatsonState {
      file_list: HashMap::<ID, File>::new(),
      analyzers: HashSet::new()
    }
  }
}
