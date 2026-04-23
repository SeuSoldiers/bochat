from __future__ import annotations

from dataclasses import dataclass


class SdkError(Exception):
    """Base SDK error."""


class RequestBuildError(SdkError):
    pass


class TransportError(SdkError):
    pass


@dataclass
class ApiError(SdkError):
    code: str
    message: str
    status: int

    def __str__(self) -> str:
        return f"服务端返回错误: {self.code} - {self.message} (status={self.status})"


@dataclass
class HttpStatusError(SdkError):
    status: int
    body: str

    def __str__(self) -> str:
        return f"HTTP 状态异常: status={self.status}, body={self.body}"


class SerdeError(SdkError):
    pass


class MissingUserToken(SdkError):
    pass


class MissingBotToken(SdkError):
    pass


class InvalidUrl(SdkError):
    pass


class WebSocketError(SdkError):
    pass


def parse_api_error(data: dict) -> ApiError:
    return ApiError(
        code=str(data.get("code", "unknown_error")),
        message=str(data.get("message", "unknown error")),
        status=int(data.get("status", 500)),
    )
