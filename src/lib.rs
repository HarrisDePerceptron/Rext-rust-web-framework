pub mod app;
pub mod application_factory;
pub mod auth;
pub mod secrets;
pub mod server;
pub mod server_errors;
pub mod utils;

pub mod websocket;

pub mod persistence;

pub fn hello() {
    println!("hello world. axum test");
}
