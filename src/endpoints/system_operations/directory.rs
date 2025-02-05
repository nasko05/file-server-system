use actix_web::{post, web, HttpResponse, Responder};
use actix_web::http::StatusCode;
use crate::app_config::AppConfig;
use crate::models::authentication::auth_user::AuthenticatedUser;
use crate::models::file_structure::directory_create_request::DirectoryCreateRequest;
use crate::services::file_structure::directory_service::DirectoryService;

#[post("/directory/create")]
pub async fn create_directory(
    payload: web::Json<DirectoryCreateRequest>,
    auth_user: AuthenticatedUser,
    config: web::Data<AppConfig>
) -> impl Responder {
    let path = &payload.path;
    let name = &payload.name;
    let user = &auth_user.0.sub;
    let root = config.root_dir.as_ref();
    
    let directory_service = DirectoryService::new(root.clone());
    
    match directory_service.create_directory(user, path, name).await {  
        Ok(msg) => HttpResponse::Ok().body(msg),
        Err((code, m)) => HttpResponse::build(StatusCode::from_u16(code).unwrap()).body(m)
    }
}