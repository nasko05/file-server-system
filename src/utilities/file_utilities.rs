use std::ffi::CString;
use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Responder};
use futures_util::TryStreamExt;
use log::info;
use tokio::io::AsyncWriteExt;
use crate::ROOT_DIR;

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| *c != '/' && *c != '\\')
        .collect()
}

pub(crate) async fn save_file_to_root_directory(
    payload: &mut Multipart,
    user_directory: &str
) -> bool {
    while let Ok(Some(item)) = payload.try_next().await {
        let mut field = item;
        let content_disposition = field.content_disposition();

        if let Some(filename) = content_disposition.get_filename() {
            let filename = sanitize_filename(filename).as_str();
            let filepath = join_user_directory(user_directory, filename);

            // Create file asynchronously
            let mut f = match tokio::fs::File::create(&filepath).await {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error creating file {}: {:?}", filepath, e);
                    return false;
                }
            };

            // Write chunks asynchronously
            while let Ok(Some(chunk)) = field.try_next().await {
                if let Err(e) = f.write_all(&chunk).await {
                    eprintln!("Error writing chunk: {:?}", e);
                    return false;
                }
            }
        }
    }

    true
}

pub(crate) async fn read_file_from_directory(
    req: HttpRequest,
    user_directory: &str
) -> impl Responder {
    let query_str = req.query_string();
    let parts: Vec<&str> = query_str.split('=').collect();
    if parts.len() != 2 || parts[0] != "filename" {
        return HttpResponse::BadRequest().body("Missing or invalid 'filename' parameter.");
    }

    let filename = parts[1];
    let filepath = join_user_directory(user_directory, filename);
    info!("This is the resulting path: {}", filepath);
    match std::fs::read(&filepath) {
        Ok(contents) => HttpResponse::Ok().body(contents),
        Err(_) => HttpResponse::NotFound().body(format!("File '{}' not found", filename)),
    }
}

fn join_user_directory(user_directory: &str, filename: &str) -> String {
    if user_directory.is_empty() {
        format!("{}/{}", ROOT_DIR, filename)
    } else {
        format!("{}/{}/{}", ROOT_DIR, user_directory, filename)
    }
}