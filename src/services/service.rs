use crate::services::dao::DaoObj;

use std::sync::Arc;

use crate::services::dto::DTO;
use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub struct Service<T> {
    dao: Arc<DaoObj<T>>,
}

impl<T> Service<T>
where
    T: Clone + Serialize + DeserializeOwned + 'static,
{
    pub fn new(dao: Arc<DaoObj<T>>) -> Self {
        Self { dao }
    }

    pub fn get_dao(&self) -> Arc<DaoObj<T>> {
        self.dao.clone()
    }

    pub async fn create(&self, data: T) -> Result<DTO<T>> {
        let data = DTO::new(data);
        let dao = self.get_dao();
        let result = dao.create(data).await?;

        Ok(result)
    }

    pub async fn get(&self, id: &str) -> Result<DTO<T>> {
        let dao = self.get_dao();
        let result = dao.get(id).await?;

        Ok(result)
    }

    pub async fn list(&self, page: u64, page_size: i64) -> Result<Vec<DTO<T>>> {
        let dao = self.get_dao();
        let result = dao.list(page, page_size).await?;

        Ok(result)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        let dao = self.get_dao();
        dao.delete(id).await?;

        Ok(())
    }

    pub async fn update(&self, data: DTO<T>) -> Result<DTO<T>> {
        let dao = self.get_dao();
        let result = dao.update(data).await?;
        Ok(result)
    }
}
