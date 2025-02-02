use async_trait::async_trait;

#[async_trait]
pub trait PrivilegeStore: Send + Sync {
    async fn get_privilege_level(&self, role: &str) -> Result<i32, String>;
}