use std::sync::Arc;

//#![allow(unused_imports)]
use anyhow::Result;
//use thiserror::Error;

use axum_test::server;

use axum_test::auth;
use axum_test::secrets;
use axum_test::user;
use dotenv::dotenv;

use axum_test::application_factory;
use axum_test::user::UserPersist;
use tokio::sync::Mutex;

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

    let token = auth::generate_token(
        "this is a long subject",
        &secrets::TOKEN_ISSUER.to_string(),
        secrets::TOKEN_EXPIRY_DAYS
            .to_string()
            .parse::<u64>()
            .unwrap(),
    )
    .unwrap();

    log::info!("Token is: {}", token);

    let u_req = user::UserRequest {
        name: Some("harris".to_string()),
        password: "123".to_string(),
        email: "harris.perceptron@gmail.com".to_string(),
    };

    let fac = application_factory::ApplicationFactory::new().await;
    let fac = Arc::new(Mutex::new(fac));

    let mut dao = user::UserDao::new(fac);
    let result = dao.create_user(u_req).await?;

    log::info!("User is created: {:?}", result);

    let handler = server::server(&address).await?;

    log::info!("Server started");

    handler.await?;

    log::info!("Server Stopped!!");

    Ok(())
}
