use crate::app::collections::Collections;
use crate::app::dao::DaoObj;
use crate::application_factory::ApplicationFactory;
use std::sync::Arc;
use std::sync::OnceLock;

use crate::app::user::user_model::User;
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

pub static APPLICATION_DAO: OnceLock<Arc<ApplicationDao>> = OnceLock::new();
