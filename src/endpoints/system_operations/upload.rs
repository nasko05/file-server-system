use crate::services::file_structure::file_service;
use crate::ROOT_DIR;
use actix_web::{post, HttpResponse, Responder};
use std::path::Path;
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use log::{error, info};
use crate::models::authentication::auth_user::AuthenticatedUser;
use crate::models::system_operations::upload_file_request::UploadFileRequest;

/// POST endpoint to handle file uploads from the user directory.
#[post("/upload")]
pub async fn upload_file_from_user_directory(
    mut payload: Multipart,
    authenticated_user: AuthenticatedUser
) -> impl Responder {

    let username = authenticated_user.0.sub;
    let file_service = file_service::FileService::new(ROOT_DIR.into());
    
    let mut data = UploadFileRequest {
        file: None,
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
                continue;
            }
        };

        match field_name.as_str() {
            "file" => {
                // Handle file field
                if let Some(filename) = content_disposition.and_then(|cd| cd.get_filename()) {
                    let sanitized_filename = file_service.sanitize_filename(&filename);

                    // Save the file directly by calling the utility function
                    // We need to construct the absolute path first
                    if let Some(ref path) = data.path {
                        let abs_path = Path::new(ROOT_DIR)
                            .join(&username)
                            .join(path.trim_start_matches('/'))
                            .join(&sanitized_filename);

                        match file_service.save_file_to_root_directory(&abs_path, &mut field).await {
                            Ok(success) => info!("{}", success),
                            Err(err) => {
                                error!("{}", err);
                                return HttpResponse::InternalServerError().body(err);
                            },
                        }
                    } else {
                        error!("Path not set before file field");
                        return HttpResponse::BadRequest().body("Path not provided before file field");
                    }
                } else {
                    error!("File field without filename");
                    return HttpResponse::BadRequest().body("File field missing filename");
                }
            }
            "path" => {
                // Handle path field
                let mut path_bytes = Vec::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    path_bytes.extend_from_slice(&chunk);
                }
                match String::from_utf8(path_bytes) {
                    Ok(path_str) => data.path = Some(path_str.trim().to_string()),
                    Err(e) => {
                        error!("Failed to parse path as UTF-8: {:?}", e);
                        return HttpResponse::BadRequest().body("Invalid path encoding");
                    }
                }
            }
            _ => {
                info!("Unknown field: {}", field_name);
                // Optionally, handle or skip unknown fields
            }
        }
    }

    // Validate required fields
    if data.file.is_none() {
        return HttpResponse::BadRequest().body("Missing file field");
    }
    if data.path.is_none() {
        return HttpResponse::BadRequest().body("Missing path field");
    }

    HttpResponse::Ok().body("File uploaded successfully")
}