use crate::mongo_persistence::MongoProvider;
use crate::secrets;

use anyhow::Result;

pub struct ApplicationFactory {
    pub mongo_provider: MongoProvider,
}

impl ApplicationFactory {
    pub async fn new() -> Result<Self> {
        let mongo_uri = secrets::MONGO_URI.to_string();
        let mongo_database = secrets::MONGO_DATABASE.to_string();

        let mut mongo_provider = MongoProvider::new(&mongo_uri, &mongo_database);

        mongo_provider.connect().await?;

        Ok(Self { mongo_provider })
    }
}
