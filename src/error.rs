use actix_web::error::ResponseError;
use actix_web::{http::StatusCode, HttpResponse};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Display, Error)]
pub struct APIErrorResponse {
    pub error: String,
}

#[derive(Debug, Error)]
pub struct APIError {
    pub message: String,
    pub code: StatusCode,
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (code: {})", self.message, self.code)
    }
}

impl APIError {
    pub fn new(message: String, code: StatusCode) -> APIError {
        APIError {
            message: message,
            code: code,
        }
    }
}

impl ResponseError for APIError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.code).body(
            serde_json::to_string(&APIErrorResponse {
                error: self.message.to_string(),
            })
            .unwrap(),
        )
    }
}
