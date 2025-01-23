use actix_web::{get, HttpRequest, HttpResponse, Responder};
use crate::ROOT_DIR;

#[get("/download/{filename:.*}")]
async fn download_file(req: HttpRequest) -> impl Responder {
    // Extract the raw query string, e.g. "filename=example.txt"
    let query_str = req.query_string();
    // For simplicity, we’ll just parse manually:
    let parts: Vec<&str> = query_str.split('=').collect();
    if parts.len() != 2 || parts[0] != "filename" {
        return HttpResponse::BadRequest().body("Missing or invalid 'filename' parameter.");
    }

    let filename = parts[1];
    let filepath = format!("{}/{}", ROOT_DIR, filename);

    // Attempt to read the file from disk
    match std::fs::read(&filepath) {
        Ok(contents) => {
            // In a real server, you might set Content-Type based on file type
            // e.g., using the `mime_guess` crate.
            HttpResponse::Ok().body(contents)
        }
        Err(_) => {
            HttpResponse::NotFound().body(format!("File '{}' not found", filename))
        }
    }
}
