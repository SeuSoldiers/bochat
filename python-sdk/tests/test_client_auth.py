import unittest
from dataclasses import dataclass
from unittest.mock import AsyncMock

import httpx

from bochat_sdk.auth import AuthApi
from bochat_sdk.client import AuthKind, BochatClient
from bochat_sdk.error import (
    ApiError,
    HttpStatusError,
    InvalidUrl,
    MissingBotToken,
    MissingUserToken,
    SerdeError,
    TransportError,
)
from bochat_sdk.retry import RetryPolicy


class BochatClientCoreTests(unittest.IsolatedAsyncioTestCase):
    async def test_builder_sets_values(self):
        policy = RetryPolicy(max_attempts=5, base_delay=0.0, max_delay=0.0)
        client = (
            BochatClient.builder("http://example.com/")
            .timeout_secs(3)
            .user_agent("ua-test")
            .user_token("u-token")
            .bot_token("b-token")
            .retry_policy(policy)
            .build()
        )
        self.assertEqual(client.base_url(), "http://example.com")
        self.assertEqual(client.user_token(), "u-token")
        self.assertEqual(client.bot_token(), "b-token")
        self.assertEqual(client.retry_policy().max_attempts, 5)
        await client.close()

    async def test_auth_headers_and_missing_tokens(self):
        client = BochatClient("http://example.com")
        self.assertEqual(client._auth_headers(AuthKind.NONE), {})
        with self.assertRaises(MissingUserToken):
            client._auth_headers(AuthKind.USER)
        with self.assertRaises(MissingBotToken):
            client._auth_headers(AuthKind.BOT)
        client.set_user_token("u1")
        client.set_bot_token("b1")
        self.assertEqual(client._auth_headers(AuthKind.USER)["Authorization"], "Bearer u1")
        self.assertEqual(client._auth_headers(AuthKind.BOT)["Authorization"], "Bearer b1")
        await client.close()

    async def test_path_to_url_and_invalid_url(self):
        client = BochatClient("http://host")
        self.assertEqual(client._path_to_url("/x"), "http://host/x")
        self.assertEqual(client._path_to_url("x"), "http://host/x")
        self.assertEqual(client._path_to_url("https://a.com/p"), "https://a.com/p")
        with self.assertRaises(InvalidUrl):
            client._path_to_url("http://")
        await client.close()

    async def test_decode_json_and_non_object(self):
        client = BochatClient("http://example.com")
        ok = httpx.Response(200, json={"a": 1})
        self.assertEqual(client._decode_json(ok)["a"], 1)
        bad = httpx.Response(200, json=[1, 2, 3])
        with self.assertRaises(SerdeError):
            client._decode_json(bad)
        await client.close()

    async def test_parse_http_error_api_and_plain(self):
        client = BochatClient("http://example.com")
        api_resp = httpx.Response(400, json={"code": "bad", "message": "oops", "status": 400})
        err = client._parse_http_error(api_resp)
        self.assertIsInstance(err, ApiError)
        plain_resp = httpx.Response(503, text="down")
        plain_err = client._parse_http_error(plain_resp)
        self.assertIsInstance(plain_err, HttpStatusError)
        self.assertEqual(plain_err.status, 503)
        await client.close()

    async def test_get_json_retries_and_succeeds(self):
        calls = {"n": 0}

        async def handler(request: httpx.Request) -> httpx.Response:
            calls["n"] += 1
            if calls["n"] == 1:
                raise httpx.ConnectError("network down")
            return httpx.Response(200, json={"ok": True})

        client = BochatClient("http://example.com", retry_policy=RetryPolicy(2, 0.0, 0.0))
        await client._http.aclose()
        client._http = httpx.AsyncClient(transport=httpx.MockTransport(handler))
        result = await client._get_json("/ping", AuthKind.NONE)
        self.assertTrue(result["ok"])
        self.assertEqual(calls["n"], 2)
        await client.close()

    async def test_get_json_does_not_retry_post_status(self):
        async def handler(request: httpx.Request) -> httpx.Response:
            return httpx.Response(503, text="unavailable")

        client = BochatClient("http://example.com")
        await client._http.aclose()
        client._http = httpx.AsyncClient(transport=httpx.MockTransport(handler))
        with self.assertRaises(HttpStatusError):
            await client._request_json("POST", "/x", AuthKind.NONE, {"x": 1})
        await client.close()

    async def test_request_empty_and_multipart(self):
        async def handler(request: httpx.Request) -> httpx.Response:
            if request.url.path == "/ok":
                return httpx.Response(204, text="")
            if request.url.path == "/multipart":
                return httpx.Response(200, json={"file_id": "f1"})
            return httpx.Response(500, text="e")

        client = BochatClient("http://example.com")
        await client._http.aclose()
        client._http = httpx.AsyncClient(transport=httpx.MockTransport(handler))
        await client._request_empty("DELETE", "/ok", AuthKind.NONE)
        data = await client._request_multipart("/multipart", AuthKind.NONE, {"f": ("x.txt", b"a")})
        self.assertEqual(data["file_id"], "f1")
        with self.assertRaises(HttpStatusError):
            await client._request_empty("DELETE", "/bad", AuthKind.NONE)
        await client.close()

    async def test_json_payload_conversion(self):
        @dataclass
        class D:
            a: int

        class T:
            def to_dict(self):
                return {"v": 1}

        self.assertIsNone(BochatClient._json_payload(None))
        self.assertEqual(BochatClient._json_payload(T()), {"v": 1})
        self.assertEqual(BochatClient._json_payload(D(3)), {"a": 3})
        self.assertEqual(BochatClient._json_payload({"k": 1}), {"k": 1})


class AuthApiTests(unittest.IsolatedAsyncioTestCase):
    async def test_register_login_and_profile_calls(self):
        client = BochatClient("http://example.com")
        client._request_json = AsyncMock(
            side_effect=[
                {"message": "ok", "name": "n1", "token": "t1", "account": "a1"},
                {"message": "ok", "name": "n2", "token": "t2", "account": "a2"},
                {"name": "newname"},
            ]
        )
        client._get_json = AsyncMock(return_value={"name": "self"})
        client._request_empty = AsyncMock(return_value=None)

        api = AuthApi(client)
        register_resp = await api.register().account("a").password("p").nickname("nick").send()
        self.assertEqual(register_resp.token, "t1")
        self.assertEqual(client.user_token(), "t1")

        login_resp = await api.login().account("a").password("p").send()
        self.assertEqual(login_resp.token, "t2")
        self.assertEqual(client.user_token(), "t2")

        me = await api.me()
        self.assertEqual(me.name, "self")
        updated = await api.update_profile(type("Req", (), {"to_dict": lambda self: {"name": "x"}})())
        self.assertEqual(updated.name, "newname")
        await api.delete_account()
        await client.close()


if __name__ == "__main__":
    unittest.main()
