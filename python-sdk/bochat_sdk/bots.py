from __future__ import annotations

from .client import AuthKind, BochatClient
from .error import ApiError
from .models import BotInfo, BotListResponse, CreateBotRequest, UpdateBotRequest


class BotsApi:
    def __init__(self, client: BochatClient):
        self._client = client

    async def list(self) -> list[BotInfo]:
        data = await self._client._get_json("/api/v1/bots", AuthKind.USER)
        return BotListResponse.from_dict(data).bots

    async def create(self, req: CreateBotRequest) -> BotInfo:
        data = await self._client._request_json("POST", "/api/v1/bots", AuthKind.USER, req)
        return BotInfo.from_dict(data)

    async def get(self, bot_id: str) -> BotInfo:
        data = await self._client._get_json(f"/api/v1/bots/{bot_id}", AuthKind.NONE)
        return BotInfo.from_dict(data)

    async def update(self, bot_id: str, req: UpdateBotRequest) -> BotInfo:
        data = await self._client._request_json(
            "PUT", f"/api/v1/bots/{bot_id}", AuthKind.USER, req
        )
        return BotInfo.from_dict(data)

    async def delete(self, bot_id: str) -> None:
        await self._client._request_empty("DELETE", f"/api/v1/bots/{bot_id}", AuthKind.USER)

    async def use_bot_token(self, bot_id: str | None = None) -> str:
        bots = await self.list()
        chosen = None
        if bot_id:
            for bot in bots:
                if bot.bot_id == bot_id:
                    chosen = bot
                    break
        else:
            for bot in bots:
                if bot.status == "active":
                    chosen = bot
                    break
        if not chosen:
            raise ApiError(
                code="no_available_bot",
                message="没有可用的 Bot 可设置 token",
                status=400,
            )
        self._client.set_bot_token(chosen.token)
        return chosen.token
