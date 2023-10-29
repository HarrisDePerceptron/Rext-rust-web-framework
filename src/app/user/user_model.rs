use anyhow::{anyhow as error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    email: String,
    password: String,
}

impl User {
    pub fn new(email: &str, password: &str) -> Result<Self> {
        if password.is_empty() {
            return Err(error!("Password cannot be empty"));
        }

        if password.len() < 5 {
            return Err(error!("Password cannot be less than length 5"));
        }

        if !email.contains("@") || !email.contains(".") {
            return Err(error!("Email is invalid"));
        }

        Ok(Self {
            email: email.to_string(),
            password: password.to_string(),
        })
    }
}
