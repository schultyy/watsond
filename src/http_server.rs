#[get("/status")]
pub fn status() -> &'static str {
  "alive"
}
