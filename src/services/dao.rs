use crate::application_factory::ApplicationFactory;
use std::{convert::From, fmt::Debug};

use anyhow::{anyhow as error, Result};
use async_trait::async_trait;
use futures::{StreamExt, TryStream};
use mongodb::bson::{doc, oid::ObjectId};
use std::sync::Arc;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::str::FromStr;

use crate::services::dto::DTO;
use std::ops::{Deref, DerefMut};

pub struct DaoObj<T> {
    factory: Option<Arc<ApplicationFactory>>,

    collection_name: String,
    collection: mongodb::Collection<DTO<T>>,
}

impl<T> DaoObj<T>
where
    T: Clone + Serialize + DeserializeOwned + 'static,
{
    pub fn new(collection_name: &str, factory: Arc<ApplicationFactory>) -> Result<Self> {
        let col = factory
            .mongo_provider
            .get_database()?
            .collection::<DTO<T>>(collection_name);
        Ok(Self {
            factory: Some(factory),
            collection_name: collection_name.to_string(),
            collection: col,
        })
    }

    fn get_factory(&self) -> Arc<ApplicationFactory> {
        self.factory
            .clone()
            .expect("Factory should have been initialzed form new")
            .clone()
    }
    fn get_collection_name(&self) -> &str {
        &self.collection_name
    }

    pub fn get_collection(&self) -> Result<mongodb::Collection<DTO<T>>> {
        Ok(self.collection.clone())
    }

    pub async fn create(&self, data: DTO<T>) -> Result<DTO<T>> {
        let col = self.get_collection()?;
        //let item = Output::from(self);

        //col.insert_one(item.clone(), None).await?;

        col.insert_one(data.clone(), None).await?;
        Ok(data)
    }
    pub async fn get(&self, id: &str) -> Result<DTO<T>> {
        let col = self.get_collection()?;
        let oid = ObjectId::from_str(id)?;
        let res = col.find(doc! {"_id": oid}, None).await?;
        let output = res.deserialize_current()?;
        Ok(output)
    }

    pub async fn update(&self, data: DTO<T>) -> Result<DTO<T>> {
        let col = self.get_collection()?;
        let id = data
            .id
            .clone()
            .ok_or(error!("Id on object not found for update"))?;
        let doc = mongodb::bson::to_bson(&data)?;
        let doc = doc
            .as_document()
            .ok_or(error!("Unable to convert bson to document"))?;
        col.update_one(doc! {"_id": id}, doc! {"$set": doc}, None)
            .await?;

        Ok(data)
    }

    pub async fn list(&self, page: u64, page_size: i64) -> Result<Vec<DTO<T>>> {
        let col = self.get_collection()?;
        let mut opt = mongodb::options::FindOptions::default();

        let start = (page as i64 - 1) * page_size;
        opt.limit = Some(page_size);
        opt.skip = Some(start as u64);

        let mut result = col.find(doc! {}, opt).await?;
        let mut data: Vec<DTO<T>> = Vec::new();
        while let Some(u) = result.next().await {
            if let Ok(uu) = u {
                data.push(uu);
            }
        }
        Ok(data)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        let col = self.get_collection()?;
        let oid = ObjectId::from_str(id)?;
        col.delete_one(doc! {"_id": oid}, None).await?;

        Ok(())
    }
}
