use rocket;
use rocket::{Rocket, State};
use rocket_contrib::{Json, Value};
use std::sync::Mutex;

#[derive(Deserialize)]
pub struct File {
    content: String,
    name: String
}

type FileList = Mutex<Vec<File>>;

#[get("/status")]
pub fn status() -> &'static str {
  "alive"
}

#[post("/file", format = "application/json", data = "<file>")]
pub fn add_file(file: Json<File>, file_list: State<FileList>) -> Json<Value> {
  let mut list = file_list.lock().expect("map lock.");
  list.push(file.0);

  Json(json!({ "status": "ok" }))
}

pub fn rocket() -> Rocket {
  rocket::ignite()
      .mount("/", routes![status, add_file])
      .manage(Mutex::new(Vec::<File>::new()))
}

#[cfg(test)]
mod test {
  use super::rocket;
  use rocket::local::Client;
  use rocket::http::{Status, ContentType};

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
    let response = client.post("/file")
        .header(ContentType::JSON)
        .body(r#"{ "name": "support_bundle/1354/container/foo.log", "content": "Test1234" }"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
  }
}
