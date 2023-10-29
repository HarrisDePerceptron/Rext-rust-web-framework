use crate::application_factory::ApplicationFactory;
use std::{convert::From, fmt::Debug};

use anyhow::{anyhow as error, Result};
use futures::StreamExt;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::FindOptions;

use std::sync::Arc;

use serde::{de::DeserializeOwned, Serialize};
use std::str::FromStr;

use crate::app::dto::DTO;
use async_trait::async_trait;

//pub struct DaoObj<T> {
//    factory: Option<Arc<ApplicationFactory>>,
//
//    collection_name: String,
//    collection: mongodb::Collection<DTO<T>>,
//}

#[async_trait]
pub trait DaoObj<T>: Send + Sync
where
    T: Clone + Serialize + DeserializeOwned + 'static + std::marker::Send + std::marker::Sync,
{
    fn get_factory(&self) -> Arc<ApplicationFactory>;

    fn get_collection_name(&self) -> &str;

    async fn init(&self) -> Result<()> {
        Ok(())
    }

    fn get_collection(&self) -> Result<mongodb::Collection<DTO<T>>> {
        let col = self
            .get_factory()
            .mongo_provider
            .get_database()?
            .collection::<DTO<T>>(self.get_collection_name());

        Ok(col)
    }

    async fn create(&self, mut data: DTO<T>) -> Result<DTO<T>> {
        let col = self.get_collection()?;
        //let item = Output::from(self);

        //col.insert_one(item.clone(), None).await?;

        let res = col.insert_one(data.clone(), None).await?;
        let id = res
            .inserted_id
            .as_object_id()
            .ok_or(error!("Unable to convert to object id"))?
            .to_string();
        data.id = Some(id);

        Ok(data)
    }
    async fn get(&self, id: &str) -> Result<DTO<T>> {
        let col = self.get_collection()?;
        let oid = ObjectId::from_str(id)?;
        let mut res = col.find(doc! {"_id": oid}, None).await?;
        let result = res
            .next()
            .await
            .ok_or(error!("Could not find item with id `{}`", id))??;

        Ok(result)
    }

    async fn update(&self, data: DTO<T>) -> Result<DTO<T>> {
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

    async fn list(&self, page: u64, page_size: i64) -> Result<Vec<DTO<T>>> {
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

    async fn find(
        &self,
        query: Document,
        page: u64,
        page_size: i64,
        options: Option<FindOptions>,
    ) -> Result<Vec<DTO<T>>> {
        let col = self.get_collection()?;

        let mut opt = options.unwrap_or(FindOptions::default());

        opt.sort = Some(doc! {"created_at": -1});
        let start = (page as i64 - 1) * page_size;
        opt.limit = Some(page_size);
        opt.skip = Some(start as u64);

        let mut result = col.find(query, opt).await?;
        let mut data: Vec<DTO<T>> = Vec::new();
        while let Some(u) = result.next().await {
            if let Ok(uu) = u {
                data.push(uu);
            }
        }
        Ok(data)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let col = self.get_collection()?;
        let oid = ObjectId::from_str(id)?;
        col.delete_one(doc! {"_id": oid}, None).await?;

        Ok(())
    }
}
