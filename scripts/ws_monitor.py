#!/usr/bin/env python3
import argparse
import asyncio
import json
import signal
import sys
from datetime import datetime
from typing import Any
from urllib.parse import urlparse, urlunparse

import websockets


def build_ws_url(base_url: str) -> str:
    parsed = urlparse(base_url)

    if parsed.scheme in ("http", "ws"):
        scheme = "ws"
    elif parsed.scheme in ("https", "wss"):
        scheme = "wss"
    else:
        raise ValueError(f"unsupported URL scheme: {parsed.scheme or '(empty)'}")

    path = parsed.path.rstrip("/")
    if not path or path == "/":
        path = "/ws"

    return urlunparse((scheme, parsed.netloc, path, "", "", ""))


def pretty_event(raw_message: str) -> str:
    try:
        payload: dict[str, Any] = json.loads(raw_message)
    except json.JSONDecodeError:
        return raw_message

    event_type = payload.get("type", "unknown")
    timestamp = payload.get("timestamp", "")
    event_payload = payload.get("payload")

    if event_type == "connection" and isinstance(event_payload, dict):
        bot_id = event_payload.get("bot_id", "")
        bot_name = event_payload.get("bot_name", "")
        group_ids = event_payload.get("group_ids", [])
        return (
            f"[{timestamp}] connection\n"
            f"bot_id: {bot_id}\n"
            f"bot_name: {bot_name}\n"
            f"group_ids: {group_ids}"
        )

    try:
        body = json.dumps(event_payload, ensure_ascii=False, indent=2)
    except TypeError:
        body = str(event_payload)

    return f"[{timestamp}] {event_type}\n{body}"


async def monitor(ws_url: str, token: str, reconnect_delay: float) -> None:
    stop_event = asyncio.Event()
    loop = asyncio.get_running_loop()

    def request_stop() -> None:
        stop_event.set()

    for sig in (signal.SIGINT, signal.SIGTERM):
        try:
            loop.add_signal_handler(sig, request_stop)
        except NotImplementedError:
            pass

    while not stop_event.is_set():
        try:
            print(f"{datetime.now().isoformat(timespec='seconds')} connecting to {ws_url}")
            headers = {"Authorization": f"Bearer {token}"}
            try:
                ws_cm = websockets.connect(
                    ws_url,
                    additional_headers=headers,
                    ping_interval=20,
                    ping_timeout=20,
                )
            except TypeError:
                ws_cm = websockets.connect(
                    ws_url,
                    extra_headers=headers,
                    ping_interval=20,
                    ping_timeout=20,
                )

            async with ws_cm as websocket:
                print(f"{datetime.now().isoformat(timespec='seconds')} connected")

                while not stop_event.is_set():
                    try:
                        message = await asyncio.wait_for(websocket.recv(), timeout=1.0)
                    except asyncio.TimeoutError:
                        continue

                    print(pretty_event(message), flush=True)
        except websockets.ConnectionClosed as exc:
            print(
                f"{datetime.now().isoformat(timespec='seconds')} connection closed: "
                f"code={exc.code} reason={exc.reason}",
                file=sys.stderr,
            )
        except Exception as exc:
            print(
                f"{datetime.now().isoformat(timespec='seconds')} connection error: {exc}",
                file=sys.stderr,
            )

        if not stop_event.is_set():
            print(
                f"{datetime.now().isoformat(timespec='seconds')} reconnecting in {reconnect_delay:.1f}s",
                file=sys.stderr,
            )
            await asyncio.sleep(reconnect_delay)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Monitor websocket events for the groups currently joined by a single bot."
    )
    parser.add_argument("token", help="single bot token sent as Authorization: Bearer ...")
    parser.add_argument(
        "--url",
        default="http://127.0.0.1:8080/ws",
        help="base websocket endpoint, default: http://127.0.0.1:8080/ws",
    )
    parser.add_argument(
        "--reconnect-delay",
        type=float,
        default=3.0,
        help="seconds to wait before reconnecting after disconnect",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    try:
        ws_url = build_ws_url(args.url)
    except ValueError as exc:
        print(f"invalid url: {exc}", file=sys.stderr)
        return 2

    try:
        asyncio.run(monitor(ws_url, args.token, args.reconnect_delay))
    except KeyboardInterrupt:
        pass

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
