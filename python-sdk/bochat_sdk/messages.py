from __future__ import annotations

import asyncio
import itertools
import os
import time
from urllib.parse import urlencode

from .client import AuthKind, BochatClient
from .error import ApiError, HttpStatusError, SdkError, TransportError
from .models import (
    GroupHistoryResponse,
    MessageContent,
    MessageResponse,
    SendMessageRequest,
)

_MESSAGE_IDEMPOTENCY_SEQ = itertools.count(1)


def _generate_idempotency_key() -> str:
    return f"sdk-{os.getpid()}-{time.time_ns()}-{next(_MESSAGE_IDEMPOTENCY_SEQ)}"


def _should_retry_send_error(err: Exception) -> bool:
    if isinstance(err, TransportError):
        return True
    if isinstance(err, ApiError):
        return err.status >= 500 or err.status == 429
    if isinstance(err, HttpStatusError):
        return err.status >= 500 or err.status == 429
    return False


class MessagesApi:
    def __init__(self, client: BochatClient):
        self._client = client

    async def send(self, req: SendMessageRequest) -> MessageResponse:
        payload = {
            "group_id": req.group_id,
            "content": req.content.to_dict(),
            "msg_type": req.msg_type,
            "idempotency_key": _generate_idempotency_key(),
        }
        return await self._send_with_retry(payload)

    async def send_text(self, group_id: str, text: str) -> MessageResponse:
        req = SendMessageRequest(
            group_id=group_id,
            content=MessageContent.text(text),
            msg_type="text",
        )
        return await self.send(req)

    async def history(
        self,
        group_id: str,
        base_id: int | None = None,
        limit: int | None = None,
    ) -> GroupHistoryResponse:
        path = f"/api/v1/groups/{group_id}/messages"
        query: dict[str, int] = {}
        if base_id is not None:
            query["base_id"] = base_id
        if limit is not None:
            query["limit"] = limit
        if query:
            path = f"{path}?{urlencode(query)}"
        data = await self._client._get_json(path, AuthKind.BOT)
        return GroupHistoryResponse.from_dict(data)

    @staticmethod
    def file_content(url: str) -> MessageContent:
        return MessageContent.file(url)

    async def _send_with_retry(self, payload: dict) -> MessageResponse:
        policy = self._client.retry_policy()
        attempts = max(1, policy.max_attempts)

        for attempt in range(attempts):
            try:
                data = await self._client._request_json(
                    "POST",
                    "/api/v1/message/send",
                    AuthKind.BOT,
                    payload,
                )
                return MessageResponse.from_dict(data)
            except SdkError as err:
                if _should_retry_send_error(err) and attempt + 1 < attempts:
                    await asyncio.sleep(policy.next_delay(attempt))
                    continue
                raise
        raise TransportError("消息发送失败")
