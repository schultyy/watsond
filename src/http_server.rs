use rocket;
use rocket::Rocket;

#[get("/status")]
pub fn status() -> &'static str {
  "alive"
}

pub fn rocket() -> Rocket {
  rocket::ignite().mount("/", routes![status])
}

#[cfg(test)]
mod test {
  use super::rocket;
  use rocket::local::Client;
  use rocket::http::Status;

  #[test]
  fn status() {
    let client = Client::new(rocket()).expect("valid rocket instance");
    let mut response = client.get("/status").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("alive".into()));
  }
}
