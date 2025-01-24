use std::path::PathBuf;
use actix_web::{error, get, web, HttpResponse, Responder};
use crate::models::auth_user::AuthenticatedUser;
use crate::services::directory_service::build_dir_tree;

#[get("/directory/{dir_name}")]
async fn get_user_directory(
    path: web::Path<(String,)>,
    auth_user: AuthenticatedUser,
) -> impl Responder {
    let dir_name = &path.0;
    let claims = auth_user.0; // 'claims' is the decoded JWT data

    // TODO: check privileges
    // Compare the route param to the user's token role
    if *dir_name != claims.sub && claims.sub != "admin" {
        // If they don't match, return 403
        return Err(error::ErrorForbidden(format!(
            "Your token role is '{}', but you tried to access '{}'",
            claims.sub, dir_name
        )));
    }

    // Build a path to: "root/<user>"
    let user_path = PathBuf::from("root").join(dir_name);

    // For safety, you might want to ensure the user can’t escape "root"
    // e.g., check that user does not contain ".." or something.

    match build_dir_tree(&user_path) {
        Ok(tree) => Ok(HttpResponse::Ok().json(tree)), // Return JSON
        Err(err) => {
            // e.g., if directory doesn’t exist, or we lack permissions
            Ok(HttpResponse::NotFound().body(format!("Error reading directory: {}", err)))
        }
    }
}