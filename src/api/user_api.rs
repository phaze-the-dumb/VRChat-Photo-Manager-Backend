use crate::{ models::user_model::{ User, UserSettings }, repository::mongodb_repo::MongoRepo };
use rocket::{ http::Status, response::{ Redirect, content }, State };
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
      dbg!(&token);

      let user = db.create_user(User {
        _id: user_id.to_string(),
        username: data["username"].as_str().unwrap().to_string(),
        avatar: format!("https://cdn.phazed.xyz/id/avatars/{}/{}.png", user_id, data["avatar"].as_str().unwrap()),
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
      let data = format!("{{ \"ok\": true, \"user\": {{ \"_id\": \"{}\", \"username\": \"{}\", \"avatar\": \"{}\", \"used\": \"{}\", \"storage\": \"{}\", \"settings\": {{ \"enableSync\": {} }}, \"serverVersion\": \"{}\" }} }}",
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