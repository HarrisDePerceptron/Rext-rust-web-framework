use once_cell::sync::Lazy;
use std::env;

pub const ONE_MINUTE: std::time::Duration = std::time::Duration::from_secs(60);

pub static SESSION_KEY: Lazy<String> =
    Lazy::new(|| env::var("SESSION_KEY").expect("Session key not found"));

pub static TOKEN_ISSUER: Lazy<String> =
    Lazy::new(|| env::var("TOKEN_ISSUER").expect("TOKEN_ISSUER not found"));
pub static TOKEN_EXPIRY_DAYS: Lazy<String> =
    Lazy::new(|| env::var("TOKEN_EXPIRY_DAYS").expect("TOKEN_EXPIRY_DAYS not found"));

pub static REDIS_URI: Lazy<String> =
    Lazy::new(|| env::var("REDIS_URI").expect("REDIS_URI not found"));
pub static MONGO_URI: Lazy<String> =
    Lazy::new(|| env::var("MONGO_URI").expect("MONGO_URI not found"));
pub static MONGO_DATABASE: Lazy<String> =
    Lazy::new(|| env::var("MONGO_DATABASE").expect("MONGO_DATABASE not found"));

pub static SERVER_ADDRESS: Lazy<String> =
    Lazy::new(|| env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS not found"));
