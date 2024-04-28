use serde::{ Serialize, Deserialize };

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSettings{
  pub enable_sync: bool
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User{
  pub _id: String,
  pub username: String,
  pub avatar: String,
  pub used: u64,
  pub storage: u64,
  pub token: String,
  pub server_version: String,
  pub settings: UserSettings
}