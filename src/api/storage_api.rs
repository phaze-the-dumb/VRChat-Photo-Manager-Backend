use crate::repository::mongodb_repo::MongoRepo;
use rocket::{ response::content, State };
use std::env;

#[get("/api/v1/upload?<token>&<size>&<worker_key>")]
pub async fn check_upload(
  db: &State<MongoRepo>,
  token: String,
  size: u64,
  worker_key: String
) -> content::RawJson<String> {
  if worker_key != env::var("WORKER_KEY").unwrap() {
    return content::RawJson("{\"ok\":false,\"error\":\"Incorrect API Key\"}".to_owned());
  }

  let user = db.find_user_by_token(token).await;

  match user{
    Some(user) => {
      if user.used + size > user.storage {
        return content::RawJson("{\"ok\":false,\"error\":\"Not enough storage\"}".to_owned());
      }

      if user.settings.enable_sync == false{
        return content::RawJson("{\"ok\":false,\"error\":\"Not enough storage\"}".to_owned());
      }

      db.use_storage(user._id, size).await;
      content::RawJson("{\"ok\":true}".to_owned())
    },
    None => {
      content::RawJson("{\"ok\":false,\"error\":\"User does not exist\"}".to_owned())
    }
  }
}

#[get("/api/v1/resetstorage?<token>&<worker_key>")]
pub async fn reset_storage(
  db: &State<MongoRepo>,
  token: String,
  worker_key: String
) -> content::RawJson<String> {
  if worker_key != env::var("WORKER_KEY").unwrap() {
    return content::RawJson("{\"ok\":false,\"error\":\"Incorrect API Key\"}".to_owned());
  }

  let user = db.find_user_by_token(token).await;

  match user{
    Some(user) => {
      db.reset_storage(user._id).await;
      content::RawJson("{\"ok\":true}".to_owned())
    },
    None => {
      content::RawJson("{\"ok\":false,\"error\":\"User does not exist\"}".to_owned())
    }
  }
}