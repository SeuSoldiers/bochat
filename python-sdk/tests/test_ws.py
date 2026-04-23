import asyncio
import unittest

from bochat_sdk.models import WsEvent
from bochat_sdk.ws import WsSessionHandle, _parse_event, _to_ws_url


class WsUtilsTests(unittest.TestCase):
    def test_to_ws_url_http(self):
        self.assertEqual(
            _to_ws_url("http://127.0.0.1:8080"),
            "ws://127.0.0.1:8080/ws",
        )

    def test_to_ws_url_https(self):
        self.assertEqual(
            _to_ws_url("https://example.com"),
            "wss://example.com/ws",
        )

    def test_parse_event(self):
        raw = '{"type":"connection","payload":{"bot_id":"b1","bot_name":"bot","group_ids":["g1"]},"timestamp":"t"}'
        event = _parse_event(raw)
        self.assertIsNotNone(event)
        assert event is not None
        self.assertEqual(event.event_type, "connection")
        payload = event.as_connection_payload()
        self.assertIsNotNone(payload)
        assert payload is not None
        self.assertEqual(payload.bot_id, "b1")


class WsDispatcherTests(unittest.IsolatedAsyncioTestCase):
    async def test_dispatcher_routing(self):
        handle = WsSessionHandle(event_buffer=16)
        dispatcher = handle.into_dispatcher()

        called = {"group": 0, "default": 0}

        @dispatcher.on_group_message("g1")
        async def on_group(msg):
            called["group"] += 1

        @dispatcher.on_message()
        async def on_default(msg):
            called["default"] += 1

        q_all = await dispatcher.subscribe_all_messages(buffer=8)
        q_group = await dispatcher.subscribe_group("g1", buffer=8)
        q_fallback = await dispatcher.subscribe_fallback(buffer=8)

        await handle.events.put(
            WsEvent.from_dict(
                {
                    "type": "message",
                    "payload": {
                        "msg_id": 1,
                        "group_id": "g1",
                        "sender_id": "s1",
                        "content": {"text": "hello"},
                        "msg_type": "text",
                        "created_at": "t1",
                    },
                    "timestamp": "t1",
                }
            )
        )
        await handle.events.put(
            WsEvent.from_dict(
                {
                    "type": "message",
                    "payload": {
                        "msg_id": 2,
                        "group_id": "g2",
                        "sender_id": "s1",
                        "content": {"text": "world"},
                        "msg_type": "text",
                        "created_at": "t2",
                    },
                    "timestamp": "t2",
                }
            )
        )
        await asyncio.sleep(0.05)

        self.assertEqual(called["group"], 1)
        self.assertEqual(called["default"], 1)
        self.assertEqual(q_all.qsize(), 2)
        self.assertEqual(q_group.qsize(), 1)
        self.assertEqual(q_fallback.qsize(), 1)

        await dispatcher.shutdown()


if __name__ == "__main__":
    unittest.main()
