mod api;
mod models;
mod repository;

#[macro_use] extern crate rocket;

use api::{ user_api, status_api, storage_api };
use repository::mongodb_repo::MongoRepo;

#[rocket::main]
async fn main(){
  let db = MongoRepo::init().await;

  rocket::build().manage(db).mount("/", routes![
    user_api::create_user,
    user_api::create_user_no_token,
    user_api::create_user_denied,
    user_api::deauth_account,
    user_api::user_account,

    storage_api::check_upload,
    storage_api::reset_storage,
    storage_api::delete_upload,

    status_api::status_check
  ])
    .configure(rocket::Config::figment()
      .merge(("port", 7002))
      .merge(("address", "0.0.0.0"))
    )
    .launch().await.unwrap();
}
