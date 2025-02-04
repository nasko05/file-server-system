#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::sync::Arc;
    use actix_web::http::header::AUTHORIZATION;
    use actix_web::{test, web, App};
    use crate::app_config::AppConfig;
    use crate::endpoints::system_operations::download::download_file_from_user_directory;
    use crate::models::authentication::auth_models::JwtAuth;
    use crate::models::system_operations::download_file_request::DownloadFileRequest;
    use crate::services::authentication::authentication_service::generate_jwt;
    use crate::tests::test_structure::get_global_test_env;

    #[actix_web::test]
    async fn test_download_file_from_user_directory() {
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let user_dir = test_root.join(username);
        let sub_path = "";
        let file_dir = user_dir.join(sub_path);
        let file_to_download = "test.txt";

        fs::create_dir_all(&file_dir).expect("failed to create files directory");
        // Ensure the directory exists
        let target_file = user_dir.join(sub_path).join(file_to_download);
        let mut file = File::create(&target_file).expect("Could not create file!");
        file.write_all(b"Some text!").expect("Could not write to file!");

        let payload = DownloadFileRequest {
            filename: file_to_download.to_string(),
            path: sub_path.to_string(),
        };

        let token = generate_jwt("test_user".to_string()).expect("failed to generate token");
        let req = test::TestRequest::post()
            .uri("/download")
            .insert_header((AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(&payload)
            .to_request();

        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string())
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .wrap(JwtAuth)
                .service(download_file_from_user_directory)
        ).await;

        let resp = test::call_service(&app, req).await;

        println!("{}", resp.status());
        assert_eq!(resp.status(), 200);
        assert!(target_file.exists());
        let body = test::read_body(resp).await;
        let expected_msg = "Some text!";
        assert_eq!(std::str::from_utf8(&body).unwrap(), expected_msg);
    }

    #[actix_web::test]
    async fn test_download_file_not_found() {
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let user_dir = test_root.join(username);
        let sub_path = "";
        let file_dir = user_dir.join(sub_path);
        let file_to_download = "test.txt";

        fs::create_dir_all(&file_dir).expect("failed to create files directory");
        // Ensure the directory exists
        let target_file = user_dir.join(sub_path).join(file_to_download);

        let payload = DownloadFileRequest {
            filename: file_to_download.to_string(),
            path: sub_path.to_string(),
        };

        let token = generate_jwt("test_user".to_string()).expect("failed to generate token");
        let req = test::TestRequest::post()
            .uri("/download")
            .insert_header((AUTHORIZATION, format!("Bearer {}", token)))
            .set_json(&payload)
            .to_request();

        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string())
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .wrap(JwtAuth)
                .service(download_file_from_user_directory)
        ).await;

        let resp = test::call_service(&app, req).await;

        println!("{}", resp.status());
        assert_eq!(resp.status(), 400);
        assert!(!target_file.exists());
    }
}