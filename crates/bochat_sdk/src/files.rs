use reqwest::Method;
use reqwest::multipart::{Form, Part};

use crate::client::{AuthKind, BochatClient};
use crate::error::{SdkError, SdkResult};
use crate::models::UploadedFile;

/// File upload and file URL helper API facade.
///
/// 文件上传与文件 URL 辅助 API 门面。
#[derive(Clone)]
pub struct FilesApi {
    client: BochatClient,
}

impl FilesApi {
    pub(crate) fn new(client: BochatClient) -> Self {
        Self { client }
    }

    /// Upload in-memory bytes as a file.
    ///
    /// 上传内存中的字节数据为文件。
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

    /// Upload a local file from disk.
    ///
    /// 从本地磁盘路径上传文件。
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

    /// Build the public download URL for a file.
    ///
    /// 构造文件的公开下载 URL。
    ///
    /// The current backend requires both `file_id` and `filename` in the path.
    ///
    /// 当前后端要求下载路径中同时带上 `file_id` 和 `filename`。
    pub fn download_url(&self, file_id: &str, filename: &str) -> String {
        format!(
            "{}/api/v1/file/download/{}/{}",
            self.client.base_url(),
            file_id,
            filename
        )
    }

    /// Delete a file uploaded by the current bot.
    ///
    /// 删除当前 Bot 上传过的文件。
    pub async fn delete(&self, file_id: &str) -> SdkResult<()> {
        self.client
            .request_empty(
                Method::DELETE,
                &format!("/api/v1/file/{}", file_id),
                AuthKind::Bot,
            )
            .await
    }
}
