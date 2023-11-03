use crate::persistence::mongo_persistence::MongoProvider;
use crate::secrets;

use crate::persistence::redis_provider::RedisProvider;

use anyhow::Result;

pub struct ApplicationFactory {
    pub mongo_provider: MongoProvider,
    pub redis_provider: RedisProvider,
}

impl ApplicationFactory {
    pub async fn new() -> Result<Self> {
        let mongo_uri = secrets::MONGO_URI.to_string();
        let mongo_database = secrets::MONGO_DATABASE.to_string();

        let mut mongo_provider = MongoProvider::new(&mongo_uri, &mongo_database);

        mongo_provider.connect().await?;
        log::info!("Mongo Connected!!");

        let redis_uri = secrets::REDIS_URI.to_string();
        let mut redis_provider = RedisProvider::new(redis_uri.as_str());
        redis_provider.connect().await?;

        log::info!("Redis connected!!");

        Ok(Self {
            mongo_provider,
            redis_provider,
        })
    }
}
