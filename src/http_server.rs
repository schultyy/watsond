use rocket;
use rocket::{Rocket, State};
use rocket_contrib::{Json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Deserialize)]
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

pub fn rocket() -> Rocket {
  rocket::ignite()
      .mount("/", routes![status, add_file])
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
}
