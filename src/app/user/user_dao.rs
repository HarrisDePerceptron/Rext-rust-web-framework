use crate::app::dto::DTO;
use crate::app::user::user_model::User;

use crate::app::dao::DaoObj;
use anyhow::{anyhow as error, Result};
use mongodb::bson::doc;

pub struct UserRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

impl DaoObj<User> {
    pub async fn create_user(&mut self, req: UserRequest) -> Result<DTO<User>> {
        let user = User::new(&req.email, &req.password)?;

        let data = DTO::new(user);

        let result = self.create(data).await?;
        Ok(result)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<DTO<User>> {
        let col = self.get_collection()?;
        let user = col
            .find_one(doc! {"email": email}, None)
            .await?
            .ok_or(error!("User with email `{}` not found", email))?;

        Ok(user)
    }
}
