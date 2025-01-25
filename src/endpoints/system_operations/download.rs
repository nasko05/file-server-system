use crate::services::file_structure::file_utilities::read_file_from_any_directory;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use log::info;
use crate::models::system_operations::download_file_request::DownloadFileRequest;

#[post("/download")]
pub async fn download_file_from_user_directory(payload: web::Json<DownloadFileRequest>) -> impl Responder {
    let username = payload.user_id.as_str();
    let path = payload.path_to_file.as_str();
    let filename = payload.filename.as_str();

    match read_file_from_any_directory(username, path, filename).await {
        Ok((content, decoded_filename)) => {
            info!("Successfully downloaded: {}", decoded_filename);

            HttpResponse::Ok()
            .header("Content-Disposition", format!("attachment; filename=\"{}\"", decoded_filename))
            .body(content)
        },
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}