#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::fs;
    use std::io::{Cursor, Read};
    use actix_web::{test, web, App, http::header, http::StatusCode};
    use multipart::client::lazy::Multipart;
    use crate::app_config::AppConfig;
    use crate::endpoints::system_operations::upload::upload_file_from_user_directory;
    use crate::services::authentication::authentication_service::generate_jwt;
    use crate::tests::test_structure::get_global_test_env;

    #[actix_web::test]
    async fn test_upload_file_success() {
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let subdir = "some/subdir";
        let file_name = "example.txt";
        let file_content = "Hello, world!";

        let token = generate_jwt(username.to_string()).expect("failed to generate token");

        let mut form = Multipart::new();
        form.add_stream(
            "path",
            Cursor::new(subdir),
            None::<&str>,
            None
        );
        form.add_stream(
            "file",
            Cursor::new(file_content),
            Some(file_name.to_string()),
            None
        );

        // Convert the multipart form into a body, capturing the boundary for the Content-Type header
        let mut prepared_form = form.prepare().unwrap();
        let content_type = format!("multipart/form-data; boundary={}", prepared_form.boundary());

        // Read the entire multipart body into memory
        let mut form_bytes = Vec::new();
        prepared_form.read_to_end(&mut form_bytes).unwrap();

        // 4. Create the Actix test application
        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .service(upload_file_from_user_directory) // Your upload handler
        )
            .await;

        // 5. Build the request with headers and body
        let req = test::TestRequest::post()
            .uri("/upload")
            .insert_header((header::CONTENT_TYPE, content_type))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_payload(form_bytes) // The multipart body
            .to_request();

        // 6. Call the service and verify
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK, "Upload should succeed");
        let body = test::read_body(resp).await;
        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            "File uploaded successfully"
        );

        // 7. Confirm file was actually written
        let expected_path = test_root.join(username).join(subdir).join(file_name);
        assert!(
            expected_path.exists(),
            "Uploaded file should be found at the expected path"
        );

        // Optionally, read back and verify content
        let uploaded_content = fs::read_to_string(expected_path).expect("failed to read uploaded file");
        assert_eq!(uploaded_content, file_content);
    }

    #[actix_web::test]
    async fn test_upload_file_missing_path_field() {
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let file_name = "example.txt";
        let file_content = "Hello, world!";

        let token = generate_jwt(username.to_string()).expect("failed to generate token");

        // Only send the "file" field, not "path"
        let mut form = Multipart::new();
        form.add_stream(
            "file",
            Cursor::new(file_content),
            Some(file_name.to_string()),
            None
        );

        let mut prepared_form = form.prepare().unwrap();
        let content_type = format!("multipart/form-data; boundary={}", prepared_form.boundary());
        let mut form_bytes = Vec::new();
        prepared_form.read_to_end(&mut form_bytes).unwrap();

        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .service(upload_file_from_user_directory)
        )
            .await;

        let req = test::TestRequest::post()
            .uri("/upload")
            .insert_header((header::CONTENT_TYPE, content_type))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_payload(form_bytes)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Expect a 400 with "Missing path field"
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should fail on missing path");
        let body = test::read_body(resp).await;
        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            "No path was provided in the request"
        );
    }

    #[actix_web::test]
    async fn test_upload_file_missing_file_field() {
        let env = get_global_test_env().await;
        let test_root = env.root_dir.path();
        let username = "test_user";
        let subdir = "some/subdir";

        let token = generate_jwt(username.to_string()).expect("failed to generate token");

        // Only send the "path" field, not "file"
        let mut form = Multipart::new();
        form.add_stream(
            "path",
            Cursor::new(subdir),
            None::<&str>,
            None
        );

        let mut prepared_form = form.prepare().unwrap();
        let content_type = format!("multipart/form-data; boundary={}", prepared_form.boundary());
        let mut form_bytes = Vec::new();
        prepared_form.read_to_end(&mut form_bytes).unwrap();

        let config = AppConfig {
            root_dir: Arc::new(test_root.to_str().unwrap().to_string()),
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config))
                .service(upload_file_from_user_directory)
        )
            .await;

        let req = test::TestRequest::post()
            .uri("/upload")
            .insert_header((header::CONTENT_TYPE, content_type))
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .set_payload(form_bytes)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Expect a 400 with "Missing file field"
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "Should fail on missing file");
        let body = test::read_body(resp).await;
        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            "No file was provided in the request"
        );
    }

}
