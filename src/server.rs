use anyhow::{anyhow, Result};

use crate::application_factory;
use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::websocket::websocket_handler::websocket_handler;
use crate::websocket::websocket_server::WebsocketServer;

#[derive(Debug, Serialize, Clone)]
struct ServerResponse<T> {
    message: String,
    result: T,
    code: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HelloResponse {
    result: String,
}

pub async fn server(address: &str) -> Result<tokio::task::JoinHandle<()>> {
    let factory = application_factory::ApplicationFactory::new().await?;

    let app_state = Arc::new(Mutex::new(WebsocketServer {
        sockets: Vec::new(),
        rooms: Vec::new(),
        factory,
    }));

    let app = Router::new()
        .route("/", get(|| async { "Hello axum" }))
        .route(
            "/hello",
            get(Json(HelloResponse {
                result: address.to_string(),
            })),
        )
        .route("/ws", get(websocket_handler))
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
