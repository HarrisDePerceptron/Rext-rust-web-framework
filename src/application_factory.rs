use crate::mongo_persistence::MongoProvider;
use crate::secrets;

use anyhow::Result;

pub struct ApplicationFactory {
    pub mongo_provider: MongoProvider,
}

impl ApplicationFactory {
    pub fn new() -> Self {
        let mongo_uri = secrets::MONGO_URI.to_string();
        let mongo_database = secrets::MONGO_DATABASE.to_string();

        let mongo_provider = MongoProvider::new(&mongo_uri, &mongo_database);

        Self { mongo_provider }
    }
}
