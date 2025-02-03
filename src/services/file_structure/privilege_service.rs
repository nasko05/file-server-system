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
        let to_be_accessed = self.store.get_privilege_level(dir_name).await;
        match to_be_accessed {  
            Ok(_) => {},
            Err(_) => return Err(format!("The role {} does not exist", dir_name).parse().unwrap())
        }
        let actual_privileges = self.store.get_privilege_level(user_name).await;
        match actual_privileges {
            Ok(_) => {},
            Err(_) => return Err(format!("The role {} does not exist", user_name).parse().unwrap())
        }

        // Compare the route param to the user's token role
        if actual_privileges < Ok(to_be_accessed?) {
            // If they don't match, return 403
            return Err(format!(
                "Your token role is '{}', but you tried to access '{}'",
                user_name, dir_name
            ));
        }

        Ok(())
    }
}