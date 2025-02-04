#[cfg(test)]
mod tests {
    use std::fs::{create_dir, File};
    use std::io;
    use std::io::Write;
    use std::path::Path;
    use bytes::Bytes;
    use futures_util::stream;
    use tempfile::{tempdir, TempDir};
    use tokio::sync::OnceCell;
    use crate::services::file_structure::file_service::FileService;

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

        let file_service = FileService::new(root);
        let res = file_service.sanitize_filename("valid_name.txt");
        assert_eq!(res, "valid_name.txt");
        let res = file_service.sanitize_filename("back\\\\slashes\\\\.txt");
        assert_eq!(res, "backslashes.txt");
        let res = file_service.sanitize_filename("valid/_name.txt");
        assert_eq!(res, "valid_name.txt");
    }

    #[tokio::test]
    async fn save_file_success() {
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let user = &env.username;

        let full_path = Path::new(&root).join(user).join("test_file.txt");
        let file_service = FileService::new(root);
        let test_chunks: Vec<Result<Bytes, actix_multipart::MultipartError>> =
            vec![Ok(Bytes::from_static(b"Some data"))];
        let mut field = stream::iter(test_chunks);

        let res = file_service.save_file_to_root_directory(&full_path, &mut field).await;

        assert_eq!(res, Ok("Successfully saved file!".to_string()));
        let contents = tokio::fs::read(&full_path).await.unwrap_or_else(|_| b"".to_vec());

        assert_eq!(b"Some data".to_vec(), contents);
    }

    #[tokio::test]
    async fn test_empty_filename() {
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let user = &env.username;

        let full_path = Path::new(&root).join(user).join("");
        let file_service = FileService::new(root);
        let test_chunks: Vec<Result<Bytes, actix_multipart::MultipartError>> =
            vec![Ok(Bytes::from_static(b"Some data"))];
        let mut field = stream::iter(test_chunks);

        let res = file_service.save_file_to_root_directory(&full_path, &mut field).await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_empty_filename_with_extension() {
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let user = &env.username;

        let full_path = Path::new(&root).join(user).join(".txt");
        let file_service = FileService::new(root);
        let test_chunks: Vec<Result<Bytes, actix_multipart::MultipartError>> =
            vec![Ok(Bytes::from_static(b"Some data"))];
        let mut field = stream::iter(test_chunks);

        let res = file_service.save_file_to_root_directory(&full_path, &mut field).await;

        assert_eq!(res, Ok("Successfully saved file!".to_string()));
        let contents = tokio::fs::read(&full_path).await.unwrap_or_else(|_| b"".to_vec());

        assert_eq!(b"Some data".to_vec(), contents);
    }

    #[tokio::test]
    async fn test_write_empty_file() {
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let user = &env.username;

        let full_path = Path::new(&root)
            .join(user)
            .join("test_dir")
            .join("sub_dir")
            .join("sub_file.txt");
        let file_service = FileService::new(root);
        let test_chunks: Vec<Result<Bytes, actix_multipart::MultipartError>> =
            vec![Ok(Bytes::from_static(b""))];
        let mut field = stream::iter(test_chunks);

        let res = file_service.save_file_to_root_directory(&full_path, &mut field).await;

        assert_eq!(res, Ok("Successfully saved file!".to_string()));
        let contents = tokio::fs::read(&full_path).await.unwrap_or_else(|_| b"".to_vec());

        println!("{:?}", contents);
        assert_eq!(b"".to_vec(), contents);
    }

    #[tokio::test]
    async fn test_read_file_successful() {
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let user = &env.username;

        let file_service = FileService::new(root);

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

        let file_service = FileService::new(root);

        let contents = file_service.read_file_from_any_directory(
            user,
            "test_dir/",
            "file22.rs"
        ).await;

        assert!(contents.is_err());
    }
}