pub struct UploadFileRequest {
    pub file: Option<(String, Vec<u8>)>,
    pub path: Option<String>,
}