use crate::{ models::user_model::{ User, UserSettings }, repository::mongodb_repo::MongoRepo };
use rocket::{ http::{HeaderMap, Status}, request::{self, FromRequest, Outcome}, response::{ content, Redirect }, Error, Request, State };
use randomizer::Randomizer;
use serde_json::Value;
use std::env;

#[get("/api/v1/auth")]
pub async fn create_user_no_token(
  _db: &State<MongoRepo>
) -> Redirect {
  let app_uri = format!("https://id.phazed.xyz/?oauth={}", env::var("APP_ID").unwrap());
  Redirect::to(app_uri)
}

#[get("/api/v1/auth?<token>&<id>")]
pub async fn create_user(
  db: &State<MongoRepo>,
  token: String,
  id: String
) -> Result<content::RawHtml<String>, Status> {
  let client = reqwest::Client::new();

  let auth_req = client.put(format!("https://api.phazed.xyz/id/v1/oauth/enable?apptoken={}&sesid={}", env::var("APP_TOKEN").unwrap(), id))
    .send().await.unwrap()
    .text().await.unwrap();

  let auth: Value = serde_json::from_str(&auth_req).unwrap();

  if auth["ok"].as_bool().unwrap() == false {
    return Err(Status::InternalServerError);
  }

  let data_req = client.get(format!("https://api.phazed.xyz/id/v1/profile/@me?token={}", token))
    .send().await.unwrap()
    .text().await.unwrap();

  let data: Value = serde_json::from_str(&data_req).unwrap();

  if data["ok"].as_bool().unwrap() == false {
    return Err(Status::InternalServerError);
  }

  let trash_req = client.delete(format!("https://api.phazed.xyz/id/v1/oauth?token={}", token))
    .send().await.unwrap()
    .text().await.unwrap();

  let trash: Value = serde_json::from_str(&trash_req).unwrap();

  if trash["ok"].as_bool().unwrap() == false {
    return Err(Status::InternalServerError);
  }

  let user_id = data["id"].as_str().unwrap();

  let user_data = db.find_user(user_id.to_string()).await;
  match user_data {
    Some(user) => {
      if data["username"].as_str().unwrap().to_string() != user.username {
        db.update_user_username(user_id.to_string(), data["username"].as_str().unwrap().to_string()).await;
      }

      if data["avatar"].as_str().unwrap().to_string() != user.username {
        db.update_user_avatar(user_id.to_string(), data["avatar"].as_str().unwrap().to_string()).await;
      }

      let html = format!("<style>body{{ background: black; color: white; }}</style>Authentication flow finished, you may close this tab now <script>window.location.href = ('vrcpm://auth-callback/{}')</script>", user.token);
      return Ok(content::RawHtml(html));
    }
    None => {
      let token = Randomizer::ALPHANUMERIC(64).string().unwrap();

      let user = db.create_user(User {
        _id: user_id.to_string(),
        username: data["username"].as_str().unwrap().to_string(),
        avatar: data["avatar"].as_str().unwrap().to_owned(),
        used: 0,
        storage: 0,
        token: token,
        server_version: "1.1".to_string(),
        settings: UserSettings {
          enable_sync: false
        }
      }).await;

      let html = format!("<style>body{{ background: black; color: white; }}</style>Authentication flow finished, you may close this tab now <script>window.location.href = ('vrcpm://auth-callback/{}')</script>", user.token);
      return Ok(content::RawHtml(html));
    }
  }
}

#[get("/api/v1/auth?denied=yup")]
pub async fn create_user_denied(
  _db: &State<MongoRepo>
) -> content::RawHtml<String> {
  let html = "<style>body{ background: black; color: white; }</style>Authentication flow finished, you may close this tab now <script>window.location.href = ('vrcpm://auth-callback/denied')</script>";
  return content::RawHtml(html.to_owned());
}

#[get("/api/v1/account?<token>")]
pub async fn user_account(
  db: &State<MongoRepo>,
  token: String
) -> content::RawJson<String> {
  let user = db.find_user_by_token(token).await;

  match user{
    Some(user) => {
      let data = format!("{{ \"ok\": true, \"user\": {{ \"_id\": \"{}\", \"username\": \"{}\", \"avatar\": \"{}\", \"used\": {}, \"storage\": {}, \"settings\": {{ \"enableSync\": {} }}, \"serverVersion\": \"{}\" }} }}",
        user._id,
        user.username,
        user.avatar,
        user.used,
        user.storage,
        user.settings.enable_sync,
        user.server_version
      );

      content::RawJson(data)
    }
    None => {
      content::RawJson("{\"ok\":false}".to_owned())
    }
  }
}

#[delete("/api/v1/deauth?<token>")]
pub async fn deauth_account(
  db: &State<MongoRepo>,
  token: String
) -> content::RawJson<String> {
  let user = db.find_user_by_token(token).await;

  match user{
    Some(user) => {
      let client = reqwest::Client::new();

      let data = client.delete(format!("https://api.phazed.xyz/id/v1/oauth/app?userid={}&apptoken={}", user._id, env::var("APP_TOKEN").unwrap()))
        .send().await.unwrap()
        .text().await.unwrap();

      content::RawJson(data)
    }
    None => {
      content::RawJson("{\"ok\":false}".to_owned())
    }
  }
}

#[derive(Debug)]
pub struct RequestHeaders<'h>(&'h HeaderMap<'h>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestHeaders<'r> {
  type Error = Error;
  async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
    let request_headers = req.headers();
    Outcome::Success(RequestHeaders(request_headers))
  }
}

#[get("/api/v1/updateProfile?<key>")]
pub async fn update_profile(
  db: &State<MongoRepo>,
  key: String,
  headers: RequestHeaders<'_>
) -> content::RawJson<String> {
  if key != env::var("UPDATE_KEY").unwrap(){
    return content::RawJson("{\"ok\":false}".to_owned());
  }

  let update_type = headers.0.get("update-type").nth(0);
  let user_id = headers.0.get("user").nth(0);
  let value = headers.0.get("value").nth(0);

  if update_type.is_none() {
    return content::RawJson("{\"ok\":false}".to_owned());
  }

  if user_id.is_none() {
    return content::RawJson("{\"ok\":false}".to_owned());
  }

  if value.is_none() {
    return content::RawJson("{\"ok\":false}".to_owned());
  }

  let user = db.find_user(user_id.unwrap().to_owned()).await;
  if user.is_none() {
    return content::RawJson("{\"ok\":false}".to_owned());
  }

  match update_type.unwrap(){
    "avatar" => { db.update_user_avatar(user_id.unwrap().to_owned(), value.unwrap().to_owned()).await; },
    "username" => { db.update_user_username(user_id.unwrap().to_owned(), value.unwrap().to_owned()).await; },
    _ => {}
  }

  content::RawJson("{\"ok\":true}".to_owned())
}