use std::sync::{Arc, Mutex};

use crate::websocket::socket;

use anyhow::{anyhow as error, Result};

use crate::application_factory::ApplicationFactory;
use crate::websocket::room;
use serde::{Deserialize, Serialize};

use super::{redis_pubsub::RedisPubsubAdapter, socket::AppSocket};

#[derive(Debug, Serialize, Clone)]
struct ServerResponse<T> {
    message: String,
    result: T,
    code: u32,
}

pub struct WebsocketServer {
    sockets: Vec<socket::AppSocket>,
    rooms: Vec<room::Room>,
    factory: Arc<Mutex<ApplicationFactory>>,
}

impl WebsocketServer {
    pub fn new(fac: Arc<Mutex<ApplicationFactory>>) -> Self {
        Self {
            sockets: vec![],
            rooms: vec![],
            factory: fac,
        }
    }

    pub fn add_client(&mut self, app_socket: AppSocket) {
        self.sockets.push(app_socket);
    }
    pub fn update_room(&mut self, room: room::Room) -> room::Room {
        for r in self.rooms.iter_mut() {
            if r.id == room.id {
                r.sockets = room.sockets;
                return r.clone();
            }
        }

        self.rooms.push(room.clone());
        room
    }
    pub fn join_room(&mut self, room_id: &str, client: socket::AppSocket) -> Result<room::Room> {
        let room = self.get_room(room_id);

        if let Some(mut room) = room {
            if let Err(e) = room.add_client(client) {
                Err(error!("Error adding client to room: {}", e.to_string()))
            } else {
                Ok(self.update_room(room))
            }
        } else {
            let mut room = room::Room::new(room_id);
            if let Err(e) = room.add_client(client) {
                Err(error!("Error adding client to room: {}", e.to_string()))
            } else {
                Ok(self.update_room(room))
            }
        }
    }

    pub fn leave_room(&mut self, room_id: &str, client_id: &str) -> Result<room::Room> {
        //

        let room = self.get_room(room_id);

        if let Some(mut room) = room {
            room.remove_client(client_id)?;
            return Ok(self.update_room(room));
        }

        Err(error!("Could not leave room"))
    }

    pub fn get_room(&mut self, room_id: &str) -> Option<room::Room> {
        let result: Vec<&room::Room> = self.rooms.iter().filter(|e| e.id == room_id).collect();

        let mut room: Option<room::Room> = None;

        if !result.is_empty() {
            let r = result[0];

            room = Some(r.clone())
        }

        room
    }

    pub fn get_client(&self, client_id: &str) -> Option<socket::AppSocket> {
        let result: Vec<&socket::AppSocket> =
            self.sockets.iter().filter(|e| e.id == client_id).collect();

        if result.is_empty() {
            None
        } else {
            let res = result[0];
            Some(res.clone())
        }
    }

    pub async fn send_to_room(&mut self, room_id: &str, message: &str) -> Result<()> {
        let room = self.get_room(room_id);
        if let Some(room) = room {
            room.send(message).await?;
        }

        Ok(())
    }

    pub fn remove_client_server(&mut self, client_id: &str) -> Result<()> {
        //remove from all the rooms first
        for r in &mut self.rooms {
            if r.remove_client(client_id).is_ok() {
                log::debug!("Client {} removed from room {}", client_id, r.id);
            }
        }

        self.sockets.retain(|e| e.id != client_id);

        Ok(())
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct HelloResponse {
    result: String,
}
