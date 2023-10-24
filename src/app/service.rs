use crate::app::dao::DaoObj;

use std::sync::Arc;

use crate::app::dto::DTO;
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

use async_trait::async_trait;

#[async_trait]
pub trait Service<T>
where
    T: Clone + Serialize + DeserializeOwned + 'static + std::marker::Send + std::marker::Sync,
{
    fn get_dao(&self) -> Arc<dyn DaoObj<T>>;

    async fn create(&self, data: T) -> Result<DTO<T>> {
        let data = DTO::new(data);
        let dao = self.get_dao();
        let result = dao.create(data).await?;

        Ok(result)
    }

    async fn get(&self, id: &str) -> Result<DTO<T>> {
        let dao = self.get_dao();
        let result = dao.get(id).await?;

        Ok(result)
    }

    async fn list(&self, page: u64, page_size: i64) -> Result<Vec<DTO<T>>> {
        let dao = self.get_dao();
        let result = dao.list(page, page_size).await?;

        Ok(result)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let dao = self.get_dao();
        dao.delete(id).await?;

        Ok(())
    }

    async fn update(&self, data: DTO<T>) -> Result<DTO<T>> {
        let dao = self.get_dao();
        let result = dao.update(data).await?;
        Ok(result)
    }
}
