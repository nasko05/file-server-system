use std::path::PathBuf;
use actix_web::{get, web, HttpResponse, Responder};
use crate::services::directory_service::build_dir_tree;

#[get("/directory/{user}")]
async fn get_user_directory(path: web::Path<(String,)>) -> impl Responder {
    let user = &path.0;

    // Build a path to: "root/<user>"
    let user_path = PathBuf::from("root").join(user);

    // For safety, you might want to ensure the user can’t escape "root"
    // e.g., check that user does not contain ".." or something.

    match build_dir_tree(&user_path) {
        Ok(tree) => HttpResponse::Ok().json(tree), // Return JSON
        Err(err) => {
            // e.g., if directory doesn’t exist, or we lack permissions
            HttpResponse::NotFound().body(format!("Error reading directory: {}", err))
        }
    }
}