#[cfg(test)]
mod tests {
    use tempfile::{tempdir, TempDir};
    use std::fs::{File, create_dir};
    use std::io;
    use std::path::{Path, PathBuf};
    use async_trait::async_trait;
    use tokio;
    use mockall::predicate::*;
    use mockall::mock;
    use tokio::sync::OnceCell;
    use crate::dao::privilege_store::PrivilegeStore;
    use crate::services::file_structure::directory_service::DirectoryService;
    use crate::services::file_structure::privilege_service::PrivilegeService;

    // Create mock implementation
    mock! {
        pub PrivilegeStoreMock {}
        
        #[async_trait]
        impl PrivilegeStore for PrivilegeStoreMock {
            async fn get_privilege_level(&self, role: &str) -> Result<i32, String>;
        }
    }

    struct TestEnv {
        root_dir: TempDir,
        username: String,
    }

    impl TestEnv {
        async fn new() -> Self {
            let root_dir = tempdir().unwrap();
            let username = "test_user".to_string();

            // Create user directory structure
            let user_dir = root_dir.path().join(&username);
            create_dir(&user_dir).unwrap();

            // Create test directory and file
            create_dir(user_dir.join("test_dir")).unwrap();
            File::create(user_dir.join("test_file.txt")).unwrap();

            TestEnv { root_dir, username }
        }
    }

    // Define a global OnceCell. Note that OnceCell is thread-safe.
    static GLOBAL_TEST_ENV: OnceCell<TestEnv> = OnceCell::const_new();

    // A helper function to get the global object. It will initialize it on the first call.
    async fn get_global_test_env() -> &'static TestEnv {
        GLOBAL_TEST_ENV
            .get_or_init(|| async {
                TestEnv::new().await
            })
            .await
    }

    // Helper function to create test directory structure
    fn create_test_structure(root: &Path) -> io::Result<()> {
        let dir1 = root.join("test_dir");
        create_dir(&dir1)?;
        File::create(dir1.join("file1.txt"))?;
        File::create(dir1.join("file2.rs"))?;

        let sub_dir = dir1.join("sub_dir");
        create_dir(&sub_dir)?;
        File::create(sub_dir.join("sub_file.txt"))?;

        Ok(())
    }

    #[tokio::test]
    async fn test_build_dir_tree() {
        let temp_dir = tempdir().unwrap();let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let directory_service = DirectoryService::new(root.clone());
        create_test_structure(temp_dir.path()).unwrap();

        let tree = directory_service.build_dir_tree(&temp_dir.path().join("test_dir")).unwrap();

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
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let directory_service = DirectoryService::new(root.clone());
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
        let temp_dir = tempdir().unwrap();
        let empty_dir = temp_dir.path().join("empty_dir");
        let env = get_global_test_env().await;
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