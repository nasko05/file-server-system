#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::sync::Arc;
    use actix_web::{test, App, web, http::header::AUTHORIZATION};
    use tokio::fs;
    use crate::app_config::AppConfig;
    use crate::models::system_operations::rename_item_request::RenameItemRequest;
    use crate::endpoints::system_operations::rename::{rename_directory};
    use crate::services::authentication::authentication_service::generate_jwt;
    use crate::tests::test_structure::get_global_test_env;
    

    #[actix_web::test]
    async fn test_rename_directory_success() {
        // 1. Setup test environment
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let sub_path = "test_dir/";
        let old_dir_name = "old_folder";
        let new_dir_name = "new_folder";

        // 2. Create a directory to rename
        let user_dir = test_root.join(username);
        let sub_dir = user_dir.join(sub_path);
        fs::create_dir_all(&sub_dir).await.expect("failed to create sub_dir");

        let old_folder_path = sub_dir.join(old_dir_name);
        fs::create_dir_all(&old_folder_path)
            .await
            .expect("failed to create old_folder");

        // 3. Construct the request payload
        let payload = RenameItemRequest {
            path: sub_path.to_string(),
            old_name: old_dir_name.to_string(),
            new_name: new_dir_name.to_string(),
        };

        // 4. Create and sign a JWT token (assuming you have some utility for that)
        let token = generate_jwt(username.to_string()).expect("failed to generate token");

        // 5. Build the Actix test request
        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/directory/rename")
            .set_json(&payload)
            .to_request();

        // 6. Initialize the Actix test application with your config + service
        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .service(rename_directory)
        )
            .await;

        // 7. Call the service
        let resp = test::call_service(&app, req).await;

        // 8. Verify response
        assert_eq!(resp.status(), 200);
        let body = test::read_body(resp).await;
        assert_eq!(std::str::from_utf8(&body).unwrap(), "Successfully renamed");

        // 9. Check the file system changes
        assert!(
            !old_folder_path.exists(),
            "Old directory should not exist after rename"
        );

        let new_folder_path = sub_dir.join(new_dir_name);
        assert!(
            new_folder_path.exists(),
            "New directory must exist after successful rename"
        );
    }

    #[actix_web::test]
    async fn test_rename_directory_not_found() {
        // 1. Setup test environment
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let sub_path = "test_dir/";
        let old_dir_name = "non_existent";
        let new_dir_name = "renamed";

        // Notice we do NOT create the `non_existent` directory here

        // 2. Construct request payload
        let payload = RenameItemRequest {
            path: sub_path.to_string(),
            old_name: old_dir_name.to_string(),
            new_name: new_dir_name.to_string(),
        };

        // 3. Create token and request
        let token = generate_jwt(username.to_string()).expect("failed to generate token");
        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/directory/rename")
            .set_json(&payload)
            .to_request();

        // 4. Initialize Actix app
        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .service(rename_directory)
        )
            .await;

        // 5. Call the service
        let resp = test::call_service(&app, req).await;

        // 6. Verify response: should be 404 "File not found"
        assert_eq!(resp.status(), 404);
        let body = test::read_body(resp).await;
        assert_eq!(std::str::from_utf8(&body).unwrap(), "File not found");
    }

    #[actix_web::test]
    async fn test_rename_file_success() {
        // Even though the endpoint is called `rename_directory`, your code
        // allows renaming both files and directories if they exist. 
        //
        // So let's test the "file" scenario:

        // 1. Setup environment
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let sub_path = "files/";
        let old_file_name = "my_file.txt";
        let new_file_name = "renamed_file.txt";

        // 2. Create a file to rename
        let user_dir = test_root.join(username);
        let files_dir = user_dir.join(sub_path);
        fs::create_dir_all(&files_dir).await.expect("failed to create files dir");

        let old_file_path = files_dir.join(old_file_name);
        {
            let mut file = File::create(&old_file_path)
                .expect("failed to create test file");
            writeln!(file, "dummy content").expect("failed to write dummy content");
        }

        // 3. Construct request payload
        let payload = RenameItemRequest {
            path: sub_path.to_string(),
            old_name: old_file_name.to_string(),
            new_name: new_file_name.to_string(),
        };

        // 4. Create token and request
        let token = generate_jwt(username.to_string()).expect("failed to generate token");
        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/directory/rename")
            .set_json(&payload)
            .to_request();

        // 5. Initialize Actix app
        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .service(rename_directory)
        )
            .await;

        // 6. Call the service
        let resp = test::call_service(&app, req).await;

        // 7. Verify response
        assert_eq!(resp.status(), 200);
        let body = test::read_body(resp).await;
        assert_eq!(std::str::from_utf8(&body).unwrap(), "Successfully renamed");

        // 8. Check file system changes
        assert!(
            !old_file_path.exists(),
            "Old file should not exist after rename"
        );

        let new_file_path = files_dir.join(new_file_name);
        assert!(
            new_file_path.exists(),
            "New file must exist after successful rename"
        );
    }
}