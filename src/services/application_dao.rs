use crate::application_factory::ApplicationFactory;
use crate::services::collections::Collections;
use crate::services::dao::DaoObj;
use std::sync::Arc;
use std::sync::OnceLock;

use crate::services::user::user_model::User;
use anyhow::Result;

pub struct ApplicationDao {
    pub user: Arc<DaoObj<User>>,
}

impl ApplicationDao {
    pub fn new(fac: Arc<ApplicationFactory>) -> Result<Self> {
        Ok(Self {
            user: Arc::new(DaoObj::new(
                Collections::User.to_string().as_str(),
                fac.clone(),
            )?),
        })
    }
}

pub static APPLICATION_DAO: OnceLock<ApplicationDao> = OnceLock::new();
