use crate::mongo_persistence::MongoProvider;
use crate::secrets;

use anyhow::Result;

pub struct ApplicationFactory {
    pub client: mongodb::Client,
    pub mongo_uri: String,
    pub mongo_database: String,
}

impl ApplicationFactory {
    pub async fn new() -> Self {
        let mongo_uri = secrets::MONGO_URI.to_string();
        let mongo_database = secrets::MONGO_DATABASE.to_string();

        let mongo_provider = MongoProvider::new(&mongo_uri);
        let client = mongo_provider
            .connect()
            .await
            .expect(&format!("Unable to connect to {}", mongo_uri));

        Self {
            mongo_database,
            client,
            mongo_uri,
        }
    }

    pub fn get_client(&self) -> mongodb::Client {
        self.client.clone()
    }

    pub fn get_database(&self) -> mongodb::Database {
        let db = self.client.database(&self.mongo_database);
        db
    }
}
