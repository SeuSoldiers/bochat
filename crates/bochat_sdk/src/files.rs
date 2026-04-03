use reqwest::multipart::{Form, Part};

use crate::client::{AuthKind, BochatClient};
use crate::error::{SdkError, SdkResult};
use crate::models::UploadedFile;

#[derive(Clone)]
pub struct FilesApi {
    client: BochatClient,
}

impl FilesApi {
    pub(crate) fn new(client: BochatClient) -> Self {
        Self { client }
    }

    pub async fn upload_bytes(
        &self,
        filename: impl Into<String>,
        bytes: Vec<u8>,
        mime: Option<&str>,
    ) -> SdkResult<UploadedFile> {
        let filename = filename.into();
        let mut part = Part::bytes(bytes).file_name(filename);
        if let Some(mime) = mime {
            part = part
                .mime_str(mime)
                .map_err(|e| SdkError::RequestBuild(e.to_string()))?;
        }

        let form = Form::new().part("file", part);
        self.client
            .request_multipart("/api/v1/file/upload", AuthKind::Bot, form)
            .await
    }

    pub async fn upload_path(
        &self,
        path: impl AsRef<std::path::Path>,
        mime: Option<&str>,
    ) -> SdkResult<UploadedFile> {
        let path_ref = path.as_ref();
        let filename = path_ref
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "upload.bin".to_string());

        let bytes = tokio::fs::read(path_ref)
            .await
            .map_err(|e| SdkError::Transport(e.to_string()))?;

        self.upload_bytes(filename, bytes, mime).await
    }

    pub fn download_url(&self, file_id: &str) -> String {
        format!("{}/api/v1/file/download/{}", self.client.base_url(), file_id)
    }
}
