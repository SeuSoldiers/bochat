import asyncio
import time

from bochat_sdk import ApiError, BochatClient, CreateGroupRequest


async def main() -> None:
    client = BochatClient.builder("http://127.0.0.1:8080").build()
    ts = int(time.time())
    account = f"sdk_user_{ts}"
    password = "Passw0rd"
    nickname = f"SDK用户{ts % 10000}"

    group_ids: list[str] = []

    try:
        try:
            await (
                client.auth()
                .register()
                .account(account)
                .password(password)
                .nickname(nickname)
                .send()
            )
        except ApiError as err:
            if err.code != "account_conflict":
                raise
            await client.auth().login().account(account).password(password).send()

        bots = await client.bots().list()
        first_bot = bots[0]
        client.set_bot_token(first_bot.token)

        for i in range(3):
            group = await client.groups().create(
                CreateGroupRequest(
                    name=f"SDK-WS-Group-{ts % 10000}-{i}",
                    description="用于 ws 分发示例",
                    group_code=f"WS{ts % 10000}{i}",
                    bot_id=first_bot.bot_id,
                )
            )
            group_ids.append(group.group_id)

        session = await (
            client.ws()
            .heartbeat_interval(15)
            .heartbeat_timeout(45)
            .reconnect_max_attempts(20)
            .build()
        )
        dispatcher = (await session.spawn()).into_dispatcher()

        conn = await dispatcher.wait_connection_payload(timeout=10)
        print("连接成功:", conn.bot_name, "可用群:", ",".join(conn.group_ids))

        handled = {"count": 0}

        @dispatcher.on_message()
        async def on_default(msg):
            print(
                f"[Message-Handler] group={msg.group_id} sender={msg.sender_id} content={msg.content.to_dict()}"
            )
            handled["count"] += 1

        @dispatcher.on_group_message(group_ids[0])
        async def on_a(msg):
            print(
                f"[Handler-A] group={msg.group_id} sender={msg.sender_id} content={msg.content.to_dict()}"
            )
            handled["count"] += 1

        @dispatcher.on_group_message(group_ids[1])
        async def on_b(msg):
            print(
                f"[Handler-B] group={msg.group_id} sender={msg.sender_id} content={msg.content.to_dict()}"
            )
            handled["count"] += 1

        for idx, gid in enumerate(group_ids):
            await client.messages().send_text(gid, f"来自 ws 分发示例的消息 {idx + 1}")

        for _ in range(20):
            if handled["count"] >= len(group_ids):
                break
            await asyncio.sleep(0.2)

        await dispatcher.shutdown()

        for gid in group_ids:
            await client.groups().delete(gid)
            print("群聊资源清理完成:", gid)

        for bot in await client.bots().list():
            await client.bots().delete(bot.bot_id)
            print("Bot资源清理完成:", bot.bot_id)

        await client.auth().delete_account()
        print("用户账号清理完成")
    finally:
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())
