use crate::app::service::Service;
use crate::app::user::User;

use crate::app::dto::DTO;

use anyhow::{anyhow as error, Result};

impl Service<User> {
    pub async fn find_by_email(&self, email: &str) -> Result<DTO<User>> {
        let dao = self.get_dao();
        let result = dao.find_by_email(email).await?;

        Ok(result)
    }
}
