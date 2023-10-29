use anyhow::{anyhow, Result};

use crate::app::application_dao::ApplicationDao;
use crate::app::application_service::ApplicationService;
use crate::application_factory::{self, ApplicationFactory};
use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::app::application_dao;
use crate::app::application_service;
use crate::websocket::websocket_handler::websocket_handler;
use crate::websocket::websocket_server::WebsocketServer;

use crate::app::user;

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

pub struct ServerState {
    pub appliction_factory: Arc<ApplicationFactory>,
    pub application_dao: Arc<ApplicationDao>,
    pub application_service: Arc<ApplicationService>,
    pub websocke_server: Arc<Mutex<WebsocketServer>>,
}

pub async fn server(address: &str) -> Result<tokio::task::JoinHandle<()>> {
    let factory = application_factory::ApplicationFactory::new().await?;

    let websocket_server = Arc::new(Mutex::new(WebsocketServer {
        sockets: Vec::new(),
        rooms: Vec::new(),
        factory,
    }));
    let fac2 = application_factory::ApplicationFactory::new().await?;
    let fac2 = Arc::new(fac2);

    let application_dao = Arc::new(
        application_dao::ApplicationDao::new(fac2.clone())
            .await
            .expect("Application dao unable to initialze"),
    );

    let application_service = Arc::new(
        application_service::ApplicationService::new(application_dao.clone())
            .expect("Unable to init application factory"),
    );

    let server_state = Arc::new(ServerState {
        application_service: application_service.clone(),
        application_dao: application_dao.clone(),
        appliction_factory: fac2.clone(),
        websocke_server: websocket_server,
    });

    let app = Router::new()
        .route("/", get(|| async { "Hello axum" }))
        .route(
            "/hello",
            get(Json(HelloResponse {
                result: address.to_string(),
            })),
        )
        .nest("/user", user::user_routes::user_routes())
        .route("/ws", get(websocket_handler))
        .with_state(server_state);

    //.with_state(server_state);

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
