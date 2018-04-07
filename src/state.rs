use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub type ID = Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct File {
  pub content: String,
  pub name: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileMetadata {
  pub id: String,
  pub name: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkflowItem {
  pub id: u32,
  pub regex: String,
  pub context: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Workflow {
  pub name: String,
  pub steps: Vec<WorkflowItem>
}

pub struct WatsonState {
  pub file_list: HashMap<ID, File>,
  pub analyzers: HashSet<String>,
  pub workflows: HashMap<ID, Workflow>
}

impl WatsonState {
  pub fn new() -> WatsonState {
    WatsonState {
      file_list: HashMap::<ID, File>::new(),
      analyzers: HashSet::new(),
      workflows: HashMap::<ID, Workflow>::new(),
    }
  }
}
