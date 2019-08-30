use actix_web::web;
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::AppData;
use crate::db::TxError;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/v1/keys", web::post().to(gen_key));
}

#[derive(serde::Deserialize)]
struct Email {
    email: String,
}

fn gen_key(data: web::Data<AppData>, email: web::Query<Email>) -> Result<String, ()> {
    data.db.with_transaction(|tx| {
        // TODO check email uniqueness
        let mut key = [0u8;48];
        rand::thread_rng().fill(&mut key[..]);
        let mut prefix = [0u8;6];
        rand::thread_rng().fill(&mut prefix);
        let hashed = hash(&key);
        tx.insert_api_key(&email.email, &prefix, &hashed)?;
        Ok(format!("{}.{}", base64::encode(&prefix), base64::encode(&key[..])))
    }).map_err(|it : TxError<()>| {
        match it {
            TxError::DbError(e) => log::error!("{:?}", e),
            TxError::InnerError(()) => {},
        };
    })
}

fn hash(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.input(bytes);
    hasher.result().to_vec()
}
