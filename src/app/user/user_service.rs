use crate::app::user::User;

use crate::app::dto::DTO;

use anyhow::{anyhow as error, Result};
use std::sync::Arc;

//use super::UserDao;

use crate::app::dao::DaoObj;
use crate::app::service::Service;
use crate::app::user::user_dao::UserDao;

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
}

impl Service<User> for UserService {
    fn get_dao(&self) -> Arc<dyn DaoObj<User>> {
        self.dao.clone()
    }
}
