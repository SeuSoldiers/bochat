import asyncio
import time

from bochat_sdk import (
    ApiError,
    BochatClient,
    CreateGroupRequest,
)


async def main() -> None:
    client = BochatClient.builder("http://127.0.0.1:8080").build()
    ts = int(time.time())
    account = f"sdk_user_{ts}"
    password = "Passw0rd"
    nickname = f"SDK用户{ts % 10000}"

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
        if not bots:
            raise RuntimeError("需要至少一个 Bot")
        first_bot = bots[0]
        client.set_bot_token(first_bot.token)

        group = await client.groups().create(
            CreateGroupRequest(
                name=f"SDK测试群-{ts % 10000}",
                description="python bochat_sdk 基础流程自动创建",
                group_code=f"PYSDK{ts % 100000}",
                bot_id=first_bot.bot_id,
            )
        )
        print("群聊创建成功:", group.group_id)

        sent = await client.messages().send_text(group.group_id, "你好，来自 python bochat_sdk")
        print("发送成功:", sent.msg_id)

        uploaded = await client.files().upload_bytes(
            "hello.txt", b"hello from python sdk", mime="text/plain"
        )
        print("文件上传成功:", uploaded.url)

        await client.files().delete(uploaded.file_id)
        print("文件资源清理完成:", uploaded.file_id)

        await client.groups().delete(group.group_id)
        print("群聊资源清理完成:", group.group_id)

        for bot in await client.bots().list():
            await client.bots().delete(bot.bot_id)
            print("Bot资源清理完成:", bot.bot_id)

        await client.auth().delete_account()
        print("用户账号清理完成")
    finally:
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())
