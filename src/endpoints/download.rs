use crate::utilities::file_utilities::read_file_from_any_directory;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use log::info;

#[get("/download/{user_uuid}")]
pub async fn download_file_from_user_directory(user_uuid: web::Path<String>, req: HttpRequest) -> impl Responder {
    let query_str = req.query_string();
    let parts: Vec<&str> = query_str.split('&').collect();

    // Ensure query contains both "path" and "filename" parameters
    let mut path = None;
    let mut filename = None;
    for part in parts {
        let key_value: Vec<&str> = part.split('=').collect();
        if key_value.len() == 2 {
            match key_value[0] {
                "path" => path = Some(key_value[1]),
                "filename" => filename = Some(key_value[1]),
                _ => {}
            }
        }
    }

    if path == None {
        return HttpResponse::BadRequest().body("Path not found");
    } else if filename == None {
        return HttpResponse::BadRequest().body("Filename not found");
    }

    match read_file_from_any_directory(user_uuid.as_str(), path.unwrap(), filename.unwrap()).await {
        Ok((content, decoded_filename)) => {
            info!("Successfully downloaded: {}", decoded_filename);

            HttpResponse::Ok()
            .header("Content-Disposition", format!("attachment; filename=\"{}\"", decoded_filename))
            .body(content)
        },
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}