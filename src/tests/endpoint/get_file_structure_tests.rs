#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::sync::Arc;
    use actix_web::http::header::AUTHORIZATION;
    use actix_web::{test, web, App};
    use crate::app_config::AppConfig;
    use crate::endpoints::system_operations::get_file_structure::get_user_directory;
    use crate::models::authentication::auth_models::JwtAuth;
    use crate::models::file_structure::directory_tree::DirTree;
    use crate::models::file_structure::file_structure_request::FileStructureRequest;
    use crate::services::authentication::authentication_service::generate_jwt;
    use crate::services::locking::directory_locking_manager::DirectoryLockManager;
    use crate::tests::test_structure::get_global_test_env;
    
    #[actix_web::test]
    async fn test_get_structure_request() {
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let user_dir = test_root.join(username);
        let sub_path = "folder/sub_folder";
        let file_dir = user_dir.join(sub_path);
        let file_to_download = "test.txt";

        fs::create_dir_all(&file_dir).expect("failed to create files directory");
        // Ensure the directory exists
        let target_file = user_dir.join(sub_path).join(file_to_download);
        File::create(&target_file).expect("Could not create file!");

        let payload = FileStructureRequest {
            path: sub_path.to_string(),
        };

        let token = generate_jwt("test_user".to_string()).expect("failed to generate token");
        let req = test::TestRequest::post()
            .uri("/structure")
            .insert_header((AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(&payload)
            .to_request();

        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
            directory_lock_manager: DirectoryLockManager::new()
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .wrap(JwtAuth)
                .service(get_user_directory)
        ).await;

        let resp = test::call_service(&app, req).await;
        println!("{}", resp.status());
        assert_eq!(resp.status(), 200);

        let body_bytes = test::read_body(resp).await;

        let dir_tree: DirTree = serde_json::from_slice(&body_bytes)
            .expect("Failed to deserialize response into DirTree");

        assert_eq!(dir_tree.name, "sub_folder");
        assert_eq!(dir_tree.files.len(), 1);
        assert_eq!(dir_tree.files[0], "test.txt");
        assert!(dir_tree.dirs.is_empty(), "Expected no subdirectories");
    }
}