# Bot接入指南

本指南将帮助开发者快速接入聊天平台并创建自己的Bot。

## 📋 快速开始

### 第1步：注册用户账户

首先，您需要创建一个用户账户。这是一个实名认证的过程。

```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "张三",
    "id_number": "110101199003071234",
    "phone": "13800138000"
  }'
```

**响应示例：**
```json
{
  "user_id": "u_550e8400-e29b-41d4-a716-446655440000",
  "name": "张三",
  "id_number": "110101199003071234",
  "phone": "13800138000",
  "bot_id": "b_550e8400-e29b-41d4-a716-446655440001",
  "bot_token": "b_550e8400-e29b-41d4-a716-446655440001:1637000000:signature...",
  "created_at": "2024-01-01T12:00:00Z"
}
```

**重要字段：**
- `user_id`: 用户唯一标识
- `bot_token`: 初始Bot令牌（用于API认证）
- `bot_id`: 初始Bot ID

### 第2步：保存Bot Token

Bot token是您与平台通信的凭证，必须**安全保管**。

```bash
export BOT_TOKEN="b_550e8400-e29b-41d4-a716-446655440001:1637000000:signature..."
```

或存储在配置文件中：
```yaml
# config.yaml
bot:
  token: "b_550e8400-e29b-41d4-a716-446655440001:1637000000:signature..."
  server_url: "http://localhost:8080"
```

## 🤖 Bot管理

### 创建新Bot

您可以为自己的账户创建多个Bot，用于不同的场景。

```bash
curl -X POST http://localhost:8080/api/v1/bots \
  -H "Authorization: Bearer $BOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "我的客服Bot",
    "description": "用于处理客户服务的机器人"
  }'
```

**响应示例：**
```json
{
  "bot_id": "b_550e8400-e29b-41d4-a716-446655440002",
  "owner_id": "u_550e8400-e29b-41d4-a716-446655440000",
  "name": "我的客服Bot",
  "description": "用于处理客户服务的机器人",
  "status": "active",
  "token": "b_550e8400-e29b-41d4-a716-446655440002:1637000001:signature...",
  "created_at": "2024-01-01T13:00:00Z",
  "updated_at": "2024-01-01T13:00:00Z"
}
```

### 查看所有Bot

```bash
curl http://localhost:8080/api/v1/bots \
  -H "Authorization: Bearer $BOT_TOKEN"
```

### 获取Bot详情

```bash
curl http://localhost:8080/api/v1/bots/{bot_id}
```

### 删除Bot

只有Bot的所有者可以删除Bot：

```bash
curl -X DELETE http://localhost:8080/api/v1/bots/{bot_id} \
  -H "Authorization: Bearer $BOT_TOKEN"
```

**响应示例：**
```json
{
  "message": "Bot deleted successfully",
  "bot_id": "b_550e8400-e29b-41d4-a716-446655440002"
}
```

## 💬 发送消息

### 基本消息发送

只有Bot才能通过API发送消息。消息的发送者和接收者都必须是Bot ID。

```bash
curl -X POST http://localhost:8080/api/v1/message/send \
  -H "Authorization: Bearer $BOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "to_id": "b_550e8400-e29b-41d4-a716-446655440003",
    "content": {
      "text": "你好，这是一条测试消息"
    },
    "msg_type": "text"
  }'
```

**响应示例：**
```json
{
  "msg_id": 1,
  "sender_id": "b_550e8400-e29b-41d4-a716-446655440001",
  "to_id": "b_550e8400-e29b-41d4-a716-446655440003",
  "content": {
    "text": "你好，这是一条测试消息"
  },
  "msg_type": "text",
  "created_at": "2024-01-01T14:00:00Z"
}
```

### 发送JSON格式的复杂内容

```bash
curl -X POST http://localhost:8080/api/v1/message/send \
  -H "Authorization: Bearer $BOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "to_id": "b_550e8400-e29b-41d4-a716-446655440003",
    "content": {
      "type": "order",
      "order_id": "ORD123456",
      "amount": 99.99,
      "items": ["item1", "item2"]
    },
    "msg_type": "text"
  }'
```

## 👤 账户管理

### 取消注册用户账户

取消注册将删除用户及其所有Bot。**此操作不可逆**。

```bash
curl -X DELETE http://localhost:8080/api/v1/users/delete \
  -H "Authorization: Bearer $BOT_TOKEN"
```

**响应示例：**
```json
{
  "message": "用户账户已删除",
  "user_id": "u_550e8400-e29b-41d4-a716-446655440000"
}
```

## 🔒 安全最佳实践

### 1. Token管理

❌ **不要做：**
```python
# 不要在代码中硬编码Token
TOKEN = "b_550e8400-e29b-41d4-a716-446655440001:1637000000:signature..."
```

✅ **应该做：**
```python
import os
from dotenv import load_dotenv

load_dotenv()
TOKEN = os.getenv("BOT_TOKEN")
```

### 2. HTTPS传输

在生产环境中，始终使用HTTPS：
```bash
# 开发环境
curl http://localhost:8080/api/v1/...

# 生产环境
curl https://api.example.com/api/v1/...
```

### 3. Token刷新

Token有24小时的有效期。在Token过期前更新为新Token：

```python
import requests
import os
from datetime import datetime, timedelta

TOKEN = os.getenv("BOT_TOKEN")
TOKEN_GENERATED_AT = datetime.now()
TOKEN_EXPIRY_HOURS = 24

def ensure_token_fresh():
    global TOKEN
    elapsed = datetime.now() - TOKEN_GENERATED_AT
    if elapsed > timedelta(hours=TOKEN_EXPIRY_HOURS - 1):
        # 创建新Bot获取新Token
        # 或使用刷新机制（待实现）
        print("Token将要过期，请重新注册或创建新Bot")
```

### 4. 错误处理

```python
import requests

def send_message_safely(bot_token, recipient_id, content):
    try:
        response = requests.post(
            "http://localhost:8080/api/v1/message/send",
            headers={
                "Authorization": f"Bearer {bot_token}",
                "Content-Type": "application/json"
            },
            json={
                "to_id": recipient_id,
                "content": {"text": content},
                "msg_type": "text"
            },
            timeout=10
        )
        response.raise_for_status()
        return response.json()
    except requests.exceptions.ConnectionError:
        print("连接失败：服务器无响应")
    except requests.exceptions.HTTPError as e:
        if e.response.status_code == 401:
            print("认证失败：Token无效或已过期")
        elif e.response.status_code == 404:
            print("找不到接收方Bot")
        else:
            print(f"API错误: {e.response.json()}")
    except Exception as e:
        print(f"未知错误: {e}")
```

## 📚 Python集成示例

### 完整的Bot客户端

```python
import requests
import json
from typing import Dict, Any, Optional

class ChatBotClient:
    def __init__(self, server_url: str, bot_token: str):
        self.server_url = server_url
        self.bot_token = bot_token
        self.session = requests.Session()
        self.session.headers.update({
            "Authorization": f"Bearer {bot_token}",
            "Content-Type": "application/json"
        })

    def send_message(self, to_id: str, content: Dict[str, Any]) -> Dict:
        """发送消息"""
        response = self.session.post(
            f"{self.server_url}/api/v1/message/send",
            json={
                "to_id": to_id,
                "content": content,
                "msg_type": "text"
            }
        )
        response.raise_for_status()
        return response.json()

    def list_bots(self) -> Dict:
        """列出所有Bot"""
        response = self.session.get(
            f"{self.server_url}/api/v1/bots"
        )
        response.raise_for_status()
        return response.json()

    def create_bot(self, name: str, description: Optional[str] = None) -> Dict:
        """创建新Bot"""
        response = self.session.post(
            f"{self.server_url}/api/v1/bots",
            json={
                "name": name,
                "description": description
            }
        )
        response.raise_for_status()
        return response.json()

    def delete_bot(self, bot_id: str) -> Dict:
        """删除Bot"""
        response = self.session.delete(
            f"{self.server_url}/api/v1/bots/{bot_id}"
        )
        response.raise_for_status()
        return response.json()

    def delete_account(self) -> Dict:
        """删除账户"""
        response = self.session.delete(
            f"{self.server_url}/api/v1/users/delete"
        )
        response.raise_for_status()
        return response.json()

# 使用示例
if __name__ == "__main__":
    client = ChatBotClient(
        server_url="http://localhost:8080",
        bot_token="b_xxx:timestamp:signature"
    )

    # 列出所有Bot
    bots = client.list_bots()
    print("我的Bot:", bots)

    # 发送消息
    message = client.send_message(
        to_id="b_yyy:timestamp:signature",
        content={"text": "你好！"}
    )
    print("消息已发送:", message)

    # 创建新Bot
    new_bot = client.create_bot("新Bot", "这是一个新Bot")
    print("新Bot已创建:", new_bot)
```

## 🚨 常见问题解决

### Q: Token显示"invalid or expired"
**A:** 可能的原因：
1. Token格式不正确
2. Token已过期（24小时）
3. Token被篡改

解决方案：
- 确保Token格式正确：`{bot_id}:{timestamp}:{signature}`
- 创建新Bot获取新Token
- 检查Token是否完整复制

### Q: 收到"Recipient bot not found"错误
**A:** 接收方Bot不存在。请确保：
1. Bot ID正确
2. Bot仍然存在（未被删除）
3. 使用的是Bot ID而不是User ID

### Q: 如何在多个Bot之间转移所有权？
**A:** 目前系统不支持Bot所有权转移。建议方案：
1. 创建新的用户账户
2. 其他用户创建Bot
3. 使用Bot ID进行通信

### Q: Token泄露了怎么办？
**A:** 立即删除泄露的Bot，然后创建新的Bot：

```bash
# 删除泄露的Bot
curl -X DELETE http://localhost:8080/api/v1/bots/{泄露的bot_id} \
  -H "Authorization: Bearer {另一个有效的token}"

# 创建新的Bot
curl -X POST http://localhost:8080/api/v1/bots \
  -H "Authorization: Bearer {有效的token}" \
  -H "Content-Type: application/json" \
  -d '{"name": "新Bot"}'
```

## 📞 API速查表

| 操作 | 方法 | 端点 | 认证 |
|------|------|------|------|
| 注册用户 | POST | `/api/v1/auth/register` | ❌ |
| 创建Bot | POST | `/api/v1/bots` | ✅ |
| 列出Bot | GET | `/api/v1/bots` | ✅ |
| 获取Bot详情 | GET | `/api/v1/bots/{bot_id}` | ❌ |
| 删除Bot | DELETE | `/api/v1/bots/{bot_id}` | ✅ |
| 发送消息 | POST | `/api/v1/message/send` | ✅ |
| 删除账户 | DELETE | `/api/v1/users/delete` | ✅ |

## 🎓 进阶功能

### 消息分页查询
（待实现）

### WebSocket实时通知
（待实现）

### 消息加密
（待实现）

### 机器人智能回复
（待实现）

## 📖 更多资源

- [API完整文档](./API_GUIDE.md)
- [系统架构说明](./ARCHITECTURE_REFORM.md)
- [项目README](./README.md)

## 🤝 获取帮助

- 查看服务器日志：`tail -f /tmp/server.log`
- 联系技术支持
- 查看GitHub Issues
