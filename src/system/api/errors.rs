use actix_web::{HttpResponse, error::ResponseError};
use derive_more::Display;

#[derive(Debug, Display)]
pub enum ApiError {
    #[display("Internal Server Error")]
    InternalServerError,

    #[display("BadRequest: {}", _0)]
    BadRequest(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::InternalServerError => {
                HttpResponse::InternalServerError().json("Internal Server Error")
            }
            ApiError::BadRequest(message) => HttpResponse::BadRequest().json(message),
        }
    }
}
