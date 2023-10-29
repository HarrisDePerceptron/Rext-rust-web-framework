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

use crate::{
    app::{dto::DTO, user::User},
    server,
};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use tokio::sync::mpsc;

use crate::websocket::websocket_server::WebsocketServer;

use crate::websocket::messages;
use crate::websocket::room;

#[derive(Clone, Debug)]
pub struct AppSocket {
    pub id: String,
    pub socket: mpsc::Sender<String>,
    pub user: Option<DTO<User>>,
}

pub struct AppSocketResv {
    pub id: String,
    pub socket: mpsc::Receiver<String>,
}

pub async fn handle_socket(
    socket: WebSocket,
    state: Arc<Mutex<WebsocketServer>>,
    user: Option<DTO<User>>,
) {
    log::info!("Socket connected!!");

    let (sender, resv) = socket.split();

    let (tx, rx) = mpsc::channel(32);
    let id = Uuid::new_v4().to_string();

    let app_socket = AppSocket {
        id: id.to_string(),
        socket: tx.clone(),
        user,
    };

    let app_socket_resc = AppSocketResv {
        id: id.to_string(),
        socket: rx,
    };

    {
        state.lock().unwrap().sockets.push(app_socket.clone());
    }

    tokio::spawn(read(resv, id.to_string(), state.clone()));
    tokio::spawn(write(sender, app_socket_resc, id, state.clone()));
}

pub async fn broadcast(msg: &str, state: Arc<Mutex<WebsocketServer>>) {
    //
    let sockets = state.lock().unwrap().sockets.clone();
    for soc in &sockets {
        soc.socket.send(msg.to_string()).await.unwrap();
    }
}

pub async fn read(
    mut receiver: SplitStream<WebSocket>,
    client_id: String,
    state: Arc<Mutex<WebsocketServer>>,
) -> Result<()> {
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(msg) = msg {
            if let Err(e) = messages::parse_text_messages(msg, &client_id, state.clone()).await {
                log::error!("Parse message error: {}", e.to_string());
            };
        } else if let Message::Close(c) = msg {
            let mut msg = String::from("Unknown reason for closing");

            if let Some(f) = c {
                msg = format!("Closing... code: {},  reason: {}", f.code, f.reason);
            }

            if let Err(e) = messages::parse_close_messages(&msg, &client_id, state.clone()).await {
                log::error!("Close message error: {}", e.to_string());
            };

            //Break the loop since we received a close connection
            break;
        } else {
            continue;
        }
    }

    Ok(())
}

pub async fn write(
    mut sender: SplitSink<WebSocket, Message>,
    mut app_socket_resv: AppSocketResv,
    client_id: String,
    state: Arc<Mutex<WebsocketServer>>,
) {
    //

    while let Some(msg) = app_socket_resv.socket.recv().await {
        log::info!("Resvc message to send: {}", msg);

        if msg == messages::SocketMessages::SocketClose.to_string() {
            match sender.close().await {
                Ok(_) => (),
                Err(e) => {
                    log::error!("Failed to close connection: {}", e.to_string());
                }
            }

            if let Ok(mut state) = state.lock() {
                match state.remove_client_server(&client_id) {
                    Ok(_) => (),
                    Err(e) => {
                        log::error!("Removing client from server error: {}", e.to_string());
                    }
                }
            }
            break;
        }

        match sender.send(Message::Text(msg.to_string())).await {
            Ok(_) => (),
            Err(e) => {
                log::error!("Socket client message send error: {}", e.to_string());
                continue;
            }
        };
    }
}
