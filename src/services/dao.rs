use crate::application_factory::ApplicationFactory;
use std::convert::From;

use anyhow::{anyhow as error, Result};
use async_trait::async_trait;
use futures::{StreamExt, TryStream};
use mongodb::bson::{doc, oid::ObjectId};
use std::sync::Arc;

use serde::Serialize;
use std::str::FromStr;

pub trait DaoOutput {
    fn id(&self) -> Option<ObjectId>;
}

#[async_trait]
pub trait Dao<Input, Output>
where
    Input: Serialize + Send + 'static + Clone,
    Output: Serialize
        + From<Input>
        + Send
        + 'static
        + Sync
        + Clone
        + serde::de::DeserializeOwned
        + DaoOutput,
{
    fn get_factory(&self) -> Arc<ApplicationFactory>;
    fn get_collection_name(&self) -> &str;

    fn get_collection(&self) -> Result<mongodb::Collection<Output>> {
        let db = self.get_factory().mongo_provider.get_database()?;
        let col = db.collection::<Output>(self.get_collection_name());
        Ok(col)
    }

    async fn create(&mut self, req: Input) -> Result<Output> {
        let col = self.get_collection()?;
        let item = Output::from(req);

        col.insert_one(item.clone(), None).await?;

        Ok(item)
    }
    async fn get(&self, id: &str) -> Result<Output> {
        let col = self.get_collection()?;
        let oid = ObjectId::from_str(id)?;
        let res = col.find(doc! {"_id": oid}, None).await?;
        let output = res.deserialize_current()?;
        Ok(output)
    }

    async fn update(&self, o: Output) -> Result<Output> {
        let col = self.get_collection()?;
        let id = o.id().ok_or(error!("Id on object not found for update"))?;
        let doc = mongodb::bson::to_bson(&o)?;
        let doc = doc
            .as_document()
            .ok_or(error!("Unable to convert bson to document"))?;
        col.update_one(doc! {"_id": id}, doc! {"$set": doc}, None)
            .await?;

        Ok(o.clone())
    }

    async fn list(&self, page: u64, page_size: i64) -> Result<Vec<Output>> {
        let col = self.get_collection()?;
        let mut opt = mongodb::options::FindOptions::default();

        let start = (page as i64 - 1) * page_size;
        opt.limit = Some(page_size);
        opt.skip = Some(start as u64);

        let mut result = col.find(doc! {}, opt).await?;
        let mut data: Vec<Output> = Vec::new();
        while let Some(u) = result.next().await {
            if let Ok(uu) = u {
                data.push(uu);
            }
        }
        Ok(data)
    }
}
