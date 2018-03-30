use rocket;
use rocket::{Rocket, State};
use rocket_contrib::{Json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use uuid::Uuid;

use analyzer;

#[derive(Serialize, Deserialize, Clone)]
pub struct File {
    content: String,
    name: String
}

#[derive(Serialize)]
pub struct AnalyzedFile {
    file: File,
    findings: Vec<String>
}

#[derive(Deserialize)]
pub struct CreateAnalyzer {
  analyzer: String
}

pub struct WatsonState {
    file_list: HashMap<ID, File>,
    analyzers: HashSet<String>
}

impl WatsonState {
  pub fn new() -> WatsonState {
    WatsonState {
      file_list: HashMap::<ID, File>::new(),
      analyzers: HashSet::new()
    }
  }
}

type ID = Uuid;

#[get("/status")]
pub fn status() -> &'static str {
  "alive"
}

#[post("/file", format = "application/json", data = "<file>")]
pub fn add_file(file: Json<File>, state: State<Mutex<WatsonState>>) -> Json<Value> {
  let mut locked_state = state.lock().expect("map lock.");
  let new_id = Uuid::new_v4();
  locked_state.file_list.insert(new_id, file.0);

  Json(json!({ "status": "ok", "id": new_id.to_string() }))
}

#[get("/file/<uuid>", format = "application/json")]
pub fn get_file(uuid: String, file_list: State<Mutex<WatsonState>>) -> Option<Json<AnalyzedFile>> {
  if let Ok(file_uuid) = Uuid::parse_str(&uuid) {
    let locked_state = file_list.lock().unwrap();
    locked_state.file_list.get(&file_uuid).map(|file| {
      Json(
        AnalyzedFile {
          file: file.clone(),
          findings: analyzer::analyze(&file.content)
        }
      )
    })
  }
  else {
    None
  }
}

#[post("/analyzer", format = "application/json", data = "<analyzer>")]
pub fn add_analyzer(analyzer: Json<CreateAnalyzer>, state: State<Mutex<WatsonState>>) -> Json<Value> {
  let mut locked_state = state.lock().expect("map lock.");
  locked_state.analyzers.insert(analyzer.analyzer.clone());
  Json(json!({ "status": "ok" }))
}

#[get("/analyzers", format = "application/json")]
pub fn get_analyzers(state: State<Mutex<WatsonState>>) -> Json<Value> {
  let locked_state = state.lock().expect("map lock.");
  let analyzers = locked_state.analyzers
                    .iter()
                    .map(|st| st.clone() )
                    .collect::<Vec<String>>();
  Json(json!(analyzers))
}

#[error(404)]
fn not_found() -> Json<Value> {
  Json(json!({
    "status": "error",
    "reason": "Resource was not found."
  }))
}

pub fn rocket() -> Rocket {
  rocket::ignite()
      .mount("/", routes![status, add_file, get_file, add_analyzer, get_analyzers])
      .catch(errors![not_found])
      .manage(Mutex::new(WatsonState::new()))
}

#[cfg(test)]
mod test {
  use super::Uuid;
  use super::rocket;
  use rocket::local::Client;
  use rocket::http::{Status, ContentType};
  extern crate serde_json;
  use self::serde_json::Value;

  #[test]
  fn status() {
    let client = Client::new(rocket()).expect("valid rocket instance");
    let mut response = client.get("/status").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("alive".into()));
  }

  #[test]
  fn add_file() {
    let client = Client::new(rocket()).expect("valid rocket instance");
    let mut response = client.post("/file")
        .header(ContentType::JSON)
        .body(r#"{ "name": "support_bundle/1354/container/foo.log", "content": "Test1234" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let v: Value = serde_json::from_str(&response.body_string().unwrap()).unwrap();
    assert!(Uuid::parse_str(v["id"].as_str().unwrap()).is_ok());
  }

  #[test]
  fn retrieve_existing_file() {
    let client = Client::new(rocket()).expect("valid rocket instance");
    let mut response = client.post("/file")
        .header(ContentType::JSON)
        .body(r#"{ "name": "support_bundle/1354/container/foo.log", "content": "Test1234" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let v: Value = serde_json::from_str(&response.body_string().unwrap()).unwrap();
    let id = Uuid::parse_str(v["id"].as_str().unwrap()).unwrap();

    let mut response = client.get(format!("/file/{}", id)).header(ContentType::JSON).dispatch();
    assert_eq!(response.status(), Status::Ok);
    let file_response: Value = serde_json::from_str(&response.body_string().unwrap()).unwrap();

    assert_eq!(file_response["file"]["name"].as_str(), Some("support_bundle/1354/container/foo.log"));
    assert_eq!(file_response["file"]["content"].as_str(), Some("Test1234"));
  }

  #[test]
  fn retrieve_nonexisting_file() {
    let client = Client::new(rocket()).expect("valid rocket instance");
    let response = client.get(format!("/file/{}", "abc")).header(ContentType::JSON).dispatch();
    assert_eq!(response.status(), Status::NotFound);
  }

  #[test]
  fn return_file_with_findings() {
    let client = Client::new(rocket()).expect("valid rocket instance");
    let mut response = client.post("/file")
        .header(ContentType::JSON)
        .body(r#"{ "name": "support_bundle/1354/container/foo.log", "content": "INFO: started application\nERROR: license expired on 23-04-2017" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let v: Value = serde_json::from_str(&response.body_string().unwrap()).unwrap();
    let id = Uuid::parse_str(v["id"].as_str().unwrap()).unwrap();

    let mut response = client.get(format!("/file/{}", id)).header(ContentType::JSON).dispatch();
    assert_eq!(response.status(), Status::Ok);
    let file_response: Value = serde_json::from_str(&response.body_string().unwrap()).unwrap();

    assert_eq!(file_response["file"]["name"].as_str(), Some("support_bundle/1354/container/foo.log"));
    assert_eq!(file_response["file"]["content"].as_str(), Some("INFO: started application\nERROR: license expired on 23-04-2017"));

    assert_eq!(file_response["findings"][0].as_str(), Some("ERROR: license expired on 23-04-2017"));
  }

  #[test]
  fn add_new_analyzer() {
    let client = Client::new(rocket()).expect("valid rocket instance");
    let response = client.post("/analyzer")
        .header(ContentType::JSON)
        .body(r#"{ "analyzer": "Warning" }"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
  }

  #[test]
  fn get_all_analyzers_when_none_have_been_configured() {
    let client = Client::new(rocket()).expect("valid rocket instance");
    let mut response = client.get("/analyzers")
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let analyzers_value: Value = serde_json::from_str(&response.body_string().unwrap()).unwrap();
    let analyzers = analyzers_value.as_array().unwrap();
    assert!(analyzers.len() == 0);
  }

  #[test]
  fn get_all_analyzers() {
    let client = Client::new(rocket()).expect("valid rocket instance");

    client.post("/analyzer")
        .header(ContentType::JSON)
        .body(r#"{ "analyzer": "Warning" }"#)
        .dispatch();

    let mut response = client.get("/analyzers")
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let analyzers_value: Value = serde_json::from_str(&response.body_string().unwrap()).unwrap();
    let analyzers = analyzers_value.as_array().unwrap();
    assert!(analyzers.len() == 1);
    assert_eq!(analyzers[0], "Warning");
  }
}
