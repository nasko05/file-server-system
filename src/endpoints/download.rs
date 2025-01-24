use actix_web::{get, HttpRequest, HttpResponse, Responder};
use log::info;
use crate::ROOT_DIR;
use crate::utilities::file_utilities::read_file_from_directory;

#[get("/download")]
pub async fn download_file_from_root_directory(req: HttpRequest) -> impl Responder {
    read_file_from_directory(req, ROOT_DIR).await;
}

#[get("/download")]
pub async fn download_file_from_user_directory(req: HttpRequest) -> impl Responder {
    read_file_from_directory(req, ROOT_DIR).await;
}