# Chat Platform API Guide

## 系统架构

本系统采用**用户-Bot管理模式**：
- **用户**：使用身份证号进行实名认证，用于管理多个Bot
- **Bot**：实际的API消息发送者，每个Bot有独立的Token和Secret

## 认证流程

### 1. 用户注册

**请求：**
```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "张三",
    "id_number": "110101199003071234",
    "phone": "13800138000"
  }'
```

**参数说明：**
- `name`: 用户姓名（必需）
- `id_number`: 身份证号，18位数字（必需，已注册的ID号无法重复注册）
- `phone`: 手机号码（必需）

**响应（201 Created）：**
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

**说明：**
- 注册成功后，自动为用户创建一个默认Bot
- 返回的`bot_token`用于API请求认证
- Bot ID和Token应妥善保管

## Bot管理

### 1. 创建新Bot

**请求：**
```bash
curl -X POST http://localhost:8080/api/v1/bots \
  -H "Authorization: Bearer {bot_token}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "我的客服Bot",
    "description": "用于处理客户服务的机器人"
  }'
```

**参数说明：**
- `name`: Bot名称（必需）
- `description`: Bot描述（可选）

**响应（201 Created）：**
```json
{
  "bot_id": "b_550e8400-e29b-41d4-a716-446655440002",
  "owner_id": "u_550e8400-e29b-41d4-a716-446655440000",
  "name": "我的客服Bot",
  "description": "用于处理客户服务的机器人",
  "status": "active",
  "token": "b_550e8400-e29b-41d4-a716-446655440002:1637000000:signature...",
  "created_at": "2024-01-01T13:00:00Z",
  "updated_at": "2024-01-01T13:00:00Z"
}
```

### 2. 查询用户的所有Bot

**请求：**
```bash
curl http://localhost:8080/api/v1/bots \
  -H "Authorization: Bearer {bot_token}"
```

**响应（200 OK）：**
```json
{
  "bots": [
    {
      "bot_id": "b_550e8400-e29b-41d4-a716-446655440001",
      "owner_id": "u_550e8400-e29b-41d4-a716-446655440000",
      "name": "默认Bot",
      "description": "注册时自动创建的默认机器人",
      "status": "active",
      "token": "b_550e8400-e29b-41d4-a716-446655440001:1637000000:signature...",
      "created_at": "2024-01-01T12:00:00Z",
      "updated_at": "2024-01-01T12:00:00Z"
    },
    {
      "bot_id": "b_550e8400-e29b-41d4-a716-446655440002",
      "owner_id": "u_550e8400-e29b-41d4-a716-446655440000",
      "name": "我的客服Bot",
      "description": "用于处理客户服务的机器人",
      "status": "active",
      "token": "b_550e8400-e29b-41d4-a716-446655440002:1637000000:signature...",
      "created_at": "2024-01-01T13:00:00Z",
      "updated_at": "2024-01-01T13:00:00Z"
    }
  ]
}
```

### 3. 获取Bot详情

**请求：**
```bash
curl http://localhost:8080/api/v1/bots/{bot_id}
```

**响应（200 OK）：**
```json
{
  "bot_id": "b_550e8400-e29b-41d4-a716-446655440002",
  "owner_id": "u_550e8400-e29b-41d4-a716-446655440000",
  "name": "我的客服Bot",
  "description": "用于处理客户服务的机器人",
  "status": "active",
  "token": "b_550e8400-e29b-41d4-a716-446655440002:1637000000:signature...",
  "created_at": "2024-01-01T13:00:00Z",
  "updated_at": "2024-01-01T13:00:00Z"
}
```

## 消息API

### 1. 发送消息

只有Bot才能通过此API发送消息。

**请求：**
```bash
curl -X POST http://localhost:8080/api/v1/message/send \
  -H "Authorization: Bearer {bot_token}" \
  -H "Content-Type: application/json" \
  -d '{
    "to_id": "b_550e8400-e29b-41d4-a716-446655440003",
    "content": {
      "text": "你好，这是一条测试消息"
    },
    "msg_type": "text"
  }'
```

**参数说明：**
- `to_id`: 接收方Bot的ID（必需）
- `content`: 消息内容，JSON格式（必需）
- `msg_type`: 消息类型，可选值："text"（默认），"file"

**响应（201 Created）：**
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

**错误响应：**
- `401 Unauthorized`: 未提供有效的Token
- `400 Bad Request`: 接收方Bot不存在或Bot未激活
- `403 Forbidden`: Bot权限不足

## 错误响应

### 常见错误代码

| 状态码 | 错误信息 | 说明 |
|--------|--------|------|
| 400 | Bad request | 请求参数不合法 |
| 400 | Invalid ID number | 身份证号位数不正确（需要18位） |
| 401 | Unauthorized | 未提供有效的认证Token |
| 403 | Unauthorized | Token无效或已过期 |
| 404 | User not found | 用户不存在 |
| 404 | Bot not found | Bot不存在 |
| 409 | ID number already exists | 身份证号已被注册 |
| 500 | Database error | 数据库错误 |

## 完整示例流程

### 场景：两个用户相互发送消息

```bash
# 1. 用户A注册
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "用户A",
    "id_number": "110101199003071234",
    "phone": "13800138001"
  }'
# 响应：user_a_id, bot_a_token

# 2. 用户B注册
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "用户B",
    "id_number": "110101199003071235",
    "phone": "13800138002"
  }'
# 响应：user_b_id, bot_b_token, bot_b_id

# 3. 用户A查询自己的所有Bot
curl http://localhost:8080/api/v1/bots \
  -H "Authorization: Bearer bot_a_token"
# 获取bot_a_id

# 4. 用户A的Bot向用户B的Bot发送消息
curl -X POST http://localhost:8080/api/v1/message/send \
  -H "Authorization: Bearer bot_a_token" \
  -H "Content-Type: application/json" \
  -d '{
    "to_id": "bot_b_id",
    "content": {"text": "你好，用户B"}
  }'

# 5. 用户B的Bot向用户A的Bot发送回复
curl -X POST http://localhost:8080/api/v1/message/send \
  -H "Authorization: Bearer bot_b_token" \
  -H "Content-Type: application/json" \
  -d '{
    "to_id": "bot_a_id",
    "content": {"text": "你好，用户A"}
  }'
```

## 安全提示

1. **Token管理**：不要在代码中硬编码Bot Token，应使用环境变量或密钥管理系统
2. **身份证号**：身份证号和用户信息属于敏感数据，应在传输时使用HTTPS
3. **Token过期**：默认Token有效期为24小时，建议定期轮换
4. **API密钥**：Bot Secret应保密，不要在客户端代码中暴露

## 状态码说明

- **200 OK**: 请求成功
- **201 Created**: 资源创建成功
- **400 Bad Request**: 请求参数或格式错误
- **401 Unauthorized**: 缺少或无效的认证凭证
- **403 Forbidden**: 认证成功但权限不足
- **404 Not Found**: 请求的资源不存在
- **409 Conflict**: 资源冲突（如ID号重复）
- **500 Internal Server Error**: 服务器内部错误
