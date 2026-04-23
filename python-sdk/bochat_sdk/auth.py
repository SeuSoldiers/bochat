from __future__ import annotations

from .client import AuthKind, BochatClient
from .models import (
    AuthResponse,
    LoginRequest,
    RegisterRequest,
    UpdateProfileRequest,
    UserProfile,
)


class AuthApi:
    def __init__(self, client: BochatClient):
        self._client = client

    def register(self) -> "RegisterBuilder":
        return RegisterBuilder(self._client)

    def login(self) -> "LoginBuilder":
        return LoginBuilder(self._client)

    async def me(self) -> UserProfile:
        data = await self._client._get_json("/api/v1/users/me", AuthKind.USER)
        return UserProfile.from_dict(data)

    async def update_profile(self, req: UpdateProfileRequest) -> UserProfile:
        data = await self._client._request_json(
            "PUT", "/api/v1/users/me", AuthKind.USER, req
        )
        return UserProfile.from_dict(data)

    async def delete_account(self) -> None:
        await self._client._request_empty("DELETE", "/api/v1/users/delete", AuthKind.USER)


class RegisterBuilder:
    def __init__(self, client: BochatClient):
        self._client = client
        self._account: str | None = None
        self._password: str | None = None
        self._nickname: str | None = None

    def account(self, account: str) -> "RegisterBuilder":
        self._account = account
        return self

    def password(self, password: str) -> "RegisterBuilder":
        self._password = password
        return self

    def nickname(self, nickname: str) -> "RegisterBuilder":
        self._nickname = nickname
        return self

    async def send(self) -> AuthResponse:
        req = RegisterRequest(
            account=self._account or "",
            password=self._password or "",
            name=self._nickname,
        )
        data = await self._client._request_json(
            "POST", "/api/v1/auth/register", AuthKind.NONE, req
        )
        resp = AuthResponse.from_dict(data)
        self._client.set_user_token(resp.token)
        return resp


class LoginBuilder:
    def __init__(self, client: BochatClient):
        self._client = client
        self._account: str | None = None
        self._password: str | None = None

    def account(self, account: str) -> "LoginBuilder":
        self._account = account
        return self

    def password(self, password: str) -> "LoginBuilder":
        self._password = password
        return self

    async def send(self) -> AuthResponse:
        req = LoginRequest(account=self._account or "", password=self._password or "")
        data = await self._client._request_json(
            "POST", "/api/v1/auth/login", AuthKind.NONE, req
        )
        resp = AuthResponse.from_dict(data)
        self._client.set_user_token(resp.token)
        return resp
