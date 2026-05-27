import unittest
from unittest.mock import AsyncMock

from bochat_sdk.client import BochatClient
from bochat_sdk.error import ApiError, TransportError
from bochat_sdk.messages import MessagesApi
from bochat_sdk.models import MessageContent, SendMessageRequest
from bochat_sdk.retry import RetryPolicy


class MessagesApiTests(unittest.IsolatedAsyncioTestCase):
    async def test_send_builds_payload_and_returns_message(self):
        client = BochatClient("http://example.com", bot_token="bot-token")
        client._request_json = AsyncMock(
            return_value={
                "msg_id": 100,
                "group_id": "g1",
                "sender_id": "u1",
                "sender_name": "u",
                "sender_avatar_url": None,
                "content": {"text": "hi"},
                "msg_type": "text",
                "created_at": "now",
            }
        )
        api = MessagesApi(client)
        req = SendMessageRequest(group_id="g1", content=MessageContent.text("hi"), msg_type="text")
        resp = await api.send(req)
        self.assertEqual(resp.msg_id, 100)
        called_payload = client._request_json.call_args.args[3]
        self.assertEqual(called_payload["group_id"], "g1")
        self.assertIn("idempotency_key", called_payload)
        await client.close()

    async def test_send_text_and_file_content(self):
        client = BochatClient("http://example.com", bot_token="bot-token")
        client._request_json = AsyncMock(
            return_value={
                "msg_id": 1,
                "group_id": "g1",
                "sender_id": "u1",
                "sender_name": None,
                "sender_avatar_url": None,
                "content": {"text": "t"},
                "msg_type": "text",
                "created_at": "now",
            }
        )
        api = MessagesApi(client)
        await api.send_text("g1", "t")
        file_content = api.file_content("https://f")
        self.assertEqual(file_content.as_file_url(), "https://f")
        await client.close()

    async def test_history_with_query(self):
        client = BochatClient("http://example.com", bot_token="bot-token")
        client._get_json = AsyncMock(
            return_value={
                "group_id": "g1",
                "base_id": 10,
                "limit": 20,
                "next_base_id": 30,
                "messages": [],
            }
        )
        api = MessagesApi(client)
        resp = await api.history("g1", base_id=10, limit=20)
        self.assertEqual(resp.group_id, "g1")
        path = client._get_json.call_args.args[0]
        self.assertIn("base_id=10", path)
        self.assertIn("limit=20", path)
        await client.close()

    async def test_send_retries_then_success(self):
        client = BochatClient(
            "http://example.com",
            bot_token="bot-token",
            retry_policy=RetryPolicy(max_attempts=3, base_delay=0.0, max_delay=0.0),
        )
        client._request_json = AsyncMock(
            side_effect=[
                TransportError("net"),
                ApiError(code="x", message="retry", status=500),
                {
                    "msg_id": 2,
                    "group_id": "g1",
                    "sender_id": "u1",
                    "sender_name": None,
                    "sender_avatar_url": None,
                    "content": {"text": "ok"},
                    "msg_type": "text",
                    "created_at": "now",
                },
            ]
        )
        api = MessagesApi(client)
        resp = await api.send_text("g1", "ok")
        self.assertEqual(resp.msg_id, 2)
        self.assertEqual(client._request_json.call_count, 3)
        await client.close()

    async def test_send_non_retryable_error_raises(self):
        client = BochatClient(
            "http://example.com",
            bot_token="bot-token",
            retry_policy=RetryPolicy(max_attempts=3, base_delay=0.0, max_delay=0.0),
        )
        client._request_json = AsyncMock(
            side_effect=ApiError(code="bad", message="no", status=400)
        )
        api = MessagesApi(client)
        with self.assertRaises(ApiError):
            await api.send_text("g1", "bad")
        self.assertEqual(client._request_json.call_count, 1)
        await client.close()


if __name__ == "__main__":
    unittest.main()
