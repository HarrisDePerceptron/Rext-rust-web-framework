use crate::application_factory::ApplicationFactory;
use std::sync::Arc;
use std::sync::OnceLock;

use anyhow::Result;

use crate::services::application_dao::ApplicationDao;

//use crate::services::user::user_dao::UserDao;
use crate::services::dao::DaoObj;
use crate::services::user::User;

use crate::services::collections::Collections;

pub struct ApplicationService {
    app_dao: Arc<ApplicationDao>,
    pub user: Arc<DaoObj<User>>,
}

impl ApplicationService {
    pub async fn new(
        app_dao: Arc<ApplicationDao>,
        app_fac: Arc<ApplicationFactory>,
    ) -> Result<Self> {
        Ok(Self {
            app_dao,
            user: Arc::new(DaoObj::new(
                &Collections::User.to_string(),
                app_fac.clone(),
            )?),
        })
    }
}

pub static APPLICATION_DAO: OnceLock<ApplicationDao> = OnceLock::new();
