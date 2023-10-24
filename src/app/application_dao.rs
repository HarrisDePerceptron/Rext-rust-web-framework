use crate::application_factory::ApplicationFactory;
use std::sync::Arc;
use std::sync::OnceLock;

use anyhow::Result;

use crate::app::user::user_dao::UserDao;

pub struct ApplicationDao {
    pub user: Arc<UserDao>,
}

impl ApplicationDao {
    pub fn new(fac: Arc<ApplicationFactory>) -> Result<Self> {
        Ok(Self {
            user: Arc::new(UserDao::new(fac.clone())?),
        })
    }
}

pub static APPLICATION_DAO: OnceLock<Arc<ApplicationDao>> = OnceLock::new();
