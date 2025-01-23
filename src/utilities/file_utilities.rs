use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use tokio::io::AsyncWriteExt;

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| *c != '/' && *c != '\\')
        .collect()
}

pub(crate) async fn save_file_to_root_directory(
    payload: &mut Multipart,
    root_directory: &str
) -> bool {
    while let Ok(Some(item)) = payload.try_next().await {
        let mut field = item;
        let content_disposition = field.content_disposition();

        if let Some(filename) = content_disposition.get_filename() {
            let filename = sanitize_filename(filename);
            let filepath = format!("{}/{}", root_directory, filename);

            // Create file asynchronously
            let mut f = match tokio::fs::File::create(&filepath).await {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error creating file {}: {:?}", filepath, e);
                    return false;
                }
            };

            // Write chunks asynchronously
            while let Ok(Some(chunk)) = field.try_next().await {
                if let Err(e) = f.write_all(&chunk).await {
                    eprintln!("Error writing chunk: {:?}", e);
                    return false;
                }
            }
        }
    }

    true
}