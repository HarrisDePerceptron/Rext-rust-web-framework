use std::sync::Arc;
use std::sync::OnceLock;

use anyhow::Result;

use crate::app::application_dao::ApplicationDao;

use super::user::user_service::UserService;

pub struct ApplicationService {
    pub user: Arc<UserService>,
}

impl ApplicationService {
    pub fn new(app_dao: Arc<ApplicationDao>) -> Result<Self> {
        Ok(Self {
            user: Arc::new(UserService::new(app_dao.user.clone())),
        })
    }
}

pub static APPLICATION_SERVICE: OnceLock<Arc<ApplicationService>> = OnceLock::new();
