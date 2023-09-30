use std::sync::Arc;

//#![allow(unused_imports)]
use anyhow::Result;
//use thiserror::Error;

use axum_test::server;

use axum_test::auth;
use axum_test::secrets;
use dotenv::dotenv;

use axum_test::application_factory;
use tokio::sync::Mutex;

use axum_test::services::dao::Dao;
use axum_test::services::user;
use axum_test::services::user::UserPersist;

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

    let uu = user::User::new(
        "harris.perceptron@gmail.com",
        "12345",
        Some("harris".to_string()),
    )?;

    let mut fac = application_factory::ApplicationFactory::new().await?;

    let fac = Arc::new(fac);

    let dao = user::UserDao::new(fac.clone()).await?;

    let uuu = dao.find_by_email("harris0.perceptron@gmail.com").await?;

    let tok = uuu.create_token()?;

    log::info!("User token: {}", tok);

    let ddao = user::user_dao1::User1Dao::new(fac);

    let reees = ddao.list(1, 10).await?;
    log::info!("Listing meoww: {:?}", reees);

    let handler = server::server(&address).await?;

    log::info!("Server started");

    handler.await?;

    log::info!("Server Stopped!!");

    Ok(())
}
