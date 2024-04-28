use crate::repository::mongodb_repo::MongoRepo;
use rocket::{ response::content, State };

#[get("/api/v1/status")]
pub async fn status_check(
  _db: &State<MongoRepo>
) -> content::RawJson<String> {
  content::RawJson("{\"ok\":true}".to_owned())
}