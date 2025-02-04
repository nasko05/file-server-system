use std::sync::Arc;

#[derive(Clone)]
pub struct AppConfig {
    pub root_dir: Arc<String>,
}