use crate::{ models::user_model::{ User, UserSettings }, repository::mongodb_repo::MongoRepo };
use rocket::{ http::Status, response::{ content, Redirect }, State };
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

#[get("/api/v1/auth?<token>")]
pub async fn create_user(
  db: &State<MongoRepo>,
  token: String
) -> Result<content::RawHtml<String>, Status> {
  let client = reqwest::Client::new();

  let data_req = client.get(format!("https://api.phazed.xyz/id/v1/profile/@me?token={}", &token))
    .send().await.unwrap()
    .text().await.unwrap();

  let data: Value = serde_json::from_str(&data_req).unwrap();

  if data["ok"].as_bool().unwrap() == false {
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
      let utoken = Randomizer::ALPHANUMERIC(64).string().unwrap();

      let user = db.create_user(User {
        _id: user_id.to_string(),
        username: data["username"].as_str().unwrap().to_string(),
        avatar: data["avatar"].as_str().unwrap().to_owned(),
        used: 0,
        storage: 0,
        token: utoken,
        id_token: token,
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
  let user = db.find_user_by_token(token.clone()).await;

  match user{
    Some(user) => {
      let client = reqwest::Client::new();

      let data_req = client.get(format!("https://api.phazed.xyz/id/v1/profile/@me?token={}", &user.id_token))
        .send().await.unwrap()
        .text().await.unwrap();
    
      let data: Value = serde_json::from_str(&data_req).unwrap();
    
      if data["ok"].as_bool().unwrap() == false {
        return content::RawJson("{\"ok\":false}".to_owned());
      }

      if data["username"].as_str().unwrap().to_string() != user.username {
        db.update_user_username(user._id.clone(), data["username"].as_str().unwrap().to_string()).await;
      }

      if data["avatar"].as_str().unwrap().to_string() != user.username {
        db.update_user_avatar(user._id.clone(), data["avatar"].as_str().unwrap().to_string()).await;
      }

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

// pub struct RequestHeaders<'h>(&'h HeaderMap<'h>);

// #[rocket::async_trait]
// impl<'r> FromRequest<'r> for RequestHeaders<'r> {
//   type Error = Error;
//   async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
//     let request_headers = req.headers();
//     Outcome::Success(RequestHeaders(request_headers))
//   }
// }