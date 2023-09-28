use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};

use crate::server_errors;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize, Clone)]
struct ServerErrorResponse {
    message: String,
    error: ServerError,
    code: u32,
}

#[derive(Debug, Error, Serialize, Clone)]
pub enum ServerError {
    #[error("UnAuthorized: `{0}`")]
    Unauthorized(String),

    #[error("Bad Request: `{0}`")]
    BadRequest(String),

    #[error("Internal Error: `{0}`")]
    Internal(String),
}

impl From<ServerError> for ServerErrorResponse {
    fn from(value: ServerError) -> ServerErrorResponse {
        match value {
            ServerError::Unauthorized(_) => ServerErrorResponse {
                message: "Unauthorized".to_string(),
                code: 401u32,
                error: value,
            },
            ServerError::BadRequest(_) => ServerErrorResponse {
                message: "Bad Request".to_string(),
                code: 400u32,
                error: value,
            },
            ServerError::Internal(_) => ServerErrorResponse {
                message: "Internal server error".to_string(),
                code: 401u32,
                error: value,
            },
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized(_) => (
                StatusCode::UNAUTHORIZED,
                Json(ServerErrorResponse::from(self)),
            )
                .into_response(),
            Self::BadRequest(_) => (
                StatusCode::BAD_REQUEST,
                Json(ServerErrorResponse::from(self)),
            )
                .into_response(),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ServerErrorResponse::from(ServerError::Internal(
                    "Internal server error. Check  logs".to_string(),
                ))),
            )
                .into_response(),
        }
    }
}
