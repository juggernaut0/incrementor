use std::fmt::{self, Display, Formatter};

use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;

use crate::db::IntoTxError;

#[derive(Debug)]
pub struct WebApplicationError {
    status: StatusCode,
    msg: String,
}

impl WebApplicationError {
    pub fn new(status: StatusCode) -> WebApplicationError {
        WebApplicationError::new_with_message(status, status.canonical_reason().unwrap_or(""))
    }

    pub fn new_with_message(status: StatusCode, msg: &str) -> WebApplicationError {
        WebApplicationError {
            status,
            msg: msg.to_string()
        }
    }

    pub fn unauthorized() -> WebApplicationError {
        WebApplicationError::new(StatusCode::UNAUTHORIZED)
    }
}

impl IntoTxError for WebApplicationError {}

impl Display for WebApplicationError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.msg)
    }
}

impl ResponseError for WebApplicationError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status).content_type("text/plain; charset=utf-8").body(&self.msg)
    }
}
