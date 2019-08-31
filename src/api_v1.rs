use actix_web::{web};
use actix_web::http::StatusCode;
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::AppData;
use crate::db::TxError;
use crate::util::WebApplicationError;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/v1/keys", web::post().to(gen_key));
}

#[derive(serde::Deserialize)]
struct Email {
    email: String,
}

fn gen_key(data: web::Data<AppData>, email: web::Query<Email>) -> Result<String, WebApplicationError> {
    data.db.with_transaction(|tx| {
        if tx.email_exists(&email.email)? {
            return Err(WebApplicationError::new_with_message(StatusCode::BAD_REQUEST, "Email already registered").into())
        }
        let mut key = [0u8;48];
        rand::thread_rng().fill(&mut key[..]);
        let mut prefix = [0u8;6];
        rand::thread_rng().fill(&mut prefix);
        let hashed = hash(&key);
        tx.insert_api_key(&email.email, &prefix, &hashed)?;
        Ok(format!("{}.{}", base64::encode(&prefix), base64::encode(&key[..])))
    }).map_err(|it| {
        match it {
            TxError::DbError(e) => {
                log::error!("{:#?}", e);
                WebApplicationError::new(StatusCode::INTERNAL_SERVER_ERROR)
            },
            TxError::InnerError(e) => e,
        }
    })
}

fn hash(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.input(bytes);
    hasher.result().to_vec()
}
