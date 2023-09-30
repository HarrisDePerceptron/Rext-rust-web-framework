use crate::application_factory::ApplicationFactory;
use crate::services::user::user_model::User;

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

pub struct UserDao {
    factory: Arc<ApplicationFactory>,
    collection_name: String,
}

//use std::convert::From;
//
//trait DaoOutput {
//    fn id(&self) -> Option<ObjectId>;
//}
//
//#[async_trait]
//trait Dao<Input, Output>
//where
//    Input: Serialize + Send + 'static + Clone,
//    Output: Serialize
//        + From<Input>
//        + Send
//        + 'static
//        + Sync
//        + Clone
//        + serde::de::DeserializeOwned
//        + DaoOutput,
//{
//    fn get_factory(&self) -> Arc<ApplicationFactory>;
//    fn get_collection_name(&self) -> &str;
//
//    fn get_collection(&self) -> Result<mongodb::Collection<Output>> {
//        let db = self.get_factory().mongo_provider.get_database()?;
//        let col = db.collection::<Output>(self.get_collection_name());
//        Ok(col)
//    }
//
//    async fn create(&mut self, req: Input) -> Result<Output> {
//        let col = self.get_collection()?;
//        let item = Output::from(req);
//
//        col.insert_one(item.clone(), None).await?;
//
//        Ok(item)
//    }
//    async fn get(&self, id: &str) -> Result<Output> {
//        let col = self.get_collection()?;
//        let oid = ObjectId::from_str(id)?;
//        let res = col.find(doc! {"_id": oid}, None).await?;
//        let output = res.deserialize_current()?;
//        Ok(output)
//    }
//
//    async fn update(&self, o: Output) -> Result<Output> {
//        let col = self.get_collection()?;
//        let id = o.id().ok_or(error!("Id on object not found for update"))?;
//        let doc = mongodb::bson::to_bson(&o)?;
//        let doc = doc
//            .as_document()
//            .ok_or(error!("Unable to convert bson to document"))?;
//        col.update_one(doc! {"_id": id}, doc! {"$set": doc}, None)
//            .await?;
//
//        Ok(o.clone())
//    }
//
//    async fn list(&self, page: u64, page_size: i64) -> Result<Vec<Output>> {
//        let col = self.get_collection()?;
//        let mut opt = mongodb::options::FindOptions::default();
//
//        let start = (page as i64 - 1) * page_size;
//        opt.limit = Some(page_size);
//        opt.skip = Some(start as u64);
//
//        let mut result = col.find(doc! {}, opt).await?;
//        let mut data: Vec<Output> = Vec::new();
//        while let Some(u) = result.next().await {
//            if let Ok(uu) = u {
//                data.push(uu);
//            }
//        }
//        Ok(data)
//    }
//}
//
impl UserDao {
    pub async fn new(factory: Arc<ApplicationFactory>) -> Result<Self> {
        let collection_name = "User".to_string();

        let db = factory.mongo_provider.get_database()?;
        let collection = db.collection::<User>(&collection_name);
        let mut index = mongodb::IndexModel::default();

        index.keys = doc! {"email": 1};
        let mut index_opts = IndexOptions::default();
        index_opts.unique = Some(true);

        index.options = Some(index_opts);

        match collection.create_index(index, None).await {
            Ok(_) => log::info!("Created index user"),
            Err(e) => log::error!("Error creating index: {}", e.to_string()),
        };

        Ok(Self {
            factory,
            collection_name,
        })
    }

    pub fn get_collection(&self) -> Result<mongodb::Collection<User>> {
        let db = self.factory.mongo_provider.get_database()?;
        let col = db.collection::<User>("User");
        Ok(col)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let col = self.get_collection()?;
        let oid = ObjectId::from_str(id)?;
        col.delete_one(doc! {"_id": oid }, None).await?;

        Ok(())
    }
}

#[async_trait]
impl UserPersist for UserDao {
    async fn create_user(&mut self, req: UserRequest) -> Result<User> {
        let db = self.factory.mongo_provider.get_database()?;
        let col = db.collection::<User>("User");

        let user = User::new(&req.email, &req.password, req.name)?;

        col.insert_one(user.clone(), None).await?;

        Ok(user)
    }

    async fn get_user(&self, id: &str) -> Result<User> {
        let col = self.get_collection()?;
        let oid = ObjectId::from_str(id)?;
        let res = col.find(doc! {"_id": oid}, None).await?;
        let user = res.deserialize_current()?;
        Ok(user)
    }

    async fn update_user(&self, user: &User) -> Result<User> {
        let col = self.get_collection()?;
        let id = user.id.ok_or(error!("Id on object not found for update"))?;
        //let oid = ObjectId::from_str(&id)?;
        let doc = mongodb::bson::to_bson(&user)?;
        let doc = doc
            .as_document()
            .ok_or(error!("Unable to convert bson to document"))?;
        col.update_one(doc! {"_id": id}, doc! {"$set": doc}, None)
            .await?;

        Ok(user.clone())
    }

    async fn delete_user(&self, id: &str) -> Result<()> {
        let col = self.get_collection()?;
        let oid = ObjectId::from_str(id)?;
        col.delete_one(doc! {"_id": oid }, None).await?;

        Ok(())
    }
    async fn list_users(&self, page: u64, page_size: i64) -> Result<Vec<User>> {
        let col = self.get_collection()?;
        let mut opt = mongodb::options::FindOptions::default();

        let start = (page as i64 - 1) * page_size;
        opt.limit = Some(page_size);
        opt.skip = Some(start as u64);

        let mut result = col.find(doc! {}, opt).await?;
        let mut data: Vec<User> = Vec::new();
        while let Some(u) = result.next().await {
            if let Ok(uu) = u {
                data.push(uu);
            }
        }
        Ok(data)
    }

    async fn find_by_email(&self, email: &str) -> Result<User> {
        let col = self.get_collection()?;
        let user = col
            .find_one(doc! {"email": email}, None)
            .await?
            .ok_or(error!("User with email `{}` not found", email))?;

        Ok(user)
    }
}
