from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass
class ApiErrorResponse:
    code: str
    message: str
    status: int


@dataclass
class AuthResponse:
    message: str
    name: str
    token: str
    account: str | None = None
    created_at: str | None = None

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "AuthResponse":
        return cls(
            message=str(data.get("message", "")),
            name=str(data.get("name", "")),
            token=str(data.get("token", "")),
            account=data.get("account"),
            created_at=data.get("created_at"),
        )


@dataclass
class UserProfile:
    name: str
    avatar_url: str | None = None
    created_at: str | None = None
    updated_at: str | None = None

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "UserProfile":
        return cls(
            name=str(data.get("name", "")),
            avatar_url=data.get("avatar_url"),
            created_at=data.get("created_at"),
            updated_at=data.get("updated_at"),
        )


@dataclass
class BotInfo:
    bot_id: str
    owner_id: str
    name: str
    description: str | None
    avatar_url: str | None
    status: str
    token: str
    created_at: str
    updated_at: str

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "BotInfo":
        return cls(
            bot_id=str(data.get("bot_id", "")),
            owner_id=str(data.get("owner_id", "")),
            name=str(data.get("name", "")),
            description=data.get("description"),
            avatar_url=data.get("avatar_url"),
            status=str(data.get("status", "")),
            token=str(data.get("token", "")),
            created_at=str(data.get("created_at", "")),
            updated_at=str(data.get("updated_at", "")),
        )


@dataclass
class BotListResponse:
    bots: list[BotInfo]

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "BotListResponse":
        bots = [BotInfo.from_dict(x) for x in data.get("bots", [])]
        return cls(bots=bots)


@dataclass
class CreateBotRequest:
    name: str
    description: str | None = None
    avatar_url: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "avatar_url": self.avatar_url,
        }


@dataclass
class UpdateBotRequest:
    name: str
    description: str | None = None
    avatar_url: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "avatar_url": self.avatar_url,
        }


@dataclass
class CreateGroupRequest:
    name: str
    description: str | None = None
    group_code: str | None = None
    bot_id: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "group_code": self.group_code,
            "bot_id": self.bot_id,
        }


@dataclass
class GroupInfo:
    group_id: str
    group_code: str | None
    creator_id: str
    name: str
    description: str | None
    status: str
    created_at: str
    updated_at: str

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "GroupInfo":
        return cls(
            group_id=str(data.get("group_id", "")),
            group_code=data.get("group_code"),
            creator_id=str(data.get("creator_id", "")),
            name=str(data.get("name", "")),
            description=data.get("description"),
            status=str(data.get("status", "")),
            created_at=str(data.get("created_at", "")),
            updated_at=str(data.get("updated_at", "")),
        )


@dataclass
class MessageContent:
    value: dict[str, Any]

    @classmethod
    def text(cls, text: str) -> "MessageContent":
        return cls({"text": text})

    @classmethod
    def file(cls, url: str) -> "MessageContent":
        return cls({"url": url})

    @classmethod
    def custom(cls, value: dict[str, Any]) -> "MessageContent":
        return cls(value)

    @classmethod
    def from_raw(cls, value: Any) -> "MessageContent":
        if isinstance(value, dict):
            return cls(value)
        return cls({"value": value})

    def as_text(self) -> str | None:
        value = self.value.get("text")
        return value if isinstance(value, str) else None

    def as_file_url(self) -> str | None:
        value = self.value.get("url")
        return value if isinstance(value, str) else None

    def to_dict(self) -> dict[str, Any]:
        return dict(self.value)


@dataclass
class SendMessageRequest:
    group_id: str
    content: MessageContent
    msg_type: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "group_id": self.group_id,
            "content": self.content.to_dict(),
            "msg_type": self.msg_type,
        }


@dataclass
class MessageResponse:
    msg_id: int
    group_id: str
    sender_id: str
    sender_name: str | None
    sender_avatar_url: str | None
    content: MessageContent
    msg_type: str
    created_at: str

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "MessageResponse":
        return cls(
            msg_id=int(data.get("msg_id", 0)),
            group_id=str(data.get("group_id", "")),
            sender_id=str(data.get("sender_id", "")),
            sender_name=data.get("sender_name"),
            sender_avatar_url=data.get("sender_avatar_url"),
            content=MessageContent.from_raw(data.get("content", {})),
            msg_type=str(data.get("msg_type", "")),
            created_at=str(data.get("created_at", "")),
        )


@dataclass
class GroupHistoryResponse:
    group_id: str
    base_id: int | None
    limit: int
    next_base_id: int | None
    messages: list[MessageResponse]

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "GroupHistoryResponse":
        return cls(
            group_id=str(data.get("group_id", "")),
            base_id=data.get("base_id"),
            limit=int(data.get("limit", 0)),
            next_base_id=data.get("next_base_id"),
            messages=[MessageResponse.from_dict(x) for x in data.get("messages", [])],
        )


@dataclass
class UploadedFile:
    file_id: str
    filename: str
    url: str
    created_at: str

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "UploadedFile":
        return cls(
            file_id=str(data.get("file_id", "")),
            filename=str(data.get("filename", "")),
            url=str(data.get("url", "")),
            created_at=str(data.get("created_at", "")),
        )


@dataclass
class RegisterRequest:
    account: str
    password: str
    name: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {"account": self.account, "password": self.password, "name": self.name}


@dataclass
class LoginRequest:
    account: str
    password: str

    def to_dict(self) -> dict[str, Any]:
        return {"account": self.account, "password": self.password}


@dataclass
class UpdateProfileRequest:
    name: str | None = None
    password: str | None = None
    avatar_url: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "password": self.password,
            "avatar_url": self.avatar_url,
        }


@dataclass
class WsConnectionPayload:
    bot_id: str
    bot_name: str
    group_ids: list[str]

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "WsConnectionPayload":
        return cls(
            bot_id=str(data.get("bot_id", "")),
            bot_name=str(data.get("bot_name", "")),
            group_ids=[str(x) for x in data.get("group_ids", [])],
        )


@dataclass
class WsEvent:
    event_type: str
    payload: dict[str, Any]
    timestamp: str

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "WsEvent":
        return cls(
            event_type=str(data.get("type", "")),
            payload=data.get("payload", {}) if isinstance(data.get("payload"), dict) else {},
            timestamp=str(data.get("timestamp", "")),
        )

    def group_id(self) -> str | None:
        group_id = self.payload.get("group_id")
        return group_id if isinstance(group_id, str) else None

    def as_connection_payload(self) -> WsConnectionPayload | None:
        if self.event_type != "connection":
            return None
        return WsConnectionPayload.from_dict(self.payload)

    def as_message_payload(self) -> MessageResponse | None:
        if self.event_type != "message":
            return None
        return MessageResponse.from_dict(self.payload)
