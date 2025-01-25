use crate::services::file_structure::file_utilities::save_file_to_root_directory;
use crate::ROOT_DIR;
use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse, Responder};
use std::path::Path;

#[post("/upload")]
async fn upload_file_from_root_directory(mut payload: Multipart) -> impl Responder {
    match save_file_to_root_directory(&mut payload, "").await {
        Ok(success) => HttpResponse::Ok().body(success),
        Err(err) => HttpResponse::NotFound().body(err),
    }
}

#[post("/upload/{user_uuid}")]
async fn upload_file_from_user_directory(
    user_uuid: web::Path<String>,
    mut payload: Multipart
) -> impl Responder {
    let user_path = format!("{}/{}", ROOT_DIR, user_uuid);
    if !Path::new(&user_path).exists() {
        // Ensure the upload directory exists
        match std::fs::create_dir_all(&user_path) {
            Ok(_) => (),
            Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        }
    }
    match save_file_to_root_directory(&mut payload, user_uuid.as_str()).await {
        Ok(success) => HttpResponse::Ok().body(success),
        Err(err) => HttpResponse::NotFound().body(err),
    }
}
