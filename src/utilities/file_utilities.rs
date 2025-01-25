use crate::ROOT_DIR;
use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Responder};
use futures_util::TryStreamExt;
use log::info;
use percent_encoding::percent_decode_str;
use std::path::Path;
use tokio::io::AsyncWriteExt;

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| *c != '/' && *c != '\\')
        .collect()
}

pub(crate) async fn save_file_to_root_directory(
    payload: &mut Multipart,
    user_directory: &str,
) -> Result<String, String> {
    let mut target_path: Option<String> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();

        // Check for the "path" field
        if let Some(name) = content_disposition.and_then(|cd| cd.get_name()) {
            if name == "path" {
                // Read the path field value
                let mut path_bytes = Vec::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    path_bytes.extend_from_slice(&chunk);
                }
                target_path = Some(String::from_utf8(path_bytes).map_err(|e| {
                    format!("Failed to parse path as UTF-8: {:?}", e)
                })?);
                continue; // Skip processing as a file
            }
        }

        // Check for the "file" field
        if let Some(filename) = content_disposition.and_then(|cd| cd.get_filename()) {
            let filename = sanitize_filename(filename);

            // Ensure the target path exists and build the final file path
            let (target_dir, filepath) = match &target_path {
                Some(path) => {
                    let full_path = Path::new(ROOT_DIR).join(user_directory).join(path);

                    let directory = full_path.parent().ok_or_else(|| {
                        format!(
                            "Invalid path: Unable to determine directory for {:?}",
                            full_path
                        )
                    })?;
                    (directory.to_path_buf(), full_path.join(&filename))
                }
                None => {
                    let directory = Path::new(user_directory);
                    (directory.to_path_buf(), directory.join(&filename))
                }
            };

            // Ensure the directory exists
            if let Err(e) = tokio::fs::create_dir_all(&target_dir).await {
                return Err(format!(
                    "Failed to create target directory {:?}: {:?}",
                    target_dir, e
                ));
            }

            // Create the file asynchronously
            let mut f = match tokio::fs::File::create(&filepath).await {
                Ok(file) => file,
                Err(e) => {
                    return Err(format!(
                        "Error creating file at {:?}: {:?}",
                        filepath.display(),
                        e
                    ));
                }
            };

            // Write chunks asynchronously
            while let Ok(Some(chunk)) = field.try_next().await {
                if let Err(e) = f.write_all(&chunk).await {
                    return Err(format!("Error writing chunk: {:?}", e));
                }
            }

            println!("File saved successfully: {:?}", filepath);
        }
    }

    Ok("Successfully saved file!".to_string())
}
pub(crate) async fn read_file_from_directory(
    req: HttpRequest,
    user_directory: &str,
) -> impl Responder {
    let query_str = req.query_string();
    let parts: Vec<&str> = query_str.split('=').collect();
    if parts.len() != 2 || parts[0] != "filename" {
        return HttpResponse::BadRequest().body("Missing or invalid 'filename' parameter.");
    }

    let encoded_filename = parts[1];
    let decoded_filename = match percent_decode_str(encoded_filename).decode_utf8() {
        Ok(decoded) => decoded.into_owned(),
        Err(_) => {
            return HttpResponse::BadRequest().body("Failed to decode 'filename' parameter.");
        }
    };

    let filepath = join_user_directory(user_directory, &decoded_filename);
    info!("This is the resulting path: {}", filepath);

    match std::fs::read(&filepath) {
        Ok(contents) => HttpResponse::Ok().body(contents),
        Err(_) => HttpResponse::NotFound().body(format!("File '{}' not found", decoded_filename)),
    }
}

fn join_user_directory(user_directory: &str, filename: &str) -> String {
    if user_directory.is_empty() {
        format!("{}/{}", ROOT_DIR, filename)
    } else {
        format!("{}/{}/{}", ROOT_DIR, user_directory, filename)
    }
}