import asyncio
import sys

from bochat_sdk import BochatClient, MessageContent


async def main() -> None:
    if len(sys.argv) < 2:
        raise SystemExit("用法: python examples/ws_observe.py <bot_token>")

    token = sys.argv[1]
    client = BochatClient.builder("http://127.0.0.1:8080").build()
    client.set_bot_token(token)

    try:
        session = await client.ws().build()
        dispatcher = (await session.spawn()).into_dispatcher()
        conn = await dispatcher.wait_connection_payload(timeout=10)
        print("连接成功:", conn.bot_name, "可用群:", ",".join(conn.group_ids))

        @dispatcher.on_message()
        async def on_msg(msg):
            content_text = msg.content.as_text()
            if content_text is not None:
                print(
                    f"[Message-Handler] group={msg.group_id} sender={msg.sender_id} text={content_text}"
                )
            else:
                print(
                    f"[Message-Handler] group={msg.group_id} sender={msg.sender_id} msg_type={msg.msg_type} content={msg.content.to_dict()}"
                )

        await asyncio.Event().wait()
    finally:
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())
