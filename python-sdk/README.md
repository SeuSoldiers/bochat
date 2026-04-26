# bochat-sdk (Python)

BoChat 平台 Python 异步 SDK，接口语义与 `crates/bochat_sdk` Rust 版本保持一致。

## 安装

```bash
pip install bochat-sdk
```

启用 WebSocket 能力：

```bash
pip install "bochat-sdk[ws]"
```

本地开发安装：

```bash
cd python-sdk
pip install -e .
pip install -e ".[ws]"
```

## 运行 Example

简易运行说明见：

- [examples/README.md](/home/harkerhand/codes/bochat/python-sdk/examples/README.md)

WS 相关对齐示例：

- `examples/ws_observe.py`
- `examples/ws_dispatcher.py`

`ws_dispatcher.py` 使用 Python 装饰器注册处理器（更贴合 Python 风格）。

## 快速开始

```python
import asyncio
from bochat_sdk import BochatClient, CreateGroupRequest


async def main() -> None:
    client = BochatClient.builder("http://127.0.0.1:8080").build()
    try:
        await (
            client.auth()
            .login()
            .account("demo_user_01")
            .password("Passw0rd!")
            .send()
        )

        bots = await client.bots().list()
        first_bot = bots[0]
        client.set_bot_token(first_bot.token)

        group = await client.groups().create(
            CreateGroupRequest(
                name="python-sdk-demo",
                description="created by python sdk",
                group_code="PYSDK001",
                bot_id=first_bot.bot_id,
            )
        )

        sent = await client.messages().send_text(group.group_id, "hello from python sdk")
        print("msg_id=", sent.msg_id)
    finally:
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())
```

## 对齐能力

- 注册/登录 + 自动保存 user token
- Bot 列表、创建、更新、删除、一键选中 bot token
- 群创建/删除
- 消息发送（含幂等键）和历史拉取
- 文件上传/删除与下载 URL 构造
- 可选 WebSocket 会话（自动重连、心跳、事件队列）
