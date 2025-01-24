use std::path::Path;
use actix_multipart::Multipart;
use actix_web::{post, web, Error, HttpResponse};
use crate::ROOT_DIR;
use crate::utilities::file_utilities::save_file_to_root_directory;

#[post("/upload")]
async fn upload_file_from_root_directory(mut payload: Multipart) -> Result<HttpResponse, Error> {
    save_file_to_root_directory(&mut payload, ROOT_DIR).await;
    Ok(HttpResponse::Ok().body("File uploaded successfully."))
}

#[post("/upload/{user_uuid}")]
async fn upload_file_from_user_directory(user_uuid: web::Path<String>, mut payload: Multipart)
    -> Result<HttpResponse, Error> {
    let user_path = format!("{}/{}", ROOT_DIR, user_uuid);
    if Path::new(&user_path).exists() {
        // Ensure the upload directory exists
        std::fs::create_dir_all(ROOT_DIR)?;
    }
    save_file_to_root_directory(&mut payload, user_uuid.as_str()).await;
    Ok(HttpResponse::Ok().body("File uploaded successfully."))
}
