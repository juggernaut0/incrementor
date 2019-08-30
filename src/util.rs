use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use crate::db::IntoTxError;

pub struct WebApplicationError {
    status: StatusCode,
    msg: String,
}

impl WebApplicationError {
    pub fn new(status: StatusCode, msg: &str) -> WebApplicationError {
        WebApplicationError {
            status,
            msg: msg.to_string()
        }
    }

    pub fn into_http_response(self) -> HttpResponse {
        HttpResponse::build(self.status).body(self.msg)
    }
}

impl IntoTxError for WebApplicationError {}
