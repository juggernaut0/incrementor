use actix_web::web;
use crate::AppData;
use sha2::{Sha256, Digest};
use rand::Rng;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/v1/keys", web::post().to(gen_key));
}

#[derive(serde::Deserialize)]
struct Email {
    email: String,
}

fn gen_key(data: web::Data<AppData>, email: web::Query<Email>) -> String {
    // TODO wrap in transaction
    // TODO check email uniqueness
    let mut key = [0u8;32];
    rand::thread_rng().fill(&mut key);
    let hashed = hash(&key);
    data.db.insert_api_key(&email.email, &hashed).unwrap(); // TODO handle error
    hex::encode(&key[..])
}

fn hash(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.input(bytes);
    hasher.result().to_vec()
}
