use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct File {
    pub file_id: String,
    pub owner_id: String,
    pub content_hash: String,
    pub filename: String,
    pub size: i64,
    pub mime_type: String,
    pub storage_path: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileResponse {
    pub file_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub created_at: String,
    pub url: String,
}

impl From<File> for FileResponse {
    fn from(file: File) -> Self {
        let url = format!("/api/v1/file/download/{}", file.file_id);
        FileResponse {
            file_id: file.file_id,
            filename: file.filename,
            mime_type: file.mime_type,
            size: file.size,
            created_at: file.created_at,
            url,
        }
    }
}
