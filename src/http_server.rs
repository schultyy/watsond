use rocket;
use rocket::{Rocket, State};
use rocket_contrib::{Json, Value};
use std::sync::Mutex;
use uuid::Uuid;

use analyzer;
use serializer;
use state::{WatsonState, File, FileMetadata, Workflow, WorkflowItem};

#[derive(Deserialize, Serialize)]
pub struct AnalyzedFile {
    file: File,
    findings: Vec<analyzer::Finding>
}

#[derive(Deserialize)]
pub struct CreateAnalyzer {
  analyzer: String
}

#[derive(Deserialize)]
pub struct CreateWorkflow {
  name: String,
  steps: Vec<CreateWorkflowItem>
}

#[derive(Deserialize)]
pub struct CreateWorkflowItem {
  id: u32,
  regex: String,
  context: String
}

#[get("/status")]
pub fn status() -> &'static str {
  "alive"
}

#[post("/file", format = "application/json", data = "<file>")]
pub fn add_file(file: Json<File>, state: State<Mutex<WatsonState>>) -> Json<Value> {
  let mut locked_state = state.lock().expect("map lock.");
  let new_id = Uuid::new_v4();
  locked_state.file_list.insert(new_id, file.0);
  serializer::save_to_disk(&locked_state);

  Json(json!({ "status": "ok", "id": new_id.to_string() }))
}

#[get("/files", format = "application/json")]
pub fn get_files(file_list: State<Mutex<WatsonState>>) -> Json<Vec<FileMetadata>> {
  let locked_state = file_list.lock().unwrap();
  let mut result_set = vec!();
  for (id, file) in &locked_state.file_list {
    result_set.push(FileMetadata{
      id: id.to_string(),
      name: file.name.clone()
    });
  }
  Json(result_set)
}

#[get("/file/<uuid>", format = "application/json")]
pub fn get_file(uuid: String, file_list: State<Mutex<WatsonState>>) -> Option<Json<AnalyzedFile>> {
  if let Ok(file_uuid) = Uuid::parse_str(&uuid) {
    let locked_state = file_list.lock().unwrap();
    locked_state.file_list.get(&file_uuid).map(|file| {
      Json(
        AnalyzedFile {
          file: file.clone(),
          findings: analyzer::analyze(&file.content, &locked_state.analyzers)
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

#[post("/workflow", format = "application/json", data = "<create_workflow>")]
pub fn create_workflow(create_workflow: Json<CreateWorkflow>, state: State<Mutex<WatsonState>>) -> Json<Value> {
  let mut locked_state = state.lock().expect("map lock.");
  let new_id = Uuid::new_v4();
  locked_state.workflows.insert(new_id, Workflow {
    name: create_workflow.name.clone(),
    steps: create_workflow.steps.iter()
                                .map(|wf| WorkflowItem {
                                  id: wf.id,
                                  regex: wf.regex.clone(),
                                  context: wf.context.clone()
                                })
                                .collect()
  });

  serializer::save_to_disk(&locked_state);

  Json(json!({
    "status": "Ok",
    "id": new_id.to_string()
  }))
}

#[error(404)]
fn not_found() -> Json<Value> {
  Json(json!({
    "status": "error",
    "reason": "Resource was not found."
  }))
}

pub fn rocket(mount_point: &str, state: WatsonState) -> Rocket {
  rocket::ignite()
      .mount(mount_point, routes![status, add_file, get_files, get_file, add_analyzer, get_analyzers, create_workflow])
      .catch(errors![not_found])
      .manage(Mutex::new(state))
}

#[cfg(test)]
mod test {
  use super::Uuid;
  use super::rocket;
  use rocket::local::Client;
  use rocket::http::{Status, ContentType};
  extern crate serde_json;
  use self::serde_json::Value;
  use super::*;

  fn construct_client() -> Client {
    let client = Client::new(rocket("/", WatsonState::new())).expect("valid rocket instance");
    client
  }

  #[test]
  fn status() {
    let client = construct_client();
    let mut response = client.get("/status").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("alive".into()));
  }

  #[test]
  fn add_file() {
    let client = construct_client();
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
    let client = construct_client();
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
    let client = construct_client();
    let response = client.get(format!("/file/{}", "abc")).header(ContentType::JSON).dispatch();
    assert_eq!(response.status(), Status::NotFound);
  }

  #[test]
  fn return_file_with_findings() {
    let client = construct_client();

    client.post("/analyzer")
        .header(ContentType::JSON)
        .body(r#"{ "analyzer": "INFO" }"#)
        .dispatch();

    client.post("/analyzer")
        .header(ContentType::JSON)
        .body(r#"{ "analyzer": "ERROR" }"#)
        .dispatch();

    let mut response = client.post("/file")
        .header(ContentType::JSON)
        .body(r#"{ "name": "support_bundle/1354/container/foo.log", "content": "INFO: started application\nERROR: license expired on 23-04-2017" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let v: Value = serde_json::from_str(&response.body_string().unwrap()).unwrap();
    let id = Uuid::parse_str(v["id"].as_str().unwrap()).unwrap();

    let mut response = client.get(format!("/file/{}", id)).header(ContentType::JSON).dispatch();
    assert_eq!(response.status(), Status::Ok);
    let file_response: AnalyzedFile = serde_json::from_str(&response.body_string().unwrap()).unwrap();

    assert_eq!(file_response.file.name, "support_bundle/1354/container/foo.log");
    assert_eq!(file_response.file.content, "INFO: started application\nERROR: license expired on 23-04-2017");
    assert_eq!(file_response.findings[0].line_number, 1);
    assert_eq!(file_response.findings[0].line, "INFO: started application");

    assert_eq!(file_response.findings[1].line_number, 2);
    assert_eq!(file_response.findings[1].line, "ERROR: license expired on 23-04-2017");
  }

  #[test]
  fn return_all_files() {
    let client = construct_client();

    client.post("/analyzer")
        .header(ContentType::JSON)
        .body(r#"{ "analyzer": "INFO" }"#)
        .dispatch();

    client.post("/analyzer")
        .header(ContentType::JSON)
        .body(r#"{ "analyzer": "ERROR" }"#)
        .dispatch();

    client.post("/file")
        .header(ContentType::JSON)
        .body(r#"{ "name": "support_bundle/1354/container/foo.log", "content": "INFO: started application\nERROR: license expired on 23-04-2017" }"#)
        .dispatch();

    client.post("/file")
        .header(ContentType::JSON)
        .body(r#"{ "name": "support_bundle/1354/container/bar.log", "content": "INFO: started application\nERROR: license expired on 23-04-2017" }"#)
        .dispatch();

    let mut response = client.get("/files")
                          .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let all_files_value: Value = serde_json::from_str(&response.body_string().unwrap()).unwrap();
    let all_files = all_files_value.as_array().unwrap();
    let first_file = &all_files[0];
    assert_eq!(all_files.len(), 2);

    assert!(&first_file["id"].as_str().unwrap().len() > &0);
    assert!(&first_file["name"].as_str().unwrap().len() > &0);
  }

  #[test]
  fn add_new_analyzer() {
    let client = construct_client();
    let response = client.post("/analyzer")
        .header(ContentType::JSON)
        .body(r#"{ "analyzer": "Warning" }"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
  }

  #[test]
  fn get_all_analyzers_when_none_have_been_configured() {
    let client = construct_client();
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
    let client = construct_client();

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

  #[test]
  fn define_workflow() {
    let client = construct_client();

    let mut response = client.post("/workflow")
        .header(ContentType::JSON)
        .body(r#"{
          "name": "Default Workflow",
          "steps": [
            {
              "id": 1,
              "regex": "run:received event",
              "context": "repo=([\\w\\-\\/]+)"
            },
            {
              "id": 2,
              "regex": "notifier=github_status build=",
              "context": "repo=([\\w\\-\\/]+)"
            }
          ]
        }"#).dispatch();
    assert_eq!(response.status(), Status::Ok);
    let workflow_value: Value = serde_json::from_str(&response.body_string().unwrap()).unwrap();
    let workflow_json_response = workflow_value.as_object().unwrap();
    let id = workflow_json_response["id"].as_str().unwrap();
    assert!(id.len() > 0);
  }
}
