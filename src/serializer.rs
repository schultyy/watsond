use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::collections::{HashMap, HashSet};
use bincode::{serialize, deserialize};
use state::{self, WatsonState};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct SerializableFile {
  pub content: String,
  pub name: String
}

#[derive(Serialize, Deserialize)]
struct SerializableState {
  pub file_list: HashMap<String, SerializableFile>,
  pub analyzers: HashSet<String>
}

pub fn save_to_disk(state: &WatsonState) -> Result<(), Error> {
  let encoded: Vec<u8> = serialize(&to_serializable_state(state)).unwrap();
  write_file(encoded)
}

pub fn read_from_disk() -> Result<WatsonState, Error> {
  let mut f = File::open("watson_state.bin")?;
  let mut buffer = Vec::new();
  f.read_to_end(&mut buffer)?;
  let decoded: SerializableState = deserialize(&buffer[..]).unwrap();
  Ok(to_watson_state(decoded))
}

#[cfg(test)]
fn write_file(_encoded: Vec<u8>) -> Result<(), Error> {
  Ok(())
}

#[cfg(not(test))]
fn write_file(encoded: Vec<u8>) -> Result<(), Error> {
  let mut file = File::create("watson_state.bin")?;
  file.write_all(&encoded)?;
  Ok(())
}

fn to_serializable_file(file: &state::File) -> SerializableFile {
  SerializableFile {
    content: file.content.clone(),
    name: file.name.clone()
  }
}

fn to_serializable_state(state: &WatsonState) -> SerializableState {
  let mut file_list = HashMap::new();
  for (key, value) in &state.file_list {
    file_list.insert(key.to_string(), to_serializable_file(value));
  }

  SerializableState {
    file_list: file_list,
    analyzers: state.analyzers.clone()
  }
}

fn to_watson_state(state: SerializableState) -> WatsonState {
  let mut file_list = HashMap::new();
  for (key, value) in &state.file_list {
    if let Ok(uuid) = Uuid::parse_str(&key) {
      file_list.insert(uuid, state::File { content: value.content.clone(), name: value.name.clone() });
    }
  }

  WatsonState {
    file_list: file_list,
    analyzers: state.analyzers.clone()
  }
}
