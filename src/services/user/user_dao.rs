use crate::services::user::user_model::User;
use crate::{application_factory::ApplicationFactory, services::dto::DTO};

use anyhow::{anyhow as error, Result};
use async_trait::async_trait;
use futures::{StreamExt, TryStream};
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
};
use std::sync::Arc;

use serde::Serialize;
use std::str::FromStr;

use crate::services::dao::DaoObj;

pub struct UserRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[async_trait]
pub trait UserPersist {
    async fn create_user(&mut self, req: UserRequest) -> Result<User>;
    async fn get_user(&self, id: &str) -> Result<User>;
    async fn update_user(&self, user: &User) -> Result<User>;
    async fn delete_user(&self, id: &str) -> Result<()>;
    async fn list_users(&self, page: u64, page_size: i64) -> Result<Vec<User>>;
    async fn find_by_email(&self, email: &str) -> Result<User>;
}

//pub struct UserDao {
//    factory: Arc<ApplicationFactory>,
//    collection_name: String,
//}
//
//impl UserDao {
//    pub async fn new(factory: Arc<ApplicationFactory>) -> Result<Self> {
//        let collection_name = "User".to_string();
//
//        let db = factory.mongo_provider.get_database()?;
//        let collection = db.collection::<User>(&collection_name);
//        let mut index = mongodb::IndexModel::default();
//
//        index.keys = doc! {"email": 1};
//        let mut index_opts = IndexOptions::default();
//        index_opts.unique = Some(true);
//
//        index.options = Some(index_opts);
//
//        match collection.create_index(index, None).await {
//            Ok(_) => log::info!("Created index user"),
//            Err(e) => log::error!("Error creating index: {}", e.to_string()),
//        };
//
//        Ok(Self {
//            factory,
//            collection_name,
//        })
//    }
//
//    pub fn get_collection(&self) -> Result<mongodb::Collection<User>> {
//        let db = self.factory.mongo_provider.get_database()?;
//        let col = db.collection::<User>("User");
//        Ok(col)
//    }
//
//    async fn delete(&self, id: &str) -> Result<()> {
//        let col = self.get_collection()?;
//        let oid = ObjectId::from_str(id)?;
//        col.delete_one(doc! {"_id": oid }, None).await?;
//
//        Ok(())
//    }
//}
//

impl DaoObj<User> {
    pub async fn create_user(&mut self, req: UserRequest) -> Result<DTO<User>> {
        let user = User::new(&req.email, &req.password)?;

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
