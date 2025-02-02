use crate::dao::privilege_store::PrivilegeStore;

pub struct PrivilegeService<T: PrivilegeStore> {
    store: T,
}

impl<T: PrivilegeStore> PrivilegeService<T> {
    pub fn new(store: T) -> Self {
        Self { store }
    }

    pub async fn check_privilege_status(
        &self,
        dir_name: &str,
        user_name: &str,
    ) -> Result<(), String> {
        let to_be_accessed = self.store.get_privilege_level(dir_name).await.expect(format!("The role {} does not exist", dir_name).as_str());
        let actual_privileges = self.store.get_privilege_level(user_name).await.expect(format!("The role {} does not exist", user_name).as_str());

        // Compare the route param to the user's token role
        if actual_privileges < to_be_accessed {
            // If they don't match, return 403
            return Err(format!(
                "Your token role is '{}', but you tried to access '{}'",
                user_name, dir_name
            ));
        }

        Ok(())
    }
}