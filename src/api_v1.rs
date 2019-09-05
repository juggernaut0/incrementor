use actix_web::{HttpRequest, web};
use actix_web::http::{HeaderValue, StatusCode};
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::AppData;
use crate::util::{unwrap_tx_error, WebApplicationError};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1")
        .route("/keys", web::post().to(gen_key))
        .service(web::resource("/counter/{tag}")
            .route(web::get().to(get_counter))
            .route(web::post().to(inc_counter))
        )
    );
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
    }).map_err(unwrap_tx_error)
}

fn get_counter(data: web::Data<AppData>, tag: web::Path<String>, req: HttpRequest) -> Result<String, WebApplicationError> {
    let auth_header = req.headers().get("Authorization");
    data.db.with_transaction(|tx| {
        let (prefix, hashed_key) = extract_key(auth_header)?;
        let owner_id = tx.get_user_id_by_key(&prefix, &hashed_key)?.ok_or_else(|| WebApplicationError::unauthorized())?;
        let value = tx.get_counter_by_tag_locking(owner_id, &tag)?.map(|(_, v)| v).unwrap_or(0);
        Ok(value.to_string())
    }).map_err(unwrap_tx_error)
}

fn inc_counter(data: web::Data<AppData>, tag: web::Path<String>, req: HttpRequest) -> Result<String, WebApplicationError> {
    let auth_header = req.headers().get("Authorization");
    data.db.with_transaction(|tx| {
        let (prefix, hashed_key) = extract_key(auth_header)?;
        let owner_id = tx.get_user_id_by_key(&prefix, &hashed_key)?.ok_or_else(|| WebApplicationError::unauthorized())?;
        let value = loop {
            let counter = tx.get_counter_by_tag_locking(owner_id, &tag)?;
            break if let Some((counter_id, value)) = counter {
                let new_value = value + 1; // safe because db lock is acquired here
                tx.update_counter(counter_id, new_value)?;
                new_value
            } else {
                let initial = 1;
                if tx.create_counter(owner_id, &tag, initial)? {
                    initial
                } else {
                    log::warn!("Failed to create counter, trying update");
                    continue
                }
            }
        };
        Ok(value.to_string())
    }).map_err(unwrap_tx_error)
}

fn hash(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.input(bytes);
    hasher.result().to_vec()
}

fn extract_key(auth: Option<&HeaderValue>) -> Result<(Vec<u8>, Vec<u8>), WebApplicationError> {
    auth.and_then(|header| {
        let s = header.to_str().unwrap(); // TODO error check
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return None
        }
        let prefix = base64::decode(parts[0]).ok()?;
        let key = base64::decode(parts[1]).ok()?;
        let hashed = hash(&key);
        Some((prefix, hashed))
    }).ok_or_else(|| WebApplicationError::unauthorized())
}
