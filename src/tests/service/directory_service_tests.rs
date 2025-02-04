#[cfg(test)]
mod tests {
    use std::fs::{File, create_dir};
    use std::path::{PathBuf};
    use async_trait::async_trait;
    use tokio;
    use mockall::predicate::*;
    use mockall::mock;
    use crate::dao::privilege_store::PrivilegeStore;
    use crate::services::file_structure::directory_service::DirectoryService;
    use crate::services::file_structure::privilege_service::PrivilegeService;
    use crate::tests::test_structure::get_global_test_env;

    // Create mock implementation
    mock! {
        pub PrivilegeStoreMock {}
        
        #[async_trait]
        impl PrivilegeStore for PrivilegeStoreMock {
            async fn get_privilege_level(&self, role: &str) -> Result<i32, String>;
        }
    }

    #[tokio::test]
    async fn test_build_dir_tree() {
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let user = &env.username;
        let directory_service = DirectoryService::new(root.clone());

        let tree = directory_service.build_dir_tree(
            &env.root_dir.path()
                .join(user)
                .join("test_dir")
        ).unwrap();

        assert_eq!(tree.name, "test_dir");
        assert_eq!(tree.files, vec!["file1.txt", "file2.rs"]);
        assert_eq!(tree.dirs.len(), 1);
        assert_eq!(tree.dirs[0].name, "sub_dir");
        assert_eq!(tree.dirs[0].files, vec!["sub_file.txt"]);
    }

    #[tokio::test]
    async fn test_check_privilege_status() {
        let mut mock_store = MockPrivilegeStoreMock::new();

        // Set up expectation for the call with "admin"
        mock_store.expect_get_privilege_level()
            .with(eq("admin"))
            .returning(|_| Ok(999));

        // Set up expectation for the call with "user"
        mock_store.expect_get_privilege_level()
            .with(eq("user"))
            .returning(|_| Ok(111));

        let privilege_service = PrivilegeService::new(mock_store);
        // Test equal privileges
        assert!(privilege_service.check_privilege_status("user", "user").await.is_ok());

        // Test higher privileges (assuming privilege levels: admin > user > guest)
        assert!(privilege_service.check_privilege_status("user", "admin").await.is_ok());

        // Test lower privileges
        let result = privilege_service.check_privilege_status("admin", "user").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Your token role is 'user'"));
    }

    #[tokio::test]
    async fn test_to_full_path() {
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let directory_service = DirectoryService::new(root.clone());
        let test_file = &env.root_dir.path().join("test.txt");
        File::create(&test_file).unwrap();

        // Valid path
        let result = directory_service.to_full_path(test_file.clone());
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test.txt"));

        // Invalid path
        let result = directory_service.to_full_path(PathBuf::from("/non/existent/path"));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_if_directory_exists() {let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let directory_service = DirectoryService::new(root.clone());

        // Test existing directory
        let result = directory_service.check_if_directory_exists("test_dir", &env.username, "").await;
        assert_eq!(result.unwrap(), "dir");

        // Test existing file
        let result = directory_service.check_if_directory_exists("", &env.username, "test_file.txt").await;
        assert_eq!(result.unwrap(), "file");

        // Test non-existent path
        let result = directory_service.check_if_directory_exists("invalid", &env.username, "invalid").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_directory_tree() {
        let env = get_global_test_env().await;
        let empty_dir = env.root_dir.path().join("empty_dir");
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let directory_service = DirectoryService::new(root.clone());
        create_dir(&empty_dir).unwrap();

        let tree = directory_service.build_dir_tree(&empty_dir).unwrap();

        assert_eq!(tree.name, "empty_dir");
        assert!(tree.files.is_empty());
        assert!(tree.dirs.is_empty());
    }

    #[tokio::test]
    async fn test_privilege_edge_cases() {
        let mut mock_store = MockPrivilegeStoreMock::new();
        mock_store.expect_get_privilege_level()
            .with(eq("user"))
            .returning(|_| Ok(111));

        mock_store.expect_get_privilege_level()
            .with(eq(""))
            .returning(|_| Err("".parse().unwrap()));

        mock_store.expect_get_privilege_level()
            .with(eq("nonexistent"))
            .returning(|_| Err("e".parse().unwrap()));

        let privilege_service = PrivilegeService::new(mock_store);
        // Test non-existent role
        let result = privilege_service.check_privilege_status("nonexistent", "user").await;
        assert!(result.is_err());

        // Test empty strings
        let result = privilege_service.check_privilege_status("", "").await;
        assert!(result.is_err());
    }
}