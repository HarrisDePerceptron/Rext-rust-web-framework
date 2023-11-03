use anyhow::{anyhow as error, Result};

pub struct RedisProvider {
    pub connection: Option<redis::aio::ConnectionManager>,
    pub connection_uri: String,
}

impl RedisProvider {
    pub fn new(uri: &str) -> Self {
        Self {
            connection: None,
            connection_uri: String::from(uri),
        }
    }

    pub async fn connect(&mut self) -> Result<redis::aio::ConnectionManager> {
        if let Some(conn) = &self.connection {
            Ok(conn.clone())
        } else {
            let client = redis::Client::open(self.connection_uri.to_string())?;
            let manager = client.get_tokio_connection_manager().await?;
            self.connection = Some(manager.clone());
            Ok(manager)
        }
    }

    pub fn get_connection(&self) -> Result<redis::aio::ConnectionManager> {
        self.connection
            .clone()
            .ok_or(error!("Please connect first then get connection"))
    }

    pub fn get_sync_connection(&self) -> Result<redis::Connection> {
        let client = redis::Client::open(self.connection_uri.to_string())?;
        let conn = client.get_connection()?;
        Ok(conn)
    }
}
