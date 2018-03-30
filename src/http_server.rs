use rocket;
use rocket::{Rocket, State};
use rocket_contrib::{Json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct File {
    content: String,
    name: String
}

type ID = Uuid;
type FileList = Mutex<HashMap<ID, File>>;

#[get("/status")]
pub fn status() -> &'static str {
  "alive"
}

#[post("/file", format = "application/json", data = "<file>")]
pub fn add_file(file: Json<File>, file_list: State<FileList>) -> Json<Value> {
  let mut file_list = file_list.lock().expect("map lock.");
  let new_id = Uuid::new_v4();
  file_list.insert(new_id, file.0);

  Json(json!({ "status": "ok", "id": new_id.to_string() }))
}

#[get("/file/<uuid>", format = "application/json")]
pub fn get(uuid: String, file_list: State<FileList>) -> Option<Json<File>> {
  if let Ok(file_uuid) = Uuid::parse_str(&uuid) {
    let hashmap = file_list.lock().unwrap();
    hashmap.get(&file_uuid).map(|file| { Json(file.clone()) })
  }
  else {
    None
  }
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
      .mount("/", routes![status, add_file, get])
      .catch(errors![not_found])
      .manage(Mutex::new(HashMap::<ID, File>::new()))
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

    assert_eq!(file_response["name"].as_str(), Some("support_bundle/1354/container/foo.log"));
    assert_eq!(file_response["content"].as_str(), Some("Test1234"));
  }

  #[test]
  fn retrieve_nonexisting_file() {
    let client = Client::new(rocket()).expect("valid rocket instance");
    let response = client.get(format!("/file/{}", "abc")).header(ContentType::JSON).dispatch();
    assert_eq!(response.status(), Status::NotFound);

  }
}
