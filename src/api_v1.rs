use actix_web::web;
use crate::AppData;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/v1/keys", web::post().to(gen_key));
}

#[derive(serde::Deserialize)]
struct Email {
    email: String,
}

fn gen_key(data: web::Data<AppData>, email: web::Query<Email>) -> String {
    let key = "foobar".to_string(); // TODO generate random
    // TODO hash key
    data.db.insert_api_key(&email.email, &key).unwrap(); // TODO handle error
    key
}
