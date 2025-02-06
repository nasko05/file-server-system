use std::sync::Arc;
use crate::services::locking::directory_locking_manager::DirectoryLockManager;

#[derive(Clone)]
pub struct AppConfig {
    pub root_dir: Arc<String>,
    pub directory_lock_manager: DirectoryLockManager
}