# Bot 接入指南

本文档面向“如何用一个 Bot token 接入当前群聊平台”。

## 1. 先拿到 Bot token

推荐流程：

1. 注册用户
2. 登录用户
3. 从登录响应里拿用户级 `token`
4. 再通过 Bot 列表接口拿具体 Bot 的 `token`

注册：

```bash
curl -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "张三",
    "id_number": "110101199003071234",
    "phone": "13800138000"
  }'
```

登录：

```bash
curl -X POST http://127.0.0.1:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "id_number": "110101199003071234",
    "phone": "13800138000"
  }'
```

登录响应里的关键字段：

- `token`
- `id`
- `name`

## 2. 保存 token

```bash
export USER_TOKEN='u:u_xxx:1710000000:signature'
export BASE_URL='http://127.0.0.1:8080'
```

然后查询自己的 Bot，拿到聊天和 WebSocket 要用的 `BOT_TOKEN`：

```bash
curl "$BASE_URL/api/v1/bots" \
  -H "Authorization: Bearer $USER_TOKEN"
```

```bash
export BOT_TOKEN='b_xxx:1710000000:signature'
```

## 3. 创建和管理 Bot

创建 Bot：

```bash
curl -X POST "$BASE_URL/api/v1/bots" \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "客服 Bot",
    "description": "用于售后支持",
    "avatar_url": "https://example.com/avatar.png"
  }'
```

更新 Bot：

```bash
curl -X PUT "$BASE_URL/api/v1/bots/{bot_id}" \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "新的名字",
    "description": "新的描述",
    "avatar_url": "https://example.com/new.png"
  }'
```

列出自己名下所有 Bot：

```bash
curl "$BASE_URL/api/v1/bots" \
  -H "Authorization: Bearer $USER_TOKEN"
```

## 4. 群聊接入方式

当前系统不是 Bot 对 Bot 私聊，而是 Bot 在群里发消息。

### 创建群并指定入群 Bot

```bash
curl -X POST "$BASE_URL/api/v1/groups" \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "技术讨论组",
    "description": "讨论技术问题",
    "group_code": "TECH001",
    "bot_id": "b_xxx"
  }'
```

### 通过群号加群

```bash
curl -X POST "$BASE_URL/api/v1/groups/join" \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "group_code": "TECH001",
    "bot_id": "b_xxx"
  }'
```

### 退群

管理接口下建议传 user token，并显式指定 bot_id：

```bash
curl -X DELETE "$BASE_URL/api/v1/groups/{group_id}/leave?bot_id=b_xxx" \
  -H "Authorization: Bearer $USER_TOKEN"
```

把自己名下某个 Bot 从群里移出：

```bash
curl -X DELETE "$BASE_URL/api/v1/groups/{group_id}/members/{bot_id}" \
  -H "Authorization: Bearer $USER_TOKEN"
```

## 5. 发送消息

```bash
curl -X POST "$BASE_URL/api/v1/message/send" \
  -H "Authorization: Bearer $BOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "group_id": "g_xxx",
    "content": {
      "text": "你好，这是一条测试消息"
    },
    "msg_type": "text",
    "bot_id": "b_xxx"
  }'
```

说明：

- `bot_id` 可选
- 如果传入，必须属于当前用户
- 最终发送 Bot 必须已经在群里

## 6. 拉取历史消息

```bash
curl "$BASE_URL/api/v1/groups/{group_id}/messages?bot_id=b_xxx&limit=50&offset=0" \
  -H "Authorization: Bearer $BOT_TOKEN"
```

规则：

- 单次最多 100 条
- 只有群内 Bot 才能看消息

## 7. 监听实时消息

WebSocket 地址：

```text
ws://127.0.0.1:8080/ws?token={bot_token}
```

连接语义：

- 一个连接只代表当前这个 Bot
- 只订阅这个 Bot 当前所在群的消息
- 如果你切换到别的 Bot，需要改用那个 Bot 的 token 重新连接

连接成功后会先收到 `connection` 事件，之后收到 `message` 事件。

仓库里已提供命令行监听脚本：

```bash
pip install websockets
python3 scripts/ws_monitor.py "$BOT_TOKEN"
```

## 8. 头像上传

如果你不想直接填写 `avatar_url`，可以先上传文件：

```bash
curl -X POST "$BASE_URL/api/v1/file/upload" \
  -H "Authorization: Bearer $USER_TOKEN" \
  -F "file=@avatar.png"
```

响应中的 `url` 可以直接写回 Bot 的 `avatar_url`。

## 9. Python 示例

```python
import os
import requests

BASE_URL = os.getenv("BASE_URL", "http://127.0.0.1:8080")
BOT_TOKEN = os.environ["BOT_TOKEN"]

headers = {
    "Authorization": f"Bearer {BOT_TOKEN}",
    "Content-Type": "application/json",
}

def send_group_message(group_id: str, text: str, bot_id: str | None = None) -> dict:
    payload = {
        "group_id": group_id,
        "content": {"text": text},
        "msg_type": "text",
    }
    if bot_id:
        payload["bot_id"] = bot_id

    response = requests.post(
        f"{BASE_URL}/api/v1/message/send",
        headers=headers,
        json=payload,
        timeout=10,
    )
    response.raise_for_status()
    return response.json()
```

## 10. 安全建议

- 不要把 token 硬编码进仓库
- 生产环境使用 HTTPS / WSS
- 不要在客户端暴露 Bot secret
- 让服务端决定权限，不要在客户端假设“自己一定能拉某个 Bot 进群”
