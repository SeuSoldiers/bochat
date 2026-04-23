from __future__ import annotations

import asyncio
import json
from dataclasses import dataclass
from typing import Any, Callable
from urllib.parse import urlparse

from .error import MissingBotToken, TransportError, WebSocketError
from .models import MessageResponse, WsConnectionPayload, WsEvent

try:
    import websockets
except Exception:  # pragma: no cover
    websockets = None


@dataclass
class _WsConfig:
    bot_token: str
    auto_reconnect: bool = True
    reconnect_max_attempts: int = 10
    reconnect_base_delay: float = 1.0
    heartbeat_interval: float = 20.0
    heartbeat_timeout: float = 60.0
    event_buffer: int = 128


class WsSessionBuilder:
    def __init__(self, client):
        self._client = client
        self._bot_token: str | None = None
        self._auto_reconnect = True
        self._reconnect_max_attempts = 10
        self._reconnect_base_delay = 1.0
        self._heartbeat_interval = 20.0
        self._heartbeat_timeout = 60.0
        self._event_buffer = 128

    def bot_token(self, token: str) -> "WsSessionBuilder":
        self._bot_token = token
        return self

    def auto_reconnect(self, enabled: bool) -> "WsSessionBuilder":
        self._auto_reconnect = enabled
        return self

    def reconnect_max_attempts(self, attempts: int) -> "WsSessionBuilder":
        self._reconnect_max_attempts = attempts
        return self

    def heartbeat_interval(self, interval: float) -> "WsSessionBuilder":
        self._heartbeat_interval = interval
        return self

    def heartbeat_timeout(self, timeout: float) -> "WsSessionBuilder":
        self._heartbeat_timeout = timeout
        return self

    def event_buffer(self, size: int) -> "WsSessionBuilder":
        self._event_buffer = max(1, size)
        return self

    async def build(self) -> "WsSession":
        if websockets is None:
            raise WebSocketError(
                "websockets 未安装，请安装 WS 依赖：`pip install bochat-sdk[ws]` "
                "或（uv）`uv pip install -e '.[ws]'` / `uv sync --extra ws`"
            )

        token = self._bot_token or self._client.bot_token()
        if not token:
            raise MissingBotToken("缺少 Bot token，请先选择或设置 Bot")

        config = _WsConfig(
            bot_token=token,
            auto_reconnect=self._auto_reconnect,
            reconnect_max_attempts=self._reconnect_max_attempts,
            reconnect_base_delay=self._reconnect_base_delay,
            heartbeat_interval=self._heartbeat_interval,
            heartbeat_timeout=self._heartbeat_timeout,
            event_buffer=self._event_buffer,
        )
        return WsSession(client=self._client, config=config)


class WsSession:
    def __init__(self, client, config: _WsConfig):
        self._client = client
        self._config = config

    def websocket_url(self) -> str:
        return _to_ws_url(self._client.base_url())

    async def spawn(self) -> "WsSessionHandle":
        handle = WsSessionHandle(event_buffer=self._config.event_buffer)
        handle._task = asyncio.create_task(self._run(handle))
        return handle

    async def _run(self, handle: "WsSessionHandle") -> None:
        attempt = 0
        while not handle._stop.is_set():
            try:
                await self._connect_once(handle)
                attempt = 0
                if not self._config.auto_reconnect:
                    return
            except asyncio.CancelledError:
                return
            except Exception as exc:
                if not self._config.auto_reconnect:
                    handle._error = WebSocketError(str(exc))
                    return

                attempt += 1
                if attempt > self._config.reconnect_max_attempts:
                    handle._error = WebSocketError(str(exc))
                    return
                delay = self._config.reconnect_base_delay * attempt
                await asyncio.sleep(delay)

    async def _connect_once(self, handle: "WsSessionHandle") -> None:
        ws_url = self.websocket_url()
        assert websockets is not None
        try:
            headers = {"Authorization": f"Bearer {self._config.bot_token}"}
            connect_kwargs = {"ping_interval": None}
            try:
                ws_cm = websockets.connect(
                    ws_url,
                    additional_headers=headers,
                    **connect_kwargs,
                )
            except TypeError:
                ws_cm = websockets.connect(
                    ws_url,
                    extra_headers=headers,
                    **connect_kwargs,
                )

            async with ws_cm as ws:
                last_seen = asyncio.get_running_loop().time()
                while not handle._stop.is_set():
                    try:
                        raw = await asyncio.wait_for(
                            ws.recv(), timeout=self._config.heartbeat_interval
                        )
                    except asyncio.TimeoutError:
                        if (
                            asyncio.get_running_loop().time() - last_seen
                            > self._config.heartbeat_timeout
                        ):
                            raise TransportError("WS 心跳超时，触发重连")
                        pong = ws.ping()
                        await asyncio.wait_for(pong, timeout=self._config.heartbeat_timeout)
                        continue

                    last_seen = asyncio.get_running_loop().time()
                    event = _parse_event(raw)
                    if event is None:
                        continue
                    if event.event_type == "connection":
                        payload = event.as_connection_payload()
                        if payload:
                            handle._connection_payload = payload
                            handle._connection_event.set()
                    await handle.events.put(event)
        except Exception as exc:
            raise TransportError(str(exc)) from exc


class WsSessionHandle:
    def __init__(self, event_buffer: int):
        self.events: asyncio.Queue[WsEvent] = asyncio.Queue(maxsize=max(1, event_buffer))
        self._stop = asyncio.Event()
        self._connection_event = asyncio.Event()
        self._connection_payload: WsConnectionPayload | None = None
        self._task: asyncio.Task | None = None
        self._error: Exception | None = None

    async def shutdown(self) -> None:
        self._stop.set()
        if self._task:
            self._task.cancel()
            try:
                await self._task
            except Exception:
                pass

    def into_dispatcher(self) -> "WsDispatcher":
        return WsDispatcher(self)

    def available_groups(self) -> list[str]:
        if not self._connection_payload:
            return []
        return list(self._connection_payload.group_ids)

    async def wait_connection_payload(
        self, timeout: float | None = None
    ) -> WsConnectionPayload:
        if self._connection_payload:
            return self._connection_payload
        await asyncio.wait_for(self._connection_event.wait(), timeout=timeout)
        assert self._connection_payload is not None
        return self._connection_payload

    async def recv_message_for_group(self, group_id: str) -> WsEvent | None:
        while True:
            if self._error is not None:
                raise self._error
            event = await self.events.get()
            if event.event_type == "message" and event.group_id() == group_id:
                return event


class WsDispatcher:
    def __init__(self, handle: WsSessionHandle):
        self._handle = handle
        self._group_subscribers: dict[str, list[asyncio.Queue[WsEvent]]] = {}
        self._fallback_subscribers: list[asyncio.Queue[WsEvent]] = []
        self._all_message_subscribers: list[asyncio.Queue[WsEvent]] = []
        self._group_handlers: dict[str, Callable[[WsEvent], Any]] = {}
        self._default_handler: Callable[[WsEvent], Any] | None = None
        self._task = asyncio.create_task(self._dispatch_loop())

    async def _dispatch_loop(self) -> None:
        while not self._handle._stop.is_set():
            try:
                event = await self._handle.events.get()
            except asyncio.CancelledError:
                return

            if event.event_type != "message":
                continue

            for queue in self._all_message_subscribers:
                _try_put(queue, event)

            handled = False
            group_id = event.group_id()
            if group_id:
                for queue in self._group_subscribers.get(group_id, []):
                    _try_put(queue, event)
                    handled = True

                group_handler = self._group_handlers.get(group_id)
                if group_handler is not None:
                    _safe_call(group_handler, event)
                    handled = True

            if handled:
                continue

            for queue in self._fallback_subscribers:
                _try_put(queue, event)

            if self._default_handler is not None:
                _safe_call(self._default_handler, event)

    def on_event(
        self, fn: Callable[[WsEvent], Any] | None = None
    ) -> Callable[[WsEvent], Any] | Callable[[Callable[[WsEvent], Any]], Callable[[WsEvent], Any]]:
        if fn is None:
            def _decorator(handler: Callable[[WsEvent], Any]) -> Callable[[WsEvent], Any]:
                self._default_handler = handler
                return handler

            return _decorator

        self._default_handler = fn
        return fn

    def on_message(
        self, fn: Callable[[MessageResponse], Any] | None = None
    ) -> (
        Callable[[MessageResponse], Any]
        | Callable[[Callable[[MessageResponse], Any]], Callable[[MessageResponse], Any]]
    ):
        def _bind(handler: Callable[[MessageResponse], Any]) -> Callable[[MessageResponse], Any]:
            def _inner(event: WsEvent) -> None:
                msg = event.as_message_payload()
                if msg is not None:
                    _safe_call(handler, msg)

            self._default_handler = _inner
            return handler

        if fn is None:
            return _bind
        return _bind(fn)

    def on_group(
        self, group_id: str
    ) -> Callable[[Callable[[WsEvent], Any]], Callable[[WsEvent], Any]]:
        def _decorator(handler: Callable[[WsEvent], Any]) -> Callable[[WsEvent], Any]:
            self._group_handlers[group_id] = handler
            return handler

        return _decorator

    def on_group_message(
        self, group_id: str
    ) -> Callable[[Callable[[MessageResponse], Any]], Callable[[MessageResponse], Any]]:
        def _decorator(handler: Callable[[MessageResponse], Any]) -> Callable[[MessageResponse], Any]:
            def _inner(event: WsEvent) -> None:
                msg = event.as_message_payload()
                if msg is not None:
                    _safe_call(handler, msg)

            self._group_handlers[group_id] = _inner
            return handler

        return _decorator

    async def default_handler(self, handler: Callable[[WsEvent], Any]) -> "WsDispatcher":
        self.on_event(handler)
        return self

    async def default_message_handler(
        self, handler: Callable[[MessageResponse], Any]
    ) -> "WsDispatcher":
        self.on_message(handler)
        return self

    async def group_handler(
        self, group_id: str, handler: Callable[[WsEvent], Any]
    ) -> "WsDispatcher":
        self.on_group(group_id)(handler)
        return self

    async def group_message_handler(
        self, group_id: str, handler: Callable[[MessageResponse], Any]
    ) -> "WsDispatcher":
        self.on_group_message(group_id)(handler)
        return self

    async def subscribe_group(self, group_id: str, buffer: int = 64) -> asyncio.Queue[WsEvent]:
        queue: asyncio.Queue[WsEvent] = asyncio.Queue(maxsize=max(1, buffer))
        self._group_subscribers.setdefault(group_id, []).append(queue)
        return queue

    async def subscribe_fallback(self, buffer: int = 64) -> asyncio.Queue[WsEvent]:
        queue: asyncio.Queue[WsEvent] = asyncio.Queue(maxsize=max(1, buffer))
        self._fallback_subscribers.append(queue)
        return queue

    async def subscribe_all_messages(self, buffer: int = 64) -> asyncio.Queue[WsEvent]:
        queue: asyncio.Queue[WsEvent] = asyncio.Queue(maxsize=max(1, buffer))
        self._all_message_subscribers.append(queue)
        return queue

    async def shutdown(self) -> None:
        await self._handle.shutdown()
        self._task.cancel()
        try:
            await self._task
        except Exception:
            pass

    def available_groups(self) -> list[str]:
        return self._handle.available_groups()

    async def wait_connection_payload(
        self, timeout: float | None = None
    ) -> WsConnectionPayload:
        return await self._handle.wait_connection_payload(timeout=timeout)


def _to_ws_url(base_url: str) -> str:
    parsed = urlparse(base_url)
    if parsed.scheme not in {"http", "https"}:
        raise WebSocketError(f"不支持的 base_url 协议: {parsed.scheme}")
    ws_scheme = "wss" if parsed.scheme == "https" else "ws"
    root = f"{ws_scheme}://{parsed.netloc}"
    return f"{root}/ws"


def _parse_event(raw: Any) -> WsEvent | None:
    if isinstance(raw, bytes):
        raw = raw.decode("utf-8", errors="ignore")
    if not isinstance(raw, str):
        return None
    try:
        data = json.loads(raw)
    except json.JSONDecodeError:
        return None
    if not isinstance(data, dict):
        return None
    return WsEvent.from_dict(data)


def _try_put(queue: asyncio.Queue[WsEvent], event: WsEvent) -> None:
    try:
        queue.put_nowait(event)
    except asyncio.QueueFull:
        pass


def _safe_call(fn: Callable[[Any], Any], arg: Any) -> None:
    try:
        result = fn(arg)
        if asyncio.iscoroutine(result):
            asyncio.create_task(result)
    except Exception:
        pass
