//use crate::application_factory::ApplicationFactory;
//use crate::auth;
//use crate::secrets;
//use anyhow::{anyhow as error, Result};
//use futures::{StreamExt, TryStream};
//use mongodb::bson::document;
//use mongodb::bson::{bson, Bson};
//use serde::{Deserialize, Serialize};
//use std::str::FromStr;
//use std::sync::Arc;
//
//use tokio::sync::Mutex;
//
//use uuid::Uuid;
//
//use async_trait::async_trait;
//
//use mongodb::bson::oid::ObjectId;
//use mongodb::bson::{doc, Document};
//use mongodb::options::IndexOptions;
//
//#[derive(Debug, Clone, Serialize, Deserialize)]
//pub struct User {
//    pub name: Option<String>,
//    pub email: String,
//    pub password: String,
//
//    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//    pub id: Option<ObjectId>,
//}
//
//#[derive(Debug, Clone, Serialize, Deserialize)]
//pub struct UserTokenPayload {
//    id: String,
//    email: String,
//}
//
//#[async_trait]
//pub trait UserPersist {
//    async fn create_user(&mut self, req: UserRequest) -> Result<User>;
//    async fn get_user(&self, id: &str) -> Result<User>;
//    async fn update_user(&self, user: &User) -> Result<User>;
//    async fn delete_user(&self, id: &str) -> Result<()>;
//    async fn list_users(&self, page: u64, page_size: i64) -> Result<Vec<User>>;
//    async fn find_by_email(&self, email: &str) -> Result<User>;
//}
//
//impl User {
//    pub fn new(email: &str, password: &str, name: Option<String>) -> Result<User> {
//        let mut name_s = name.map(|v| v.to_string());
//        if let Some(name) = name_s {
//            name_s = Some(name.to_lowercase());
//        }
//
//        if password.len() < 5 {
//            return Err(error!("Password length cannot be smaller than 5"));
//        }
//
//        if !(email.contains("@") && email.contains(".")) {
//            return Err(error!("Invalid email format"));
//        }
//
//        Ok(User {
//            name: name_s,
//            email: email.to_string(),
//            password: password.to_string(),
//            id: None,
//        })
//    }
//
//    pub fn create_token(&self) -> Result<String> {
//        let user_token_payload = UserTokenPayload {
//            email: self.email.to_string(),
//            id: self.id.ok_or(error!("id does not exists"))?.to_string(),
//        };
//
//        let payload_str = serde_json::to_string(&user_token_payload)?;
//        let expiry_days = secrets::TOKEN_EXPIRY_DAYS.to_string();
//
//        let expiry_days = expiry_days.parse::<u64>()?;
//
//        let sub = self.id.ok_or(error!("is not found on object"))?.to_string();
//
//        let token = auth::generate_token(&sub, &secrets::TOKEN_ISSUER.to_string(), expiry_days)
//            .map_err(|e| error!("{}", e))?;
//
//        Ok(token)
//    }
//}
//
//pub struct UserDao {
//    factory: Arc<ApplicationFactory>,
//    collection_name: String,
//    collection: mongodb::Collection<User>,
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
//            collection,
//        })
//    }
//
//    pub fn get_collection(&self) -> Result<mongodb::Collection<User>> {
//        let db = self.factory.mongo_provider.get_database()?;
//        let col = db.collection::<User>("User");
//        Ok(col)
//    }
//}
//
//pub struct UserRequest {
//    pub email: String,
//    pub password: String,
//    pub name: Option<String>,
//}
//
//#[async_trait]
//impl UserPersist for UserDao {
//    async fn create_user(&mut self, req: UserRequest) -> Result<User> {
//        let db = self.factory.mongo_provider.get_database()?;
//        let col = db.collection::<User>("User");
//
//        let user = User::new(&req.email, &req.password, req.name)?;
//
//        col.insert_one(user.clone(), None).await?;
//
//        Ok(user)
//    }
//
//    async fn get_user(&self, id: &str) -> Result<User> {
//        let col = self.get_collection()?;
//        let oid = ObjectId::from_str(id)?;
//        let res = col.find(doc! {"_id": oid}, None).await?;
//        let user = res.deserialize_current()?;
//        Ok(user)
//    }
//
//    async fn update_user(&self, user: &User) -> Result<User> {
//        let col = self.get_collection()?;
//        let id = user.id.ok_or(error!("Id on object not found for update"))?;
//        //let oid = ObjectId::from_str(&id)?;
//        let doc = mongodb::bson::to_bson(&user)?;
//        let doc = doc
//            .as_document()
//            .ok_or(error!("Unable to convert bson to document"))?;
//        col.update_one(doc! {"_id": id}, doc! {"$set": doc}, None)
//            .await?;
//
//        Ok(user.clone())
//    }
//
//    async fn delete_user(&self, id: &str) -> Result<()> {
//        let col = self.get_collection()?;
//        let oid = ObjectId::from_str(id)?;
//        col.delete_one(doc! {"_id": oid }, None).await?;
//
//        Ok(())
//    }
//    async fn list_users(&self, page: u64, page_size: i64) -> Result<Vec<User>> {
//        let col = self.get_collection()?;
//        let mut opt = mongodb::options::FindOptions::default();
//
//        let start = (page as i64 - 1) * page_size;
//        opt.limit = Some(page_size);
//        opt.skip = Some(start as u64);
//
//        let mut result = col.find(doc! {}, opt).await?;
//        let mut data: Vec<User> = Vec::new();
//        while let Some(u) = result.next().await {
//            if let Ok(uu) = u {
//                data.push(uu);
//            }
//        }
//        Ok(data)
//    }
//
//    async fn find_by_email(&self, email: &str) -> Result<User> {
//        let col = self.get_collection()?;
//        let user = col
//            .find_one(doc! {"email": email}, None)
//            .await?
//            .ok_or(error!("User with email `{}` not found", email))?;
//
//        Ok(user)
//    }
//}
