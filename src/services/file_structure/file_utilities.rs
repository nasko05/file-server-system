use crate::{ROOT_DIR};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
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
            let sanitized_filename = sanitize_filename(filename);

            // Build the full path and separate directory from the filename
            let sanitized_path = target_path
                .as_deref()
                .unwrap_or("")
                .trim_start_matches('/'); // Remove leading slash
            let full_path = Path::new(ROOT_DIR)
                .join(user_directory)
                .join(sanitized_path);

            let directory = full_path.parent().ok_or_else(|| {
                format!(
                    "Invalid path: Unable to determine directory for {:?}",
                    full_path
                )
            })?;

            let file_path = directory.join(&sanitized_filename); // Append filename to directory

            // Ensure the directory exists
            if let Err(e) = tokio::fs::create_dir_all(directory).await {
                return Err(format!(
                    "Failed to create target directory {:?}: {:?}",
                    directory, e
                ));
            }

            // Create the file asynchronously
            let mut f = match tokio::fs::File::create(&file_path).await {
                Ok(file) => file,
                Err(e) => {
                    return Err(format!(
                        "Error creating file at {:?}: {:?}",
                        file_path.display(),
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

            println!("File saved successfully: {:?}", file_path);
        }
    }

    Ok("Successfully saved file!".to_string())
}

pub(crate) async fn read_file_from_any_directory(
    user_name: &str,
    path: &str,
    filename: &str
) -> Result<(Vec<u8>, String), String> {

    // Decode parameters
    let decoded_path = match percent_decode_str(path).decode_utf8() {
        Ok(decoded) => decoded.into_owned(),
        Err(_) => return Err("Failed to decode 'path' parameter.".parse().unwrap()),
    };

    let decoded_filename = match percent_decode_str(filename).decode_utf8() {
        Ok(decoded) => decoded.into_owned(),
        Err(_) => return Err("Failed to decode 'filename' parameter.".parse().unwrap()),
    };

    // Construct the full file path
    let full_path = Path::new(ROOT_DIR).join(user_name).join(&decoded_path).join(&decoded_filename);

    // Prevent directory traversal attacks
    if full_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err("Invalid file path: directory traversal detected.".parse().unwrap());
    }

    // Read and return the file
    match std::fs::read(&full_path) {
        Ok(contents) => Ok((contents, decoded_filename)),
        Err(_) => Err(format!("File '{}' not found", decoded_filename)),
    }
}
