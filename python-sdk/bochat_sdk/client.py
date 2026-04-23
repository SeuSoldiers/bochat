from __future__ import annotations

import asyncio
from dataclasses import asdict, is_dataclass
from enum import Enum
from typing import Any
from urllib.parse import urlparse

import httpx

from .error import (
    HttpStatusError,
    InvalidUrl,
    MissingBotToken,
    MissingUserToken,
    RequestBuildError,
    SerdeError,
    TransportError,
    parse_api_error,
)
from .retry import RetryPolicy


class AuthKind(str, Enum):
    NONE = "none"
    USER = "user"
    BOT = "bot"


class BochatClientBuilder:
    def __init__(self, base_url: str):
        self._base_url = base_url
        self._timeout_secs = 15.0
        self._user_agent = "bochat-sdk/0.1.0"
        self._user_token: str | None = None
        self._bot_token: str | None = None
        self._retry_policy = RetryPolicy()

    def timeout_secs(self, timeout_secs: int | float) -> "BochatClientBuilder":
        self._timeout_secs = float(timeout_secs)
        return self

    def user_agent(self, user_agent: str) -> "BochatClientBuilder":
        self._user_agent = user_agent
        return self

    def user_token(self, token: str) -> "BochatClientBuilder":
        self._user_token = token
        return self

    def bot_token(self, token: str) -> "BochatClientBuilder":
        self._bot_token = token
        return self

    def retry_policy(self, retry_policy: RetryPolicy) -> "BochatClientBuilder":
        self._retry_policy = retry_policy
        return self

    def build(self) -> "BochatClient":
        return BochatClient(
            base_url=self._base_url,
            timeout_secs=self._timeout_secs,
            user_agent=self._user_agent,
            user_token=self._user_token,
            bot_token=self._bot_token,
            retry_policy=self._retry_policy,
        )


class BochatClient:
    def __init__(
        self,
        base_url: str,
        timeout_secs: float = 15.0,
        user_agent: str = "bochat-sdk/0.1.0",
        user_token: str | None = None,
        bot_token: str | None = None,
        retry_policy: RetryPolicy | None = None,
    ):
        self._base_url = base_url.rstrip("/")
        self._user_token = user_token
        self._bot_token = bot_token
        self._retry_policy = retry_policy or RetryPolicy()
        try:
            self._http = httpx.AsyncClient(
                timeout=timeout_secs,
                headers={"User-Agent": user_agent},
            )
        except Exception as exc:
            raise RequestBuildError(str(exc)) from exc

    @staticmethod
    def builder(base_url: str) -> BochatClientBuilder:
        return BochatClientBuilder(base_url)

    def auth(self):
        from .auth import AuthApi

        return AuthApi(self)

    def bots(self):
        from .bots import BotsApi

        return BotsApi(self)

    def messages(self):
        from .messages import MessagesApi

        return MessagesApi(self)

    def groups(self):
        from .groups import GroupsApi

        return GroupsApi(self)

    def files(self):
        from .files import FilesApi

        return FilesApi(self)

    def ws(self):
        from .ws import WsSessionBuilder

        return WsSessionBuilder(self)

    async def close(self) -> None:
        await self._http.aclose()

    async def __aenter__(self) -> "BochatClient":
        return self

    async def __aexit__(self, exc_type, exc, tb) -> None:
        await self.close()

    def set_user_token(self, token: str | None) -> None:
        self._user_token = token

    def set_bot_token(self, token: str | None) -> None:
        self._bot_token = token

    def user_token(self) -> str | None:
        return self._user_token

    def bot_token(self) -> str | None:
        return self._bot_token

    def base_url(self) -> str:
        return self._base_url

    def retry_policy(self) -> RetryPolicy:
        return self._retry_policy

    async def _get_json(self, path: str, auth: AuthKind) -> dict[str, Any]:
        url = self._path_to_url(path)
        method = "GET"
        last_error: Exception | None = None

        attempts = max(1, self._retry_policy.max_attempts)
        for attempt in range(attempts):
            try:
                resp = await self._http.request(
                    method, url, headers=self._auth_headers(auth)
                )
            except Exception as exc:
                last_error = TransportError(str(exc))
                if attempt + 1 < attempts:
                    await asyncio.sleep(self._retry_policy.next_delay(attempt))
                    continue
                break

            if 200 <= resp.status_code < 300:
                return self._decode_json(resp)

            if (
                RetryPolicy.should_retry_method(method)
                and RetryPolicy.should_retry_status(resp.status_code)
                and attempt + 1 < attempts
            ):
                await asyncio.sleep(self._retry_policy.next_delay(attempt))
                continue

            raise self._parse_http_error(resp)

        raise last_error or TransportError("请求失败")

    async def _request_json(
        self,
        method: str,
        path: str,
        auth: AuthKind,
        body: Any,
    ) -> dict[str, Any]:
        url = self._path_to_url(path)
        payload = self._json_payload(body)
        try:
            resp = await self._http.request(
                method,
                url,
                headers=self._auth_headers(auth),
                json=payload,
            )
        except Exception as exc:
            raise TransportError(str(exc)) from exc
        return self._parse_json_response(resp)

    async def _request_empty(self, method: str, path: str, auth: AuthKind) -> None:
        url = self._path_to_url(path)
        try:
            resp = await self._http.request(method, url, headers=self._auth_headers(auth))
        except Exception as exc:
            raise TransportError(str(exc)) from exc
        if 200 <= resp.status_code < 300:
            return
        raise self._parse_http_error(resp)

    async def _request_multipart(
        self,
        path: str,
        auth: AuthKind,
        files: dict[str, Any],
    ) -> dict[str, Any]:
        url = self._path_to_url(path)
        try:
            resp = await self._http.post(url, headers=self._auth_headers(auth), files=files)
        except Exception as exc:
            raise TransportError(str(exc)) from exc
        return self._parse_json_response(resp)

    def _parse_json_response(self, resp: httpx.Response) -> dict[str, Any]:
        if 200 <= resp.status_code < 300:
            return self._decode_json(resp)
        raise self._parse_http_error(resp)

    def _decode_json(self, resp: httpx.Response) -> dict[str, Any]:
        try:
            data = resp.json()
        except Exception as exc:
            raise SerdeError(str(exc)) from exc
        if isinstance(data, dict):
            return data
        raise SerdeError("响应 JSON 不是对象")

    def _path_to_url(self, path: str) -> str:
        if path.startswith("http://") or path.startswith("https://"):
            url = path
        elif path.startswith("/"):
            url = f"{self._base_url}{path}"
        else:
            url = f"{self._base_url}/{path}"
        parsed = urlparse(url)
        if not parsed.scheme or not parsed.netloc:
            raise InvalidUrl(url)
        return url

    def _auth_headers(self, auth: AuthKind) -> dict[str, str]:
        if auth == AuthKind.NONE:
            return {}
        if auth == AuthKind.USER:
            if not self._user_token:
                raise MissingUserToken("缺少用户 token，请先登录")
            return {"Authorization": f"Bearer {self._user_token}"}
        if auth == AuthKind.BOT:
            if not self._bot_token:
                raise MissingBotToken("缺少 Bot token，请先选择或设置 Bot")
            return {"Authorization": f"Bearer {self._bot_token}"}
        return {}

    def _parse_http_error(self, resp: httpx.Response):
        body = resp.text
        try:
            data = resp.json()
            if isinstance(data, dict) and {"code", "message", "status"} <= set(data.keys()):
                return parse_api_error(data)
        except Exception:
            pass
        return HttpStatusError(status=resp.status_code, body=body)

    @staticmethod
    def _json_payload(body: Any) -> Any:
        if body is None:
            return None
        if hasattr(body, "to_dict"):
            return body.to_dict()
        if is_dataclass(body):
            return asdict(body)
        return body
