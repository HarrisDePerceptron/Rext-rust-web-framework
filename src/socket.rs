#![allow(unused_imports)]

use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::server;
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct AppSocket {
    pub id: String,
    pub socket: mpsc::Sender<String>,
}

pub struct AppSocketResv {
    pub id: String,
    pub socket: mpsc::Receiver<String>,
}

pub async fn handle_socket(mut socket: WebSocket, state: Arc<Mutex<server::AppState>>) {
    log::info!("Socket connected!!");

    let (sender, resv) = socket.split();

    let (tx, mut rx) = mpsc::channel(32);
    let id = Uuid::new_v4().to_string();

    let app_socket = AppSocket {
        id: id.to_string(),
        socket: tx.clone(),
    };

    let app_socket_resc = AppSocketResv {
        id: id.to_string(),
        socket: rx,
    };

    {
        state.lock().unwrap().sockets.push(app_socket.clone());
    }

    tokio::spawn(read(resv, id, state.clone()));
    tokio::spawn(write(sender, app_socket_resc));
}

pub async fn broadcast(msg: &str, state: Arc<Mutex<server::AppState>>) {
    //
    let sockets = state.lock().unwrap().sockets.clone();
    for soc in &sockets {
        soc.socket.send(msg.to_string()).await.unwrap();
    }
}

pub async fn read(
    mut receiver: SplitStream<WebSocket>,
    id: String,
    state: Arc<Mutex<server::AppState>>,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(msg) = msg {
            log::info!("Got message: {}: {}", id, msg);

            if msg == "broadcast" {
                broadcast("hello this is a broad cast message", state.clone()).await;
            }
        } else {
            continue;
        }
    }
}

pub async fn write(mut sender: SplitSink<WebSocket, Message>, mut app_socket_resv: AppSocketResv) {
    //
    while let Some(msg) = app_socket_resv.socket.recv().await {
        sender.send(Message::Text(msg.to_string())).await.unwrap();
    }
}
