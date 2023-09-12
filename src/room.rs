#![allow(unused_imports)]

use crate::socket;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone)]
pub struct Room {
    pub id: String,
    pub sockets: Vec<socket::AppSocket>,
}

impl Room {
    pub fn total_clients(&self) -> usize {
        self.sockets.len()
    }

    pub async fn send(&self, msg: &str) -> Result<()> {
        for s in &self.sockets {
            s.socket.send(msg.to_string()).await?;
        }

        Ok(())
    }

    pub async fn add_client(&mut self, client: socket::AppSocket) {
        //
        self.sockets.push(client);
    }

    pub async fn remove_client(&mut self, id: &str) -> Result<()> {
        let index = self
            .sockets
            .iter()
            .position(|x| x.id == id)
            .ok_or(anyhow!("Client id {} not found", id))?;
        self.sockets.remove(index);

        Ok(())
    }
}
