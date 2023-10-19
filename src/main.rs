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

use axum_test::services::dao::DaoObj;
use axum_test::services::dto;
use axum_test::services::user;
use axum_test::services::user::UserPersist;

use axum_test::services::application_dao;

use anyhow::anyhow as error;
use axum_test::services::service;

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

    let uu = user::User::new("harris.perceptron@gmail.com", "12345")?;

    let mut fac = application_factory::ApplicationFactory::new().await?;

    let fac = Arc::new(fac);

    //let dao = user::UserDao::new(fac.clone()).await?;

    let app_dao = application_dao::APPLICATION_DAO.get_or_init(|| {
        application_dao::ApplicationDao::new(fac.clone())
            .expect("Application dao unable to initialze")
    });

    let app_dao = application_dao::APPLICATION_DAO
        .get()
        .ok_or(error!("Application dao was not initialized"))?;

    let dao = app_dao.user.clone();

    let ser = service::Service::new(dao.clone());
    let res = ser.get("6516c5511a81ede030f839c4").await?;

    log::info!("Got result: {:?}", res);
    let res2 = ser.list(1, 10).await?;
    log::info!("List result: {:?}", res2);

    let res = dao.find_by_email("harris100.perceptron@gmail.com").await?;
    println!("find by email result: {:?}", res);

    let handler = server::server(&address).await?;

    log::info!("Server started");

    handler.await?;

    log::info!("Server Stopped!!");

    Ok(())
}
