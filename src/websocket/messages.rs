use anyhow::{anyhow as error, Result};
use axum::extract::ws::{Message, WebSocket};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::websocket::room;

use super::{socket::AppSocket, websocket_server::WebsocketServer};

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

#[derive(strum::Display)]
pub enum SocketMessages {
    SocketClose,
}

fn get_appsocket(client_id: &str, state: Arc<Mutex<WebsocketServer>>) -> Result<AppSocket> {
    let current_socket = match state.lock() {
        Ok(v) => v.get_client(client_id).ok_or(error!(format!(
            "Unable to get client with id: {}",
            client_id
        )))?,
        Err(e) => return Err(error!(format!("Unable to lock mutex: {}", e.to_string()))),
    };

    Ok(current_socket)
}

pub async fn parse_text_messages(
    msg: String,
    client_id: &str,
    state: Arc<Mutex<WebsocketServer>>,
) -> Result<()> {
    log::info!("Got message: {}: {}", client_id, msg);

    let command: Command = serde_json::from_str(&msg)?;

    let current_appsocket = get_appsocket(client_id, state.clone())?;

    match command {
        Command::JOIN(v) => {
            if let Ok(mut state) = state.lock() {
                state.join_room(&v, current_appsocket);

                log::info!("Joined room {} with client id {}", v, client_id);
            }
        }
        Command::LEAVE(v) => {
            log::info!("wants to leave {}", v);

            if let Ok(mut state) = state.lock() {
                state.leave_room(&v, client_id)?;
            }
        }
        Command::MESSAGE(v) => {
            let mut room: Option<room::Room> = None;

            if let Ok(mut state) = state.lock() {
                room = state.get_room(&v.room);
            }

            let room = room.ok_or(error!(format!("Unable to get room: {}", v.room)))?;
            room.send(&v.message).await?;
        }
    };

    Ok(())
}

pub async fn parse_close_messages(
    msg: &str,
    client_id: &str,
    state: Arc<Mutex<WebsocketServer>>,
) -> Result<()> {
    let current_appsocket = get_appsocket(client_id, state.clone())?;

    //current_appsocket.socket.send(msg.to_string()).await?;

    current_appsocket
        .socket
        .send(SocketMessages::SocketClose.to_string())
        .await?;

    Ok(())
}

pub async fn parse_sender_message(
    msg: &str,
    sender: &mut SplitSink<WebSocket, Message>,
) -> Result<()> {
    if msg == SocketMessages::SocketClose.to_string() {
        sender.close().await?;
        return Ok(());
    }

    sender.send(Message::Text(msg.to_string())).await?;

    Ok(())
}
