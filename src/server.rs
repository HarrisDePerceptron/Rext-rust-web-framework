use crate::socket;

use anyhow::Result;

use axum::{
    extract::{ws::WebSocketUpgrade, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub sockets: Vec<socket::AppSocket>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HelloResponse {
    result: String,
}

pub async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<Mutex<AppState>>>) -> Response {
    ws.on_upgrade(|socket| socket::handle_socket(socket, state))
}

pub async fn server(address: &str) -> Result<tokio::task::JoinHandle<()>> {
    //
    let app_state = Arc::new(Mutex::new(AppState {
        sockets: Vec::new(),
    }));

    let app = Router::new()
        .route("/", get(|| async { "Hello axum" }))
        .route(
            "/hello",
            get(Json(HelloResponse {
                result: address.to_string(),
            })),
        )
        .route("/ws", get(handler))
        .with_state(app_state);

    log::info!("Serving on {}", address);
    let addr = address.to_string();

    let handler = tokio::spawn(async move {
        axum::Server::bind(&addr.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok(handler)
}
