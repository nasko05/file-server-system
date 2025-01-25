use crate::utilities::file_utilities::read_file_from_directory;
use actix_web::{get, web, HttpRequest, Responder};

#[get("/download")]
pub async fn download_file_from_root_directory(req: HttpRequest) -> impl Responder {
    read_file_from_directory(req, "").await
}

#[get("/download/{user_uuid}")]
pub async fn download_file_from_user_directory(user_uuid: web::Path<String>, req: HttpRequest) -> impl Responder {
    read_file_from_directory(req, user_uuid.as_str()).await
}