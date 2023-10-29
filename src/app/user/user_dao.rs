use crate::app::dto::DTO;
use crate::app::user::user_model::User;

use anyhow::{anyhow as error, Result};
use mongodb::bson::doc;

use std::collections;
use std::sync::Arc;

use crate::app::collections::Collections;
use crate::application_factory::ApplicationFactory;

use crate::app::dao::DaoObj;
use mongodb::{options::IndexOptions, IndexModel};

use async_trait::async_trait;

pub struct UserDao {
    fac: Arc<ApplicationFactory>,
    collection_name: String,
}

#[async_trait]
impl DaoObj<User> for UserDao {
    fn get_factory(&self) -> Arc<ApplicationFactory> {
        self.fac.clone()
    }

    fn get_collection_name(&self) -> &str {
        &self.collection_name
    }

    async fn init(&self) -> Result<()> {
        let col = self.get_collection()?;
        let index = IndexModel::builder()
            .keys(doc! {"email": 1})
            .options(IndexOptions::builder().unique(true).build())
            .build();

        col.create_index(index, None).await?;

        let index = IndexModel::builder().keys(doc! {"name": 1}).build();

        col.create_index(index, None).await?;

        Ok(())
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

    pub async fn login(&self, email: &str, passwords: &str) -> Result<DTO<User>> {
        let query = doc! {"email": email, "password": passwords};
        let result = self.find(query, 1, 2, None).await?;

        if result.len() == 0 {
            return Err(error!("User not found with email ({}) and password", email));
        }

        let res = result[0].clone();

        Ok(res)
    }
}
