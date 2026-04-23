from __future__ import annotations

from .client import AuthKind, BochatClient
from .models import CreateGroupRequest, GroupInfo


class GroupsApi:
    def __init__(self, client: BochatClient):
        self._client = client

    async def create(self, req: CreateGroupRequest) -> GroupInfo:
        data = await self._client._request_json("POST", "/api/v1/groups", AuthKind.USER, req)
        return GroupInfo.from_dict(data)

    async def delete(self, group_id: str) -> None:
        await self._client._request_empty(
            "DELETE",
            f"/api/v1/groups/{group_id}",
            AuthKind.USER,
        )
