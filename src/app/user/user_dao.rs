use crate::app::dto::DTO;
use crate::app::user::user_model::User;

use anyhow::{anyhow as error, Result};
use mongodb::bson::doc;

use std::sync::Arc;

use crate::app::collections::Collections;
use crate::application_factory::ApplicationFactory;

use crate::app::dao::DaoObj;

pub struct UserDao {
    fac: Arc<ApplicationFactory>,
    collection_name: String,
}

impl DaoObj<User> for UserDao {
    fn get_factory(&self) -> Arc<ApplicationFactory> {
        self.fac.clone()
    }

    fn get_collection_name(&self) -> &str {
        &self.collection_name
    }
}

impl UserDao {
    pub fn new(fac: Arc<ApplicationFactory>) -> Result<Self> {
        Ok(Self {
            fac,
            collection_name: Collections::User.to_string(),
        })
    }

    pub async fn create_user(&mut self, email: &str, password: &str) -> Result<DTO<User>> {
        let user = User::new(email, password)?;

        let data = DTO::new(user);
        let result = self.create(data).await?;
        Ok(result)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<DTO<User>> {
        let col = self.get_collection()?;
        let user = col
            .find_one(doc! {"email": email}, None)
            .await?
            .ok_or(error!("User with email `{}` not found", email))?;

        Ok(user)
    }
}
