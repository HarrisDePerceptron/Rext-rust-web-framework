use std::sync::Arc;
use std::sync::OnceLock;

use anyhow::Result;

use crate::app::application_dao::ApplicationDao;

use crate::app::user::User;

use crate::app::service::Service;

pub struct ApplicationService {
    app_dao: Arc<ApplicationDao>,
    pub user: Arc<Service<User>>,
}

impl ApplicationService {
    pub fn new(app_dao: Arc<ApplicationDao>) -> Result<Self> {
        Ok(Self {
            app_dao: app_dao.clone(),
            user: Arc::new(Service::new(app_dao.user.clone())),
        })
    }
}

pub static APPLICATION_SERVICE: OnceLock<Arc<ApplicationService>> = OnceLock::new();
