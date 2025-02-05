#[cfg(test)]
mod tests {
    use std::path::Path;
    use tokio::fs;
    use crate::services::file_structure::rename_service::{RenameService};
    use crate::tests::test_structure::get_global_test_env;
    // Adjust imports as needed

    #[tokio::test]
    async fn test_rename_directory_successful() {
        // Obtain the global test environment (adjust this to your actual test env setup)
        let env = get_global_test_env().await;
        let root = env.root_dir.path().to_str().unwrap().to_string();
        let user = &env.username;

        // Instantiate the service
        let rename_service = RenameService::new(root.clone());

        // Define path components
        let test_subdir = "test_dir";
        let old_dir_name = "old_name";
        let new_dir_name = "new_name";

        // Create a directory that we'll rename
        let old_dir_path = Path::new(&root)
            .join(user)
            .join(test_subdir)
            .join(old_dir_name);

        fs::create_dir_all(&old_dir_path).await.unwrap();

        // Perform rename using the service
        let result = rename_service
            .rename_directory(
                user,
                &test_subdir.to_string(),
                &old_dir_name.to_string(),
                &new_dir_name.to_string(),
            )
            .await;

        // Validate the result
        assert!(result.is_ok(), "Rename should succeed");
        assert_eq!(Ok("Successfully renamed".to_string()), result);

        // Verify that the new directory now exists
        let new_dir_path = Path::new(&root)
            .join(user)
            .join(test_subdir)
            .join(new_dir_name);
        let new_metadata = fs::metadata(&new_dir_path).await;
        assert!(new_metadata.is_ok(), "New directory should exist after rename");

        // Verify that the old directory no longer exists
        let old_metadata = fs::metadata(&old_dir_path).await;
        assert!(
            old_metadata.is_err(),
            "Old directory should not exist after rename"
        );
    }
}