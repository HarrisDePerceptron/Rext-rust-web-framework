use anyhow::Result;
use mongodb::{options::ClientOptions, Client};

use anyhow::anyhow;
use log;

#[derive(Debug, Clone)]
pub struct MongoProvider {
    pub client: Option<mongodb::Client>,
    pub mongo_uri: String,
    pub mongo_database: String,
}

impl MongoProvider {
    pub fn new(uri: &str, database: &str) -> Self {
        Self {
            mongo_database: database.to_string(),
            client: None,
            mongo_uri: uri.to_string(),
        }
    }

    pub async fn connect(&mut self) -> Result<mongodb::Client> {
        if let Some(c) = &self.client {
            return Ok(c.clone());
        }

        log::info!("Connecting to mongo: {}", self.mongo_uri);

        let client_options = ClientOptions::parse(&self.mongo_uri).await?;

        let client = Client::with_options(client_options)?;

        self.client = Some(client.clone());
        log::info!("Mongo connected at {}", self.mongo_uri);
        Ok(client)
    }

    pub fn get_client(&self) -> Result<mongodb::Client> {
        if let Some(c) = &self.client {
            Ok(c.clone())
        } else {
            Err(anyhow!("Client was not found. Please connect first"))
        }
    }

    pub fn get_database(&self) -> Result<mongodb::Database> {
        let client = self.get_client()?;

        let db = client.database(&self.mongo_database);
        Ok(db)
    }
}
