from .error import (
    ApiError,
    HttpStatusError,
    InvalidUrl,
    MissingBotToken,
    MissingUserToken,
    RequestBuildError,
    SdkError,
    SerdeError,
    TransportError,
    WebSocketError,
)
from .models import (
    ApiErrorResponse,
    AuthResponse,
    BotInfo,
    BotListResponse,
    CreateBotRequest,
    CreateGroupRequest,
    GroupHistoryResponse,
    GroupInfo,
    LoginRequest,
    MessageContent,
    MessageResponse,
    RegisterRequest,
    SendMessageRequest,
    UpdateBotRequest,
    UpdateProfileRequest,
    UploadedFile,
    UserProfile,
    WsConnectionPayload,
    WsEvent,
)
from .retry import RetryPolicy
from .ws import WsDispatcher, WsSession, WsSessionBuilder, WsSessionHandle

try:
    from .auth import AuthApi
    from .bots import BotsApi
    from .client import BochatClient, BochatClientBuilder
    from .files import FilesApi
    from .groups import GroupsApi
    from .messages import MessagesApi
except ModuleNotFoundError as exc:
    if exc.name != "httpx":
        raise

__all__ = [
    "ApiError",
    "ApiErrorResponse",
    "AuthApi",
    "AuthResponse",
    "BochatClient",
    "BochatClientBuilder",
    "BotInfo",
    "BotListResponse",
    "BotsApi",
    "CreateBotRequest",
    "CreateGroupRequest",
    "FilesApi",
    "GroupHistoryResponse",
    "GroupInfo",
    "GroupsApi",
    "HttpStatusError",
    "InvalidUrl",
    "LoginRequest",
    "MessageContent",
    "MessageResponse",
    "MessagesApi",
    "MissingBotToken",
    "MissingUserToken",
    "RegisterRequest",
    "RequestBuildError",
    "RetryPolicy",
    "SdkError",
    "SendMessageRequest",
    "SerdeError",
    "TransportError",
    "UpdateBotRequest",
    "UpdateProfileRequest",
    "UploadedFile",
    "UserProfile",
    "WebSocketError",
    "WsDispatcher",
    "WsConnectionPayload",
    "WsEvent",
    "WsSession",
    "WsSessionBuilder",
    "WsSessionHandle",
]
