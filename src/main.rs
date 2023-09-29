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
        password: "12345".to_string(),
        email: "harris2.perceptron@gmail.com".to_string(),
    };

    let mut fac = application_factory::ApplicationFactory::new();
    fac.mongo_provider.connect().await?;

    let fac = Arc::new(fac);

    let mut dao = user::UserDao::new(fac).await?;

    //for i in 0..20 {
    //    //
    //    let email = format!("harris{}.perceptron@gmail.com", i);
    //    let ureq = user::UserRequest {
    //        name: None,
    //        password: "123456".to_string(),
    //        email,
    //    };

    //    dao.create_user(ureq).await?;
    //}

    //let result = dao.create_user(u_req).await?;

    //log::info!("User created: {:?}", result);
    let mut uu = dao.get_user("6516c5511a81ede030f839c4").await?;
    uu.name = Some("muhammad harris".to_string());

    dao.update_user(&uu).await?;

    //log::info!("User search: {:?}", uu);

    let all_users = dao.list_users(2, 10).await?;
    log::info!("Users data: {:?}", all_users);
    let handler = server::server(&address).await?;

    log::info!("Server started");

    handler.await?;

    log::info!("Server Stopped!!");

    Ok(())
}
