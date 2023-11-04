use anyhow::{anyhow as error, Result};
use axum::extract::ws::{Message, WebSocket};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::{
    app::application_dao::ApplicationDao, application_factory::ApplicationFactory, websocket::room,
};

use super::{socket::AppSocket, websocket_server::WebsocketServer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMessage {
    pub message: String,
    pub room: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, strum::Display)]
pub enum Command {
    JOIN(String),
    LEAVE(String),
    MESSAGE(RoomMessage),
}

#[derive(strum::Display)]
pub enum SocketMessages {
    SocketClose,
}

#[derive(strum::Display, Clone, Deserialize, Serialize, Debug)]
pub enum SocketResponseType {
    Ok,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SocketResponse {
    pub message: String,
    pub data: Option<String>,
    pub response_type: SocketResponseType,
    pub method_name: String,
}

pub async fn send_error_response(
    error_message: &str,
    method_name: &str,
    client_id: &str,
    state: Arc<Mutex<WebsocketServer>>,
) -> Result<()> {
    let app_socket = get_appsocket(client_id, state)?;

    let response = SocketResponse {
        response_type: SocketResponseType::Error,
        data: None,
        method_name: method_name.to_string(),
        message: error_message.to_string(),
    };

    //let response_str = serde_json::to_string(&response)?;

    app_socket.socket.send(response).await?;

    Ok(())
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
    app_fac: Arc<Mutex<ApplicationFactory>>,
) -> Result<()> {
    log::info!("Got message: {}: {}", client_id, msg);

    let command: Command = serde_json::from_str(&msg)?;

    let current_appsocket = get_appsocket(client_id, state.clone())?;

    match command {
        Command::JOIN(v) => {
            if let Ok(mut state) = state.lock() {
                state.join_room(&v, current_appsocket)?;

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
            //let mut room: Option<room::Room> = None;

            //if let Ok(mut state) = state.lock() {
            //    room = state.get_room(&v.room);
            //}

            //let room = room.ok_or(error!(format!("Unable to get room: {}", v.room)))?;
            //room.send(&v.message).await?;
            //

            let mut conn = match app_fac.lock() {
                Ok(v) => v.redis_provider.get_connection()?,
                Err(e) => return Err(error!("application factory lock error: {}", e.to_string())),
            };

            let channel_name = format!("room::{}", v.room);

            conn.publish(channel_name, v.message).await?;

            log::info!("Sending message through message parse");
        }
    };

    Ok(())
}

pub async fn parse_close_messages(
    client_id: &str,
    state: Arc<Mutex<WebsocketServer>>,
) -> Result<()> {
    let current_appsocket = get_appsocket(client_id, state.clone())?;

    //current_appsocket.socket.send(msg.to_string()).await?;

    let response = SocketResponse {
        message: SocketMessages::SocketClose.to_string(),
        data: None,
        method_name: String::from("parse_close_message"),
        response_type: SocketResponseType::Ok,
    };

    current_appsocket.socket.send(response).await?;

    Ok(())
}

pub async fn parse_text_response(
    msg: &SocketResponse,
    sender: &mut SplitSink<WebSocket, Message>,
) -> Result<()> {
    let response_str = serde_json::to_string(msg)?;

    sender.send(Message::Text(response_str)).await?;

    Ok(())
}

pub async fn parse_sender_message(
    msg: &SocketResponse,
    sender: &mut SplitSink<WebSocket, Message>,
    client_id: &str,
    state: Arc<Mutex<WebsocketServer>>,
) -> Result<()> {
    if msg.message == SocketMessages::SocketClose.to_string() {
        if let Err(e) = sender.close().await {
            log::error!("Sender socker close error: {}", e.to_string());
        };

        if let Ok(mut state) = state.lock() {
            state.remove_client_server(client_id)?;
        }
        return Ok(());
    }

    parse_text_response(msg, sender).await?;

    Ok(())
}
