use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
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
        let encoded_filename = utf8_percent_encode(&file.filename, NON_ALPHANUMERIC).to_string();
        let url = format!(
            "/api/v1/file/download/{}/{}",
            file.file_id, encoded_filename
        );
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

#[cfg(test)]
mod tests {
    use super::{File, FileResponse};

    #[test]
    fn file_response_encodes_filename_in_url() {
        let file = File {
            file_id: "f1".to_string(),
            owner_id: "b1".to_string(),
            content_hash: "h1".to_string(),
            filename: "a b.txt".to_string(),
            size: 3,
            mime_type: "text/plain".to_string(),
            storage_path: "/tmp/a b.txt".to_string(),
            created_at: "now".to_string(),
        };

        let resp = FileResponse::from(file);
        assert_eq!(resp.file_id, "f1");
        assert_eq!(resp.url, "/api/v1/file/download/f1/a%20b%2Etxt");
    }
}
