use std::io::Write;
use actix_multipart::Multipart;
use futures_util::TryStreamExt;

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| *c != '/' && *c != '\\')
        .collect()
}

pub(crate) async fn save_file_to_root_directory(payload: &mut Multipart, root_directory: &str) -> bool {

    // Process each field in the multipart form
    while let Some(item) = payload.try_next().await {
        let mut field = item;
        let content_disposition = field.content_disposition();

        // Check if the form field is named "file" and has a filename
        if let Some(filename) = content_disposition.get_filename() {
            // Sanitize the filename to avoid path traversal
            let filename = sanitize_filename(filename);
            let filepath = format!("{}/{}", root_directory, filename);

            // Create or overwrite the file
            let mut f = std::fs::File::create(&filepath)?;

            // Write the chunks to the file
            while let Some(chunk) = field.try_next().await? {
                f.write_all(&chunk)?;
            }
        }
    }
    true
}