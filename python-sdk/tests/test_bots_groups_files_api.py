import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock

from bochat_sdk.bots import BotsApi
from bochat_sdk.client import BochatClient
from bochat_sdk.error import ApiError
from bochat_sdk.files import FilesApi
from bochat_sdk.groups import GroupsApi
from bochat_sdk.models import CreateBotRequest, CreateGroupRequest, UpdateBotRequest


class BotsApiTests(unittest.IsolatedAsyncioTestCase):
    async def test_crud_and_use_token_paths(self):
        client = BochatClient("http://example.com")
        client._get_json = AsyncMock(
            side_effect=[
                {
                    "bots": [
                        {
                            "bot_id": "b1",
                            "owner_id": "u1",
                            "name": "bot1",
                            "description": None,
                            "avatar_url": None,
                            "status": "inactive",
                            "token": "tok-inactive",
                            "created_at": "now",
                            "updated_at": "now",
                        },
                        {
                            "bot_id": "b2",
                            "owner_id": "u1",
                            "name": "bot2",
                            "description": None,
                            "avatar_url": None,
                            "status": "active",
                            "token": "tok-active",
                            "created_at": "now",
                            "updated_at": "now",
                        },
                    ]
                },
                {
                    "bot_id": "b1",
                    "owner_id": "u1",
                    "name": "bot1",
                    "description": None,
                    "avatar_url": None,
                    "status": "active",
                    "token": "tok1",
                    "created_at": "now",
                    "updated_at": "now",
                },
                {
                    "bots": [
                        {
                            "bot_id": "b1",
                            "owner_id": "u1",
                            "name": "bot1",
                            "description": None,
                            "avatar_url": None,
                            "status": "inactive",
                            "token": "tok-inactive",
                            "created_at": "now",
                            "updated_at": "now",
                        },
                        {
                            "bot_id": "b2",
                            "owner_id": "u1",
                            "name": "bot2",
                            "description": None,
                            "avatar_url": None,
                            "status": "active",
                            "token": "tok-active",
                            "created_at": "now",
                            "updated_at": "now",
                        },
                    ]
                },
                {"bots": []},
            ]
        )
        client._request_json = AsyncMock(
            side_effect=[
                {
                    "bot_id": "b3",
                    "owner_id": "u1",
                    "name": "new",
                    "description": "d",
                    "avatar_url": None,
                    "status": "active",
                    "token": "tok3",
                    "created_at": "now",
                    "updated_at": "now",
                },
                {
                    "bot_id": "b1",
                    "owner_id": "u1",
                    "name": "updated",
                    "description": "d2",
                    "avatar_url": None,
                    "status": "active",
                    "token": "tok1",
                    "created_at": "now",
                    "updated_at": "now",
                },
            ]
        )
        client._request_empty = AsyncMock(return_value=None)

        api = BotsApi(client)
        bots = await api.list()
        self.assertEqual(len(bots), 2)

        created = await api.create(CreateBotRequest(name="new", description="d"))
        self.assertEqual(created.bot_id, "b3")

        got = await api.get("b1")
        self.assertEqual(got.bot_id, "b1")

        updated = await api.update("b1", UpdateBotRequest(name="updated", description="d2"))
        self.assertEqual(updated.name, "updated")

        await api.delete("b1")

        token = await api.use_bot_token()
        self.assertEqual(token, "tok-active")
        self.assertEqual(client.bot_token(), "tok-active")

        with self.assertRaises(ApiError):
            await api.use_bot_token("missing")

        await client.close()


class FilesAndGroupsApiTests(unittest.IsolatedAsyncioTestCase):
    async def test_files_and_groups_paths(self):
        client = BochatClient("http://example.com", bot_token="b-token", user_token="u-token")
        client._request_multipart = AsyncMock(
            return_value={
                "file_id": "f1",
                "url": "http://example.com/f1",
                "filename": "a.txt",
                "created_at": "now",
            }
        )
        client._request_json = AsyncMock(
            return_value={
                "group_id": "g1",
                "group_code": "G1",
                "name": "Group",
                "description": None,
                "creator_id": "u1",
                "creator_name": None,
                "creator_avatar_url": None,
                "avatar_url": None,
                "is_public": True,
                "status": "active",
                "member_count": 1,
                "created_at": "now",
                "updated_at": "now",
            }
        )
        client._request_empty = AsyncMock(return_value=None)

        files_api = FilesApi(client)
        uploaded = await files_api.upload_bytes("a.txt", b"abc", mime="text/plain")
        self.assertEqual(uploaded.file_id, "f1")
        url = files_api.download_url("f1", "a b.txt")
        self.assertIn("a%20b.txt", url)

        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / "up.bin"
            p.write_bytes(b"xyz")
            await files_api.upload_path(p)

        await files_api.delete("f1")

        groups_api = GroupsApi(client)
        group = await groups_api.create(
            CreateGroupRequest(name="Group", group_code="G1", description=None)
        )
        self.assertEqual(group.group_id, "g1")
        await groups_api.delete("g1")

        await client.close()


if __name__ == "__main__":
    unittest.main()
