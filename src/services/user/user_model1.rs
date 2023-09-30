use mongodb::bson::oid::ObjectId;

use crate::services::dao::DaoOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User1 {
    pub email: String,
    pub id: Option<ObjectId>,
}

impl DaoOutput for User1 {
    fn id(&self) -> Option<mongodb::bson::oid::ObjectId> {
        self.id
    }
}
