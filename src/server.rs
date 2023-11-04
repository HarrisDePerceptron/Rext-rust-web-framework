use anyhow::{anyhow, Result};

use crate::app::application_dao::ApplicationDao;
use crate::app::application_service::ApplicationService;
use crate::application_factory::{self, ApplicationFactory};
use crate::websocket::redis_pubsub::RedisPubsubAdapter;
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
    pub appliction_factory: Arc<Mutex<ApplicationFactory>>,
    pub application_dao: Arc<ApplicationDao>,
    pub application_service: Arc<ApplicationService>,
    pub websocke_server: Arc<Mutex<WebsocketServer>>,
}

async fn adapter_loop(
    mut adapt: RedisPubsubAdapter,
    state: Arc<Mutex<WebsocketServer>>,
) -> Result<()> {
    let mut resv = adapt.run()?;

    log::info!("Adapter running!!!");

    while let Some(payload) = resv.recv().await {
        log::info!(
            "Got payload adapter loop: {}: {}",
            payload.channel,
            payload.data
        );
        let channel_split: Vec<String> =
            payload.channel.split("::").map(|e| e.to_string()).collect();

        if channel_split.len() < 2 {
            continue;
        }

        let room_name = channel_split[1].to_string();
        if room_name.is_empty() {
            continue;
        }

        let room = match state.lock() {
            Ok(mut v) => match v.get_room(room_name.as_str()) {
                Some(v) => v,
                None => {
                    log::error!("Did not get room: {}", payload.channel);
                    continue;
                }
            },
            Err(e) => {
                log::error!("State lock error: {}", e.to_string());
                continue;
            }
        };

        if let Err(e) = room.send(payload.data.as_str()).await {
            log::error!("Unable to send to room: {}", e.to_string());
            continue;
        }
    }

    log::info!("Exiting adapter loop...");

    Ok(())
}

pub async fn server(
    address: &str,
    fac: Arc<Mutex<ApplicationFactory>>,
) -> Result<tokio::task::JoinHandle<()>> {
    let websocket_server = Arc::new(Mutex::new(WebsocketServer::new(fac.clone())));

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
        appliction_factory: fac.clone(),
        websocke_server: websocket_server.clone(),
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

    let adapter = RedisPubsubAdapter::new("room::*", fac.clone());

    tokio::spawn(adapter_loop(adapter, websocket_server.clone()));
    let handler = tokio::spawn(async move {
        axum::Server::bind(&addr.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok(handler)
}
