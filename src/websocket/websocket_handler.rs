use crate::app::service::Service;
use crate::server_errors::{self, ServerError};

use crate::websocket::socket;

use axum::{
    extract::{ws::WebSocketUpgrade, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use std::sync::{Arc, Mutex};

use crate::server::ServerState;
use crate::websocket::websocket_server::WebsocketServer;

use crate::auth;

pub async fn websocket_handler(
    headers: HeaderMap,
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> Response {
    let headers_str = format!("here are the header: {:?}", headers);

    let authorization = headers.get("Authorization");

    let auth_error =
        server_errors::ServerError::Unauthorized("Not authorized. token not found".to_string());

    let authorization = match authorization {
        Some(v) => match v.to_str() {
            Ok(v) => v.to_string(),
            Err(e) => {
                return ServerError::Unauthorized(format!("No token in header: {}", e))
                    .into_response()
            }
        },
        None => return auth_error.into_response(),
    };

    let tokens = authorization
        .split(' ')
        .map(|v| v.to_string())
        .collect::<Vec<String>>();

    if tokens.is_empty() {
        return ServerError::Unauthorized(format!("No token in header found")).into_response();
    }

    let mut token = String::new();

    if tokens.len() > 1 {
        token = tokens[1].to_string();
    }

    if tokens.len() == 1 {
        token = tokens[0].to_string();
    }

    let ver = match auth::verify_token(&token) {
        Ok(v) => v,
        Err(e) => {
            return ServerError::Unauthorized(format!("Auth token unable to verify: {}", e))
                .into_response()
        }
    };

    if !ver {
        return ServerError::Unauthorized("Token verification returned false".to_string())
            .into_response();
    }

    let decoded = match auth::decode_token(token) {
        Ok(v) => v,
        Err(e) => {
            return ServerError::Unauthorized(format!("Auth token unable to decode: {}", e))
                .into_response()
        }
    };

    let user_id = decoded.sub.to_string();

    let user = match state.application_service.user.get(&user_id).await {
        Ok(v) => v,
        Err(e) => {
            return ServerError::Unauthorized(format!("User with id {} not found: {}", user_id, e))
                .into_response()
        }
    };

    let websocket_server = state.websocke_server.clone();
    let app_fac = state.appliction_factory.clone();

    ws.on_upgrade(|socket| socket::handle_socket(socket, websocket_server, Some(user), app_fac))
}
