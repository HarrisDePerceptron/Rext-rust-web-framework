use crate::app::user::User;

use crate::app::dto::DTO;

use anyhow::{anyhow as error, Result};
use std::sync::Arc;

//use super::UserDao;

use crate::app::dao::DaoObj;
use crate::app::service::Service;
use crate::app::user::user_dao::UserDao;

use crate::auth::generate_token;

use crate::secrets;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoginResponse {
    pub user: DTO<User>,
    pub token: String,
}

pub struct UserService {
    dao: Arc<UserDao>,
}

impl UserService {
    pub fn new(dao: Arc<UserDao>) -> Self {
        Self { dao }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<DTO<User>> {
        let result = self.dao.find_by_email(email).await?;

        Ok(result)
    }

    pub async fn login(&self, req: UserLoginRequest) -> Result<UserLoginResponse> {
        let result = self
            .dao
            .login(req.email.as_str(), req.password.as_str())
            .await?;

        let id = result
            .id
            .clone()
            .ok_or(error!("id is none. canot generate token"))?;

        let token = generate_token(
            &id,
            secrets::TOKEN_ISSUER.as_str(),
            secrets::TOKEN_EXPIRY_DAYS.to_string().parse()?,
        )?;

        let result = UserLoginResponse {
            token,
            user: result,
        };

        Ok(result)
    }
}

impl Service<User> for UserService {
    fn get_dao(&self) -> Arc<dyn DaoObj<User>> {
        self.dao.clone()
    }
}
