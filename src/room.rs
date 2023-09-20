#![allow(unused_imports)]

use crate::socket;
use anyhow::{anyhow, Result};
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

    pub fn add_client(&mut self, client: socket::AppSocket) {
        //
        self.sockets.push(client);
    }

    pub fn remove_client(&mut self, id: &str) -> Result<()> {
        let index = self
            .sockets
            .iter()
            .position(|x| x.id == id)
            .ok_or(anyhow!("Client id {} not found", id))?;
        self.sockets.remove(index);

        Ok(())
    }
}
