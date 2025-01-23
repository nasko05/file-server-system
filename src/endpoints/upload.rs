use std::io::Write;
use actix_multipart::Multipart;
use actix_web::{post, web, Error, HttpResponse};
use futures_util::TryStreamExt;
use crate::ROOT_DIR;
mod file_utilities;
#[post("/upload")]
async fn upload_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    save_file_to_root_directory(&mut payload, ROOT_DIR).await;
    Ok(HttpResponse::Ok().body("File uploaded successfully."))
}

#[post("/upload/{user_uuid}")]
async fn upload_file_user_uuid(user_uuid: web::Path<String>, mut payload: Multipart)
    -> Result<HttpResponse, Error> {

    if() {
        // Ensure the upload directory exists
        std::fs::create_dir_all(ROOT_DIR)?;
    }
    save_file_to_root_directory(&mut payload, user_uuid).await;
}
