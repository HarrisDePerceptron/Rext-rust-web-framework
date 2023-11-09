use std::sync::{Arc, Mutex};

//#![allow(unused_imports)]
use anyhow::Result;

use axum_test::application_factory::ApplicationFactory;
use axum_test::server;

use axum_test::secrets;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        std::env::set_var("RUST_LOG", "info");
        "info".to_string()
    });
    println!("Using log level: {}", rust_log);
    env_logger::init();

    dotenv().expect(".dot env file unable to load");

    let address = secrets::SERVER_ADDRESS.to_string();

    let fac = ApplicationFactory::new().await?;

    let app_factory = Arc::new(Mutex::new(fac));

    let handler = server::server(&address, app_factory).await?;

    log::info!("Server started");

    handler.await?;

    log::info!("Server Stopped!!");

    Ok(())
}
