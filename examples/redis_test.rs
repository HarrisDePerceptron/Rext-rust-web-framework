use anyhow::{anyhow as error, Result};

use futures::prelude::*;
use redis::AsyncCommands;

use std::sync::{Arc, Mutex};
struct ApplicationContext {
    redis_connection: redis::aio::MultiplexedConnection,
}

use axum_test::application_factory::ApplicationFactory;
use axum_test::websocket::redis_pubsub::RedisPubsubAdapter;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    dotenv::dotenv()?;

    env_logger::init();

    //let client = redis::Client::open("redis://:87654321@127.0.0.1:6379")?;
    //let mut manager = client.get_tokio_connection_manager().await?;

    //let mut con = client.get_multiplexed_async_connection().await?;

    //let result: String = con.get("room::1").await?;
    //log::info!("Got result: {result}");

    //let mut con2 = client.get_connection()?;

    //let handle = std::thread::spawn(move || {
    //    if let Err(e) = pubsub_loop(con2) {
    //        log::error!("pubsub loop error: {}", e.to_string());
    //    }
    //});

    //std::thread::sleep(std::time::Duration::from_secs(2));

    //for i in 0..10 {
    //    let val: String = manager.get("room::1").await?;

    //    let p_res: std::result::Result<(), redis::RedisError> = con
    //        .publish("room::1", format!("hello {i}: {val}").as_str())
    //        .await;
    //    if let Err(e) = p_res {
    //        log::error!("Got publish error: {}", e.to_string());
    //    }
    //}

    //handle.join();
    //

    let fac = Arc::new(Mutex::new(ApplicationFactory::new().await?));
    let mut adap = RedisPubsubAdapter::new("room::*", fac);

    //let mut resv = adap.run()?;

    //for _ in 0..10 {
    //    let val = resv.recv().await.ok_or(error!("Receive error"))?;
    //    log::info!("Got value pub sub: {:?}", val);
    //}

    Ok(())
}

fn pubsub_loop(mut con: redis::Connection) -> Result<()> {
    let mut pubsub = con.as_pubsub();
    pubsub.psubscribe("room::*")?;

    loop {
        let msg = pubsub.get_message()?;

        let payload: std::result::Result<String, redis::RedisError> = msg.get_payload();

        if let Ok(payload) = payload {
            log::info!("Got  payload: {payload}");
        }
    }
    Ok(())
}
