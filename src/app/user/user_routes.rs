use axum::{
    extract::{Json, Path, Query, State},
    http::HeaderMap,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};

use crate::server_errors::AppError;
use crate::{app::service::Service, server::ServerState};
use anyhow::anyhow as error;
use serde::{Deserialize, Serialize};

use crate::app::dao::DaoObj;
use crate::app::dto::DTO;
use crate::app::user::User;

use super::user_service;

use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreateRequest {
    pub email: String,
    pub password: String,
}

pub async fn user_create(
    headers: HeaderMap,
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<UserCreateRequest>,
) -> Result<Json<DTO<User>>, AppError> {
    let headers_str = format!("here are the header: {:?}", headers);

    log::info!("header msg: {}", headers_str);
    let user_service = state.application_service.user.clone();
    let data = User::new(payload.email.as_str(), payload.password.as_str())?;

    let result = user_service.create(data).await?;

    Ok(result.into())
}

pub async fn list_user(
    headers: HeaderMap,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<Vec<DTO<User>>>, AppError> {
    let user_service = state.application_service.user.clone();

    let result = user_service.list(1, 10).await?;

    Ok(result.into())
}

pub async fn get_user(
    headers: HeaderMap,
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Result<Json<DTO<User>>, AppError> {
    let user_service = state.application_service.user.clone();

    let result = user_service.get(&id).await?;

    Ok(result.into())
}

pub async fn user_login(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<user_service::UserLoginRequest>,
) -> Result<Json<user_service::UserLoginResponse>, AppError> {
    let user_service = state.application_service.user.clone();
    let result = user_service.login(payload).await?;

    Ok(result.into())
}

pub fn user_routes() -> Router<Arc<ServerState>> {
    Router::new()
        .route("/", post(user_create))
        .route("/", get(list_user))
        .route("/:user_id", get(get_user))
        .route("/users/login", post(user_login))
}
