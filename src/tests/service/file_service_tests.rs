#[cfg(test)]
mod tests {
    use std::fs::{create_dir, File};
    use std::io;
    use std::io::Write;
    use std::path::Path;
    use tempfile::{tempdir, TempDir};
    use tokio::fs;
    use tokio::sync::OnceCell;
    use crate::services::file_structure::file_service::FileService;
    use crate::services::locking::directory_locking_manager::DirectoryLockManager;

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

            create_test_structure((&root_dir).as_ref(), &username).expect("Could not generate test environment!");
            TestEnv { root_dir, username }
        }
    }

    // Helper function to create test directory structure
    fn create_test_structure(root: &Path, user: &String) -> io::Result<()> {
        let dir1 = root.join(user).join("test_dir");

        let mut file1 = File::create(dir1.join("file1.txt"))?;
        file1.write_all(b"Some text!").expect("Could not write to file1");
        let mut file2 = File::create(dir1.join("file2.rs"))?;
        file2.write_all(b"Some code!").expect("Could not write to file2");

        let sub_dir = dir1.join("sub_dir");
        create_dir(&sub_dir)?;
        File::create(sub_dir.join("sub_file.txt"))?;

        Ok(())
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


    #[tokio::test]
    async fn sanitize_file_name() {
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();

        let file_service = FileService::new(
            root,
            DirectoryLockManager::new()
        );
        let res = file_service.sanitize_filename("valid_name.txt");
        assert_eq!(res, "valid_name.txt");
        let res = file_service.sanitize_filename("back\\\\slashes\\\\.txt");
        assert_eq!(res, "backslashes.txt");
        let res = file_service.sanitize_filename("valid/_name.txt");
        assert_eq!(res, "valid_name.txt");
    }

    #[tokio::test]
    async fn test_read_file_successful() {
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let user = &env.username;

        let file_service = FileService::new(
            root,
            DirectoryLockManager::new()
        );

        let contents = file_service.read_file_from_any_directory(
            user,
            "test_dir/",
            "file2.rs"
        ).await;

        assert!(contents.is_ok());
        assert_eq!(Ok((b"Some code!".to_vec(), "file2.rs".into())), contents);
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let user = &env.username;

        let file_service = FileService::new(
            root,
            DirectoryLockManager::new()
        );

        let contents = file_service.read_file_from_any_directory(
            user,
            "test_dir/",
            "file22.rs"
        ).await;

        assert!(contents.is_err());
    }
    
    #[tokio::test]
    async fn test_save_file_bytes_to_root_directory_success() {
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let user = &env.username;

        // Instantiate FileService
        let file_service = FileService::new(
            root.clone(),
            DirectoryLockManager::new()
        );

        // Construct a path for the new file inside "test_dir"
        let new_file_path = Path::new(&root)
            .join(user)
            .join("test_dir")
            .join("saved_bytes.txt");

        // The content we want to write
        let file_content = b"Hello from test_save_file_bytes";

        // Call the method
        let result = file_service
            .save_file_bytes_to_root_directory(&new_file_path, file_content)
            .await;

        // Verify success
        assert!(result.is_ok(), "Expected Ok from save_file_bytes_to_root_directory");
        assert_eq!(result.unwrap(), "Successfully saved file!".to_string());

        // Check the file actually exists and contains the data
        let saved_contents = fs::read_to_string(&new_file_path)
            .await.expect("Failed to read back the saved file");
        assert_eq!(saved_contents, "Hello from test_save_file_bytes");
    }

    #[tokio::test]
    async fn test_save_file_bytes_to_root_directory_failure_no_parent_dir() {
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let user = &env.username;

        // Instantiate FileService
        let file_service = FileService::new(
            root.clone(),
            DirectoryLockManager::new()
        );

        // Construct a path to a directory that doesn't exist (and we won't create).
        let no_such_dir_path = Path::new(&root)
            .join(user)
            .join("nonexistent_subdir")
            .join("file_should_fail.txt");

        // Attempt to save; this should fail because "nonexistent_subdir" doesn't exist
        let result = file_service
            .save_file_bytes_to_root_directory(&no_such_dir_path, b"some data")
            .await;

        // We expect an error
        assert!(result.is_err(), "Expected Err when parent directories do not exist");
        let err_msg = result.err().unwrap();
        assert_eq!(err_msg.0, 500);
        assert!(
            err_msg.1.contains("Error creating file"),
            "Expected error creating file error; got: {}",
            err_msg.1
        );

        // Confirm the file was not created
        assert!(!no_such_dir_path.exists(), "File should not exist after failure");
    }
}