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

use crate::websocket::websocket_server::WebsocketServer;

use crate::websocket::room;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMessage {
    pub message: String,
    pub room: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    JOIN(String),
    LEAVE(String),
    MESSAGE(RoomMessage),
}

#[derive(Clone, Debug)]
pub struct AppSocket {
    pub id: String,
    pub socket: mpsc::Sender<String>,
}

pub struct AppSocketResv {
    pub id: String,
    pub socket: mpsc::Receiver<String>,
}

pub async fn handle_socket(socket: WebSocket, state: Arc<Mutex<WebsocketServer>>) {
    log::info!("Socket connected!!");

    let (sender, resv) = socket.split();

    let (tx, rx) = mpsc::channel(32);
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
            log::info!("Got message: {}: {}", client_id, msg);

            let command: Command = match serde_json::from_str(&msg) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("Command deserialzation error: {}", e.to_string());
                    continue;
                }
            };
            match command {
                Command::JOIN(v) => {
                    if let Ok(mut s) = state.lock() {
                        let client = s.get_client(&client_id);
                        if let Some(client) = client {
                            s.join_room(&v, client);

                            log::info!("Joined room {} with client id {}", v, client_id);
                        }
                    }
                }
                Command::LEAVE(v) => {
                    println!("wants to leave {}", v);

                    if let Ok(mut state) = state.lock() {
                        match state.leave_room(&v, &client_id) {
                            Ok(_) => {
                                log::info!("Client {} left room {}", client_id, v);
                            }
                            Err(e) => {
                                log::error!("{}", e.to_string());
                                continue;
                            }
                        };
                    }
                }
                Command::MESSAGE(v) => {
                    let mut room: Option<room::Room> = None;

                    if let Ok(mut state) = state.lock() {
                        room = state.get_room(&v.room);
                    }

                    if let Some(room) = room {
                        if let Err(e) = room.send(&v.message).await {
                            log::error!(
                                "Got error while sending message '{}' to room '{}': {}",
                                v.message,
                                v.room,
                                e.to_string()
                            )
                        }
                    } else {
                        log::error!("room {} not found", v.room)
                    }
                }
            };
        } else {
            continue;
        }
    }

    Ok(())
}

pub async fn write(mut sender: SplitSink<WebSocket, Message>, mut app_socket_resv: AppSocketResv) {
    //
    while let Some(msg) = app_socket_resv.socket.recv().await {
        sender.send(Message::Text(msg.to_string())).await.unwrap();
    }
}
