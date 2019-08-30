use actix_web::{HttpResponse, web};
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

fn gen_key(data: web::Data<AppData>, email: web::Query<Email>) -> HttpResponse {
    data.db.with_transaction(|tx| {
        if tx.email_exists(&email.email)? {
            return Err(WebApplicationError::new(StatusCode::BAD_REQUEST, "Email already registered").into())
        }
        let mut key = [0u8;48];
        rand::thread_rng().fill(&mut key[..]);
        let mut prefix = [0u8;6];
        rand::thread_rng().fill(&mut prefix);
        let hashed = hash(&key);
        tx.insert_api_key(&email.email, &prefix, &hashed)?;
        Ok(format!("{}.{}", base64::encode(&prefix), base64::encode(&key[..])))
    }).map(|it| {
        HttpResponse::Ok().body(it)
    }).unwrap_or_else(|it| {
        match it {
            TxError::DbError(e) => {
                log::error!("{:#?}", e);
                HttpResponse::InternalServerError().body("Internal Server Error")
            },
            TxError::InnerError(e) => e.into_http_response(),
        }
    })
}

fn hash(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.input(bytes);
    hasher.result().to_vec()
}
