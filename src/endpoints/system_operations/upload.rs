use crate::services::file_structure::file_service;
use actix_web::{post, web, HttpResponse, Responder};
use std::path::Path;
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use log::{error, info};
use crate::app_config::AppConfig;
use crate::models::authentication::auth_user::AuthenticatedUser;
use crate::models::system_operations::upload_file_request::{UploadRequestData};

/// POST endpoint to handle file uploads from the user directory.
#[post("/upload")]
pub async fn upload_file_from_user_directory(
    mut payload: Multipart,
    authenticated_user: AuthenticatedUser,
    config: web::Data<AppConfig>,
) -> impl Responder {
    let username = authenticated_user.0.sub;
    let file_service = file_service::FileService::new(config.root_dir.as_ref().clone());

    // We'll store all fields in this struct while iterating,
    // then process them after the loop to avoid ordering issues.
    let mut data = UploadRequestData {
        filename: None,
        file_bytes: None,
        path: None,
    };

    // Iterate over multipart fields
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();

        // Get the field name
        let field_name = match content_disposition.and_then(|cd| cd.get_name()) {
            Some(name) => name.to_string(),
            None => {
                error!("No field name in content disposition");
                continue; // skip this field
            }
        };

        match field_name.as_str() {
            "path" => {
                // Read the entire path field into a string
                let mut path_bytes = Vec::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    path_bytes.extend_from_slice(&chunk);
                }
                match String::from_utf8(path_bytes) {
                    Ok(path_str) => {
                        let cleaned = path_str.trim().to_string();
                        data.path = Some(cleaned);
                    }
                    Err(e) => {
                        error!("Failed to parse path as UTF-8: {:?}", e);
                        return HttpResponse::BadRequest().body("Invalid path encoding");
                    }
                }
            }

            "file" => {
                // We have a file: read its entire content into memory
                // and store the sanitized filename
                if let Some(filename) = content_disposition.and_then(|cd| cd.get_filename()) {
                    // Sanitize the filename
                    let sanitized_filename = file_service.sanitize_filename(&filename);

                    // Read file bytes
                    let mut file_bytes = Vec::new();
                    while let Ok(Some(chunk)) = field.try_next().await {
                        file_bytes.extend_from_slice(&chunk);
                    }

                    // Store in our data struct
                    data.filename = Some(sanitized_filename);
                    data.file_bytes = Some(file_bytes);
                } else {
                    error!("File field without filename");
                    return HttpResponse::BadRequest().body("File field missing filename");
                }
            }

            _ => {
                info!("Unknown multipart field: {}", field_name);
                // Skip or handle other fields as needed.
                while let Ok(Some(_chunk)) = field.try_next().await {
                    // Drain the field
                }
            }
        }
    }

    // Now that we've read all fields, check if we have both the file and path
    let filename = match data.filename {
        Some(f) => f,
        None => {
            return HttpResponse::BadRequest().body("No file was provided in the request");
        }
    };

    let file_bytes = match data.file_bytes {
        Some(bytes) => bytes,
        None => {
            return HttpResponse::BadRequest().body("File bytes were not captured");
        }
    };

    let path = match data.path {
        Some(p) => p,
        None => {
            return HttpResponse::BadRequest().body("No path was provided in the request");
        }
    };

    // Now we can safely write the file to disk
    let abs_path = Path::new(config.root_dir.as_ref())
        .join(&username)
        .join(path.trim_start_matches('/'))
        .join(&filename);

    // Create all necessary directories if not present
    if let Some(parent) = abs_path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            error!("Failed to create directories: {}", e);
            return HttpResponse::InternalServerError()
                .body("Failed to create target directory");
        }
    }

    // Save the file
    match file_service
        .save_file_bytes_to_root_directory(&abs_path, &file_bytes)
        .await
    {
        Ok(success) => {
            info!("{}", success);
        }
        Err(err) => {
            error!("{}", err);
            return HttpResponse::InternalServerError().body(err);
        }
    }

    HttpResponse::Ok().body("File uploaded successfully")
}