use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
// Import the CORS middleware
use dotenv::dotenv;
use models::authentication;
use std::sync::Arc;
use crate::app_config::AppConfig;
extern crate env_logger;
use crate::endpoints::authentication::authentication::{login_handler, protected_resource_handler};
use crate::endpoints::system_operations::delete::{delete_file, delete_user_directory};
use crate::endpoints::system_operations::directory::create_directory;
use crate::endpoints::system_operations::download::download_file_from_user_directory;
use crate::endpoints::system_operations::get_file_structure::get_user_directory;
use crate::endpoints::system_operations::rename::rename_directory;
use crate::endpoints::system_operations::upload::{upload_file_from_user_directory};

static ROOT_DIR: &str = "./root";
pub mod endpoints;
mod services;
mod models;
mod dao;
mod tests;
mod app_config;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Ensure the upload directory exists
    env_logger::init();
    std::fs::create_dir_all(ROOT_DIR)?;
    dotenv().ok();
    let root_dir = std::env::var("ROOT_DIR").unwrap_or_else(|_| "./root".to_string());
    let config = AppConfig { root_dir: Arc::new(root_dir) };

    println!("Server running on http://0.0.0.0:8080");

    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173") // Allow only this origin
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"]) // Allowed HTTP methods
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::AUTHORIZATION,
            ]) // Allow specific headers
            .supports_credentials(); // Allow cookies or authorization headers

        App::new()
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .wrap(cors) // Add the CORS middleware
            .service(login_handler)
            .service(
                web::scope("/api")
                    .wrap(authentication::auth_models::JwtAuth)
                    .service(web::resource("/protected").route(web::get().to(protected_resource_handler)))
                    .service(download_file_from_user_directory)
                    .service(upload_file_from_user_directory)
                    .service(get_user_directory)
                    .service(delete_user_directory)
                    .service(delete_file)
                    .service(rename_directory)
                    .service(create_directory),
            )
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}