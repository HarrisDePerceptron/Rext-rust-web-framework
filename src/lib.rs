pub mod app;
pub mod application_factory;
pub mod auth;
pub mod mongo_persistence;
pub mod room;
pub mod secrets;
pub mod server;
pub mod server_errors;
pub mod socket;
pub mod utils;

pub fn hello() {
    println!("hello world. axum test");
}
