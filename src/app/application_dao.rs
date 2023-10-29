use crate::application_factory::ApplicationFactory;
use std::sync::Arc;
use std::sync::OnceLock;

use anyhow::Result;

use crate::app::user::user_dao::UserDao;

use super::DaoObj;

pub struct ApplicationDao {
    pub user: Arc<UserDao>,
}

impl ApplicationDao {
    pub async fn new(fac: Arc<ApplicationFactory>) -> Result<Self> {
        let user = UserDao::new(fac.clone())?;
        user.init().await?;

        Ok(Self {
            user: Arc::new(user),
        })
    }
}

pub static APPLICATION_DAO: OnceLock<Arc<ApplicationDao>> = OnceLock::new();
