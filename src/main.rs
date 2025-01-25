use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use crate::endpoints::authentication::{login_handler, protected_resource_handler};
use crate::endpoints::download::{download_file_from_root_directory, download_file_from_user_directory};
use crate::endpoints::system_operations::get_user_directory;
use crate::endpoints::upload::{upload_file_from_root_directory, upload_file_from_user_directory};
// Import the CORS middleware
use dotenv::dotenv;

static ROOT_DIR: &str = "./root";
static ROOT: &str = "root";
mod endpoints;
mod utilities;
mod services;
mod models;
mod dao;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Ensure the upload directory exists
    std::fs::create_dir_all(ROOT_DIR)?;
    dotenv().ok();

    println!("Server running on http://0.0.0.0:8080");

    HttpServer::new(|| {
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
            .wrap(Logger::default())
            .wrap(cors) // Add the CORS middleware
            .service(login_handler)
            .service(
                web::scope("/api")
                    .wrap(models::auth_models::JwtAuth)
                    .service(web::resource("/protected").route(web::get().to(protected_resource_handler)))
                    .service(download_file_from_root_directory)
                    .service(download_file_from_user_directory)
                    .service(upload_file_from_root_directory)
                    .service(upload_file_from_user_directory)
                    .service(get_user_directory),
            )
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}