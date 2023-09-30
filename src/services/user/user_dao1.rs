use crate::application_factory::ApplicationFactory;
use crate::services::dao;
use crate::services::user::user_model1::User1;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

pub struct User1Dao {
    factory: Arc<ApplicationFactory>,
}

impl User1Dao {
    pub fn new(factory: Arc<ApplicationFactory>) -> Self {
        Self { factory }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserRequest {
    email: String,
}

impl From<UserRequest> for User1 {
    fn from(value: UserRequest) -> Self {
        Self {
            id: None,
            email: value.email,
        }
    }
}

impl dao::Dao<UserRequest, User1> for User1Dao {
    fn get_factory(&self) -> Arc<ApplicationFactory> {
        self.factory.clone()
    }

    fn get_collection_name(&self) -> &str {
        "User"
    }
}
