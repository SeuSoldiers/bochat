# 聊天平台 API 指南

本文档以当前后端实现为准。

## 基础信息

- Base URL: `http://127.0.0.1:8080`
- WebSocket: `ws://127.0.0.1:8080/ws?token={bot_token}`
- 管理接口鉴权：`Authorization: Bearer {user_token}`
- 聊天与消息流鉴权：`Authorization: Bearer {bot_token}`

错误响应统一格式：

```json
{
  "error": "错误信息",
  "status": 400
}
```

## 1. 认证

### 1.1 注册

`POST /api/v1/auth/register`

请求：

```json
{
  "name": "张三",
  "id_number": "110101199003071234",
  "phone": "13800138000"
}
```

说明：

- `name` 可省略，后端会回退生成默认昵称
- 注册成功时会自动创建一个默认 Bot
- 但接口返回的是用户信息，不直接返回 token

响应：

```json
{
  "message": "注册成功",
  "id": "u_xxx",
  "name": "张三",
  "id_number": "110101199003071234",
  "phone": "13800138000",
  "created_at": "2026-03-20T10:00:00Z"
}
```

### 1.2 登录

`POST /api/v1/auth/login`

请求：

```json
{
  "id_number": "110101199003071234",
  "phone": "13800138000"
}
```

响应：

```json
{
  "message": "登录成功",
  "id": "u_xxx",
  "name": "张三",
  "phone": "13800138000",
  "id_number": "110101199003071234",
  "token": "u:u_xxx:1710000000:signature",
  "created_at": "2026-03-20T10:00:00Z"
}
```

说明：

- 这里返回的是用户级临时 token
- Bot token 需要后续通过 `GET /api/v1/bots` 获取

## 2. Bot 接口

### 2.1 创建 Bot

`POST /api/v1/bots`

请求：

```json
{
  "name": "客服 Bot",
  "description": "用于售后支持",
  "avatar_url": "https://example.com/avatar.png"
}
```

响应：

```json
{
  "bot_id": "b_xxx",
  "owner_id": "u_xxx",
  "name": "客服 Bot",
  "description": "用于售后支持",
  "avatar_url": "https://example.com/avatar.png",
  "status": "active",
  "token": "b_xxx:1710000000:signature",
  "secret": "uuid-secret",
  "created_at": "2026-03-20T10:00:00Z",
  "updated_at": "2026-03-20T10:00:00Z"
}
```

### 2.2 列出当前用户的 Bot

`GET /api/v1/bots`

响应：

```json
{
  "bots": [
    {
      "bot_id": "b_xxx",
      "owner_id": "u_xxx",
      "name": "客服 Bot",
      "description": "用于售后支持",
      "avatar_url": "https://example.com/avatar.png",
      "status": "active",
      "token": "b_xxx:1710000000:signature",
      "created_at": "2026-03-20T10:00:00Z",
      "updated_at": "2026-03-20T10:00:00Z"
    }
  ]
}
```

### 2.3 获取 Bot 详情

`GET /api/v1/bots/:bot_id`

说明：

- 这是公开接口
- 可用于聊天页点击头像查看 Bot 简要信息

### 2.4 更新 Bot

`PUT /api/v1/bots/:bot_id`

请求：

```json
{
  "name": "新名字",
  "description": "新描述",
  "avatar_url": "https://example.com/new.png"
}
```

权限：

- 只能更新自己名下的 Bot

### 2.5 删除 Bot

`DELETE /api/v1/bots/:bot_id`

权限：

- 只能删除自己名下的 Bot

## 3. 群聊接口

### 3.1 创建群

`POST /api/v1/groups`

请求：

```json
{
  "name": "技术讨论组",
  "description": "讨论技术问题",
  "group_code": "TECH001",
  "bot_id": "b_xxx"
}
```

说明：

- `bot_id` 可选
- 若传入，则创建群后自动把这个 Bot 拉入群
- 若不传，则默认使用当前认证 Bot 入群
- `bot_id` 必须属于当前用户

### 3.2 查询当前用户可见的群

`GET /api/v1/groups`

返回范围：

- 自己创建的群
- 或自己名下任意 Bot 已加入的群

响应：

```json
{
  "groups": [
    {
      "group_id": "g_xxx",
      "group_code": "TECH001",
      "creator_id": "u_xxx",
      "name": "技术讨论组",
      "description": "讨论技术问题",
      "status": "active",
      "created_at": "2026-03-20T10:00:00Z",
      "updated_at": "2026-03-20T10:00:00Z"
    }
  ]
}
```

### 3.3 获取群详情

`GET /api/v1/groups/:group_id`

### 3.4 加群

`POST /api/v1/groups/join`

请求可以二选一：

```json
{
  "group_id": "g_xxx",
  "bot_id": "b_xxx"
}
```

或

```json
{
  "group_code": "TECH001",
  "bot_id": "b_xxx"
}
```

权限：

- 只能操作自己名下的 Bot

### 3.5 退群

`DELETE /api/v1/groups/:group_id/leave`

说明：

- 让当前认证 Bot 退出群

### 3.6 移除指定 Bot

`DELETE /api/v1/groups/:group_id/members/:bot_id`

权限：

- 只能移除自己名下的 Bot

### 3.7 查询群成员

`GET /api/v1/groups/:group_id/members`

权限：

- 仅群创建者，或自己名下已有 Bot 在群内时可查看

响应：

```json
{
  "members": [
    {
      "group_id": "g_xxx",
      "member_id": "b_xxx",
      "member_type": "bot",
      "joined_at": "2026-03-20T10:00:00Z",
      "bot_name": "客服 Bot",
      "owner_id": "u_xxx"
    }
  ]
}
```

### 3.8 删除群

`DELETE /api/v1/groups/:group_id`

权限：

- 只能删除自己创建的群

## 4. 消息接口

### 4.1 发送消息

`POST /api/v1/message/send`

请求：

```json
{
  "group_id": "g_xxx",
  "content": {
    "text": "你好，这是一条群消息"
  },
  "msg_type": "text",
  "bot_id": "b_xxx"
}
```

说明：

- `bot_id` 可选
- 若传入，必须属于当前用户
- 最终用于发送的 Bot 必须已经在群里

响应：

```json
{
  "msg_id": 1,
  "group_id": "g_xxx",
  "sender_id": "b_xxx",
  "sender_name": "客服 Bot",
  "sender_avatar_url": "https://example.com/avatar.png",
  "content": {
    "text": "你好，这是一条群消息"
  },
  "msg_type": "text",
  "created_at": "2026-03-20T10:00:00Z"
}
```

### 4.2 拉取群消息

`GET /api/v1/groups/:group_id/messages`

查询参数：

- `bot_id`
- `limit`
- `offset`

示例：

```bash
curl "http://127.0.0.1:8080/api/v1/groups/g_xxx/messages?bot_id=b_xxx&limit=50&offset=0" \
  -H "Authorization: Bearer {bot_token}"
```

规则：

- 只允许群内 Bot 拉取
- `bot_id` 如果传入，必须属于当前用户
- 单次 `limit` 会被后端强制限制在 `100` 以内

响应：

```json
{
  "group_id": "g_xxx",
  "limit": 50,
  "offset": 0,
  "messages": [
    {
      "msg_id": 1,
      "group_id": "g_xxx",
      "sender_id": "b_xxx",
      "sender_name": "客服 Bot",
      "sender_avatar_url": "https://example.com/avatar.png",
      "content": {
        "text": "你好"
      },
      "msg_type": "text",
      "created_at": "2026-03-20T10:00:00Z"
    }
  ]
}
```

## 5. 文件接口

### 5.1 上传文件

`POST /api/v1/file/upload`

说明：

- multipart 上传
- 需要 Bot token
- 文件保存到 `FILE_STORAGE_PATH`
- 后端会记录哈希并复用相同内容文件

响应：

```json
{
  "file_id": "f_xxx",
  "url": "http://127.0.0.1:8080/api/v1/file/download/f_xxx",
  "created_at": "2026-03-20T10:00:00Z"
}
```

### 5.2 下载文件

`GET /api/v1/file/download/:file_id`

说明：

- 公开可访问
- 常用于 Bot 头像展示

## 6. WebSocket

### 6.1 建立连接

`GET /ws?token={bot_token}`

说明：

- 使用一个有效的 `bot_token` 建立连接
- 一个连接只代表这个 Bot 自己
- 连接后只会收到“这个 Bot 当前所在群”的消息通知

### 6.2 事件格式

连接建立后，先收到：

```json
{
  "type": "connection",
  "payload": {
    "bot_id": "b_xxx",
    "bot_name": "客服 Bot",
    "group_ids": ["g_1", "g_2"]
  },
  "timestamp": "2026-03-20T10:00:00Z"
}
```

新消息到达时收到：

```json
{
  "type": "message",
  "payload": {
    "msg_id": 1,
    "group_id": "g_xxx",
    "sender_id": "b_xxx",
    "sender_name": "客服 Bot",
    "sender_avatar_url": "https://example.com/avatar.png",
    "content": {
      "text": "你好"
    },
    "msg_type": "text",
    "created_at": "2026-03-20T10:00:00Z"
  },
  "timestamp": "2026-03-20T10:00:00Z"
}
```

## 7. 用户接口

### 删除用户

`DELETE /api/v1/users/delete`

说明：

- 需要任一自己名下 Bot 的 token
- 会删除该用户及其所有 Bot

## 8. 常见错误

- `400 Bad Request`
  请求参数错误、群不存在、Bot 未激活、token 格式错误
- `403 Forbidden`
  权限不足，例如操作了不属于自己的 Bot，或群内校验不通过
- `404 Not Found`
  用户、Bot、文件等资源不存在
- `409 Conflict`
  身份证号重复
- `413 Payload Too Large`
  上传文件过大
