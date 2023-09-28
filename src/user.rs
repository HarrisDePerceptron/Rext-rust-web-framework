use crate::application_factory::ApplicationFactory;
use crate::auth;
use crate::secrets;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use tokio::sync::Mutex;

use uuid::Uuid;

use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    name: Option<String>,
    email: String,
    password: String,
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTokenPayload {
    id: String,
    email: String,
}

#[async_trait]
pub trait UserPersist {
    async fn create_user(&mut self, req: UserRequest) -> Result<User>;
    async fn get_user(id: &str) -> Result<User>;
    async fn update_user(&self) -> Result<User>;
    async fn delete_user(&self);
}

impl User {
    fn new(email: &str, password: &str, name: Option<String>) -> User {
        let name_s = name.map(|v| v.to_string());
        let id = Uuid::new_v4().to_string();

        User {
            name: name_s,
            email: email.to_string(),
            password: password.to_string(),
            id,
        }
    }

    fn create_token(&self) -> Result<String> {
        let user_token_payload = UserTokenPayload {
            email: self.email.to_string(),
            id: self.id.to_string(),
        };

        let payload_str = serde_json::to_string(&user_token_payload)?;
        let expiry_days = secrets::TOKEN_EXPIRY_DAYS.to_string();

        let expiry_days = expiry_days.parse::<u64>()?;

        let token = auth::generate_token(
            &payload_str,
            &secrets::TOKEN_ISSUER.to_string(),
            expiry_days,
        )
        .map_err(|e| anyhow!("{}", e))?;

        Ok(token)
    }
}

pub struct UserDao {
    factory: Arc<Mutex<ApplicationFactory>>,
}

impl UserDao {
    pub fn new(factory: Arc<Mutex<ApplicationFactory>>) -> Self {
        Self { factory }
    }
}

pub struct UserRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[async_trait]
impl UserPersist for UserDao {
    async fn create_user(&mut self, req: UserRequest) -> Result<User> {
        let db = self.factory.lock().await.get_database();
        let col = db.collection::<User>("User");

        let mut user = User::new(&req.email, &req.password, req.name);

        let res = col.insert_one(user.clone(), None).await?;
        let iid = res.inserted_id.to_string();

        user.id = iid;

        Ok(user)
    }

    async fn get_user(id: &str) -> Result<User> {
        todo!()
    }

    async fn update_user(&self) -> Result<User> {
        todo!()
    }

    async fn delete_user(&self) {
        todo!()
    }
}
