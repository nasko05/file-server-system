#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::ptr::null;
    use std::sync::Arc;
    use actix_web::http::header::AUTHORIZATION;
    use crate::app_config::AppConfig;
    use crate::endpoints::system_operations::delete::{delete_file, delete_user_directory};
    use crate::models::authentication::auth_models::JwtAuth;
    use crate::models::system_operations::delete_file_request::DeleteEntityRequest;
    use crate::services::authentication::authentication_service::{generate_jwt};
    use crate::services::locking::directory_locking_manager::DirectoryLockManager;
    use crate::tests::test_structure::get_global_test_env;

    /// Test the `/directory/delete` endpoint when the target directory exists.
    #[actix_web::test]
    async fn test_delete_user_directory_success() {
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let dir_to_delete = "delete_me";
        let sub_path = ""; // Use empty path if not nested.
        let user_dir = test_root.join(username);
        
        // Ensure the directory exists
        let target_dir = user_dir.join(sub_path).join(dir_to_delete);
        fs::create_dir_all(&target_dir).expect("failed to create target directory");
        
        let payload = DeleteEntityRequest {
            name: dir_to_delete.to_string(),
            path: sub_path.to_string(),
        };

        let token = generate_jwt("test_user".to_string()).expect("failed to generate token");
        // Create a test request.
        let req = test::TestRequest::post()
            .uri("/directory/delete")
            .insert_header((AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(&payload)
            .to_request();

        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
            directory_lock_manager: DirectoryLockManager::new()
        };
        // Initialize an Actix Web App with the delete_user_directory endpoint.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .wrap(JwtAuth)
                .service(delete_user_directory)
        ).await;
        
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
        let body = test::read_body(resp).await;
        let expected_msg = format!("Directory '{}' deleted successfully.", dir_to_delete);
        assert_eq!(std::str::from_utf8(&body).unwrap(), expected_msg);
        assert!(!target_dir.exists());
    }

    /// Test the `/directory/delete` endpoint when the target directory does not exist.
    #[actix_web::test]
    async fn test_delete_user_directory_not_found() {
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let dir_name = "nonexistent";
        let sub_path = "";
        
        let user_dir = test_root.join(username);
        fs::create_dir_all(&user_dir).expect("failed to create user directory");
        
        let payload = DeleteEntityRequest {
            name: dir_name.to_string(),
            path: sub_path.to_string(),
        };

        let token = generate_jwt("test_user".to_string()).expect("failed to generate token");
        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/directory/delete")
            .set_json(&payload)
            .to_request();
        
        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
            directory_lock_manager: DirectoryLockManager::new()
        };
        let app = test::init_service(
            App::new()
            .app_data(web::Data::new(config))
            .service(delete_user_directory)
        ).await;
        let resp = test::call_service(&app, req).await;

        // Expect a NotFound response.
        assert_eq!(resp.status(), 404);
    }

    /// Test the `/directory/delete` endpoint when the target exists but is not a directory.
    #[actix_web::test]
    async fn test_delete_user_directory_not_a_directory() {
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let name = "not_a_directory";
        let sub_path = "";
        
        let user_dir = test_root.join(username);
        fs::create_dir_all(&user_dir).expect("failed to create user directory");
        let file_path = user_dir.join(sub_path).join(name);
        File::create(&file_path).expect("failed to create file");

        let payload = DeleteEntityRequest {
            name: name.to_string(),
            path: sub_path.to_string(),
        };

        let token = generate_jwt("test_user".to_string()).expect("failed to generate token");
        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/directory/delete")
            .set_json(&payload)
            .to_request();

        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
            directory_lock_manager: DirectoryLockManager::new()
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .service(delete_user_directory)
        ).await;
        let resp = test::call_service(&app, req).await;

        // Expect a BadRequest response.
        assert_eq!(resp.status(), 400);
    }

    /// Test the `/file/delete` endpoint when the target file exists.
    #[actix_web::test]
    async fn test_delete_file_success() {
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let filename = "delete_me.txt";
        let sub_path = "files/";
        
        let user_dir = test_root.join(username);
        let files_dir = user_dir.join(sub_path);
        fs::create_dir_all(&files_dir).expect("failed to create files directory");
        let file_path = files_dir.join(filename);
        {
            let mut file = File::create(&file_path).expect("failed to create file");
            write!(file, "dummy content").expect("failed to write file content");
        }

        let payload = DeleteEntityRequest {
            name: filename.to_string(),
            path: sub_path.to_string(),
        };

        let token = generate_jwt("test_user".to_string()).expect("failed to generate token");
        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/file/delete")
            .set_json(&payload)
            .to_request();

        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
            directory_lock_manager: DirectoryLockManager::new()
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .service(delete_file)
        ).await;
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body = test::read_body(resp).await;
        let expected_msg = format!("File '{}' deleted successfully.", filename);
        assert_eq!(std::str::from_utf8(&body).unwrap(), expected_msg);
        assert!(!file_path.exists());
    }

    /// Test the `/file/delete` endpoint when the file does not exist.
    #[actix_web::test]
    async fn test_delete_file_not_found() {
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let filename = "nonexistent.txt";
        let sub_path = "files/";
        
        let user_dir = test_root.join(username);
        let files_dir = user_dir.join(sub_path);
        fs::create_dir_all(&files_dir).expect("failed to create files directory");

        let payload = DeleteEntityRequest {
            name: filename.to_string(),
            path: sub_path.to_string(),
        };

        let token = generate_jwt("test_user".to_string()).expect("failed to generate token");
        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/file/delete")
            .set_json(&payload)
            .to_request();

        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
            directory_lock_manager: DirectoryLockManager::new()
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .service(delete_file)
        ).await;
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    /// Test the `/file/delete` endpoint when the target exists but is not a file.
    #[actix_web::test]
    async fn test_delete_file_not_a_file() {
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let name = "not_a_file";
        let sub_path = "files/";
        
        let user_dir = test_root.join(username);
        let files_dir = user_dir.join(sub_path);
        fs::create_dir_all(&files_dir).expect("failed to create files directory");
        let dir_path = files_dir.join(name);
        fs::create_dir_all(&dir_path).expect("failed to create directory");

        let payload = DeleteEntityRequest {
            name: name.to_string(),
            path: sub_path.to_string(),
        };

        let token = generate_jwt("test_user".to_string()).expect("failed to generate token");
        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, format!("Bearer {}", token)))
            .uri("/file/delete")
            .set_json(&payload)
            .to_request();

        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
            directory_lock_manager: DirectoryLockManager::new()
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .service(delete_file)
        ).await;
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }
}