use crate::auth;
use crate::secrets;
use anyhow::{anyhow as error, Result};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::services::dao::DaoOutput;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTokenPayload {
    id: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: Option<String>,
    pub email: String,
    pub password: String,

    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
}
impl DaoOutput for User {
    fn id(&self) -> Option<ObjectId> {
        self.id
    }
}
impl User {
    pub fn new(email: &str, password: &str, name: Option<String>) -> Result<User> {
        let mut name_s = name.map(|v| v.to_string());
        if let Some(name) = name_s {
            name_s = Some(name.to_lowercase());
        }

        if password.len() < 5 {
            return Err(error!("Password length cannot be smaller than 5"));
        }

        if !(email.contains("@") && email.contains(".")) {
            return Err(error!("Invalid email format"));
        }

        Ok(User {
            name: name_s,
            email: email.to_string(),
            password: password.to_string(),
            id: None,
        })
    }

    pub fn create_token(&self) -> Result<String> {
        let user_token_payload = UserTokenPayload {
            email: self.email.to_string(),
            id: self.id.ok_or(error!("id does not exists"))?.to_string(),
        };

        let payload_str = serde_json::to_string(&user_token_payload)?;
        let expiry_days = secrets::TOKEN_EXPIRY_DAYS.to_string();

        let expiry_days = expiry_days.parse::<u64>()?;

        let sub = self.id.ok_or(error!("is not found on object"))?.to_string();

        let token = auth::generate_token(&sub, &secrets::TOKEN_ISSUER.to_string(), expiry_days)
            .map_err(|e| error!("{}", e))?;

        Ok(token)
    }
}
