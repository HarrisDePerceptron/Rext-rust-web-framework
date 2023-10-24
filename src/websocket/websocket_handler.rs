use crate::{server_errors, socket};

use axum::{
    extract::{ws::WebSocketUpgrade, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use std::sync::{Arc, Mutex};

use crate::websocket::websocket_server::WebsocketServer;

pub async fn handler(
    headers: HeaderMap,
    ws: WebSocketUpgrade,
    State(state): State<Arc<Mutex<WebsocketServer>>>,
) -> Response {
    let headers_str = format!("here are the header: {:?}", headers);

    let authorization = headers.get("Authorization");

    if authorization.is_none() {
        log::error!("Authorization not found");

        return server_errors::ServerError::Unauthorized(
            "Not authorized. token not found".to_string(),
        )
        .into_response();
    }

    log::info!("header msg: {}", headers_str);

    ws.on_upgrade(|socket| socket::handle_socket(socket, state))
}
