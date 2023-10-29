#![allow(unused_imports)]

use crate::websocket::socket;
use anyhow::{anyhow as error, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Default, Debug)]
pub struct Room {
    pub id: String,
    pub sockets: Vec<socket::AppSocket>,
}

impl Room {
    pub fn new(name: &str) -> Room {
        let id = name.to_string();

        Room {
            id,
            sockets: Vec::new(),
        }
    }
    pub fn total_clients(&self) -> usize {
        self.sockets.len()
    }

    pub async fn send(&self, msg: &str) -> Result<()> {
        for s in &self.sockets {
            log::info!("Sening message in room {} to client: {}", self.id, s.id);
            s.socket.send(msg.to_string()).await?;
        }

        Ok(())
    }

    pub fn add_client(&mut self, client: socket::AppSocket) -> Result<()> {
        //
        //

        let result: Vec<&socket::AppSocket> =
            self.sockets.iter().filter(|e| e.id == client.id).collect();

        if !result.is_empty() {
            return Err(error!("Client already exists in room"));
        }

        self.sockets.push(client);

        Ok(())
    }

    pub fn remove_client(&mut self, id: &str) -> Result<()> {
        self.sockets.retain(|e| e.id != id);

        Ok(())
    }
}
