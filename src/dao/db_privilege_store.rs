use async_trait::async_trait;
use crate::dao::login_verification::{check_privileges};
use crate::dao::privilege_store::PrivilegeStore;

pub struct DbPrivilegeStore;

#[async_trait]
impl PrivilegeStore for DbPrivilegeStore {
    async fn get_privilege_level(&self, role: &str) -> Result<i32, String> {
        check_privileges(role).await
    }
}