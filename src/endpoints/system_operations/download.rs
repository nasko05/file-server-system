use crate::services::file_structure::file_service::FileService;
use actix_web::{post, web, HttpResponse, Responder};
use log::info;
use crate::app_config::AppConfig;
use crate::models::authentication::auth_user::AuthenticatedUser;
use crate::models::system_operations::download_file_request::DownloadFileRequest;

#[post("/download")]
pub async fn download_file_from_user_directory(
    payload: web::Json<DownloadFileRequest>,
    authenticated_user: AuthenticatedUser,
    config: web::Data<AppConfig>,
) -> impl Responder {
    let username = authenticated_user.0.sub;
    let path = payload.path.as_str();
    let filename = payload.filename.as_str();
    let file_service = FileService::new(config.root_dir.as_ref().clone());

    match file_service.read_file_from_any_directory(&username, path, filename).await {
        Ok((content, decoded_filename)) => {
            info!("Successfully downloaded: {}", decoded_filename);

            HttpResponse::Ok()
            .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", decoded_filename)))
            .body(content)
        },
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}