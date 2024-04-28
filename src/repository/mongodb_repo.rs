use std::env;
extern crate dotenv;
use dotenv::dotenv;
use mongodb::{ bson::doc, Client, Collection };
use crate::models::user_model::User;

pub struct MongoRepo {
  col: Collection<User>
}

impl MongoRepo {
  pub async fn init() -> Self {
    dotenv().ok();

    let uri = match env::var("MONGOURI") {
      Ok(v) => v.to_string(),
      Err(_) => format!("Error loading env variable"),
    };

    let client = Client::with_uri_str(uri).await.unwrap();
    let db = client.database("vrcpm");

    db.run_command(doc! { "ping": 1 }, None).await.unwrap();

    let col: Collection<User> = db.collection("Users");
    MongoRepo { col }
  }

  pub async fn find_user(&self, id: String) -> Option<User> {
    let user = self.col
      .find_one(doc! { "_id": id }, None)
      .await.ok().unwrap();

      user
  }

  pub async fn find_user_by_token(&self, token: String) -> Option<User> {
    let user = self.col
      .find_one(doc! { "token": token }, None)
      .await.ok().unwrap();

    user
  }

  pub async fn create_user(&self, new_user: User) -> User {
    let _user = self
      .col
      .insert_one(&new_user, None)
      .await.ok()
      .expect("Error creating user");

    new_user
  }

  pub async fn update_user_username(&self, user_id: String, username: String){
    self.col.update_one(doc! { "_id": user_id }, doc! { "$set": { "username": username } }, None).await.unwrap();
  }

  pub async fn update_user_avatar(&self, user_id: String, avatar: String){
    self.col.update_one(doc! { "_id": user_id }, doc! { "$set": { "avatar": avatar } }, None).await.unwrap();
  }

  pub async fn use_storage(&self, user_id: String, storage: u64){
    self.col.update_one(doc! { "_id": user_id }, doc! { "$inc": { "used": storage as i64 } }, None).await.unwrap();
  }

  pub async fn delete_storage(&self, user_id: String, storage: u64){
    self.col.update_one(doc! { "_id": user_id }, doc! { "$inc": { "used": -(storage as i64) } }, None).await.unwrap();
  }

  pub async fn reset_storage(&self, user_id: String){
    self.col.update_one(doc! { "_id": user_id }, doc! { "$set": { "used": 0, "settings": { "enable_sync": false } } }, None).await.unwrap();
  }
}