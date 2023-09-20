use crate::socket;

use anyhow::{anyhow, Result};

use crate::room;
use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub struct Server {
    pub sockets: Vec<socket::AppSocket>,
    pub rooms: Vec<room::Room>,
}

impl Server {
    pub fn update_room(&mut self, room: room::Room) -> Option<room::Room> {
        for r in self.rooms.iter_mut() {
            if r.id == room.id {
                r.sockets = room.sockets;
                return Some(r.clone());
            }
        }

        self.rooms.push(room.clone());
        Some(room)
    }
    pub fn join_room(&mut self, room_id: &str, client: socket::AppSocket) -> Option<room::Room> {
        let room = self.get_room(room_id);

        if let Some(mut room) = room {
            room.add_client(client);
            self.update_room(room)
        } else {
            let mut room = room::Room::new(room_id);
            room.add_client(client);
            self.update_room(room)
        }
    }

    pub fn leave_room(&mut self, room_id: &str, client_id: &str) -> Result<room::Room> {
        //

        let room = self.get_room(room_id);

        if let Some(mut room) = room {
            room.remove_client(client_id)?;
            return self
                .update_room(room)
                .ok_or(anyhow!("Room not found. not updated"));
        }

        Err(anyhow!("Could not leave room"))
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
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct HelloResponse {
    result: String,
}

pub async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<Mutex<Server>>>) -> Response {
    ws.on_upgrade(|socket| socket::handle_socket(socket, state))
}

pub async fn server(address: &str) -> Result<tokio::task::JoinHandle<()>> {
    //
    let app_state = Arc::new(Mutex::new(Server {
        sockets: Vec::new(),
        rooms: Vec::new(),
    }));

    let app = Router::new()
        .route("/", get(|| async { "Hello axum" }))
        .route(
            "/hello",
            get(Json(HelloResponse {
                result: address.to_string(),
            })),
        )
        .route("/ws", get(handler))
        .with_state(app_state);

    log::info!("Serving on {}", address);
    let addr = address.to_string();

    let handler = tokio::spawn(async move {
        axum::Server::bind(&addr.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok(handler)
}
