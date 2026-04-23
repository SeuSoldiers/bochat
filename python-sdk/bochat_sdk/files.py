from __future__ import annotations

from pathlib import Path
from urllib.parse import quote

from .client import AuthKind, BochatClient
from .models import UploadedFile


class FilesApi:
    def __init__(self, client: BochatClient):
        self._client = client

    async def upload_bytes(
        self,
        filename: str,
        data: bytes,
        mime: str | None = None,
    ) -> UploadedFile:
        file_field = (filename, data, mime) if mime else (filename, data)
        resp = await self._client._request_multipart(
            "/api/v1/file/upload",
            AuthKind.BOT,
            files={"file": file_field},
        )
        return UploadedFile.from_dict(resp)

    async def upload_path(self, path: str | Path, mime: str | None = None) -> UploadedFile:
        path_obj = Path(path)
        filename = path_obj.name or "upload.bin"
        data = path_obj.read_bytes()
        return await self.upload_bytes(filename, data, mime=mime)

    def download_url(self, file_id: str, filename: str) -> str:
        escaped = quote(filename)
        return f"{self._client.base_url()}/api/v1/file/download/{file_id}/{escaped}"

    async def delete(self, file_id: str) -> None:
        await self._client._request_empty(
            "DELETE",
            f"/api/v1/file/{file_id}",
            AuthKind.BOT,
        )
