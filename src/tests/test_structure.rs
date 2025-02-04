use std::env;
use std::fs::{create_dir, File};
use std::io;
use std::io::Write;
use std::path::Path;
use tempfile::{tempdir, TempDir};
use tokio::sync::OnceCell;

pub struct TestEnv {
    pub root_dir: TempDir,
    pub username: String,
}

impl TestEnv {
    pub fn new() -> Self {
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
pub fn create_test_structure(root: &Path, user: &String) -> io::Result<()> {
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
pub static GLOBAL_TEST_ENV: OnceCell<TestEnv> = OnceCell::const_new();

// A helper function to get the global object. It will initialize it on the first call.
pub async fn get_global_test_env() -> &'static TestEnv {
    GLOBAL_TEST_ENV
        .get_or_init(|| async {
            env::set_var("JWT_TOKEN_SECRET", "some_secret_token");
            TestEnv::new()
        })
        .await
}