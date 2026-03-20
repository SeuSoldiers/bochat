# 聊天平台后端

一个基于 Rust、Axum 和 SQLite 的群聊后端，模型是“用户管理多个 Bot，Bot 作为群成员收发消息”。

## 当前能力

- 手机号 / 身份证号二选一注册与登录
- 多 Bot 管理
- 用户创建群聊，Bot 按群号或群 ID 入群
- 只有自己名下的 Bot 才能被拉入/移出群
- 只有群内 Bot 才能发消息、拉消息
- WebSocket 实时消息推送
- 文件上传、下载与去重复用
- 用户资料编辑与头像管理
- 公开查询 Bot 简要信息

## 技术栈

- HTTP / WebSocket: Axum
- 数据库: SQLite + SQLx
- 异步运行时: Tokio

## 启动

```bash
cargo run
```

默认监听：

- `http://127.0.0.1:8080`
- WebSocket: `ws://127.0.0.1:8080/ws?token=...`

## 关键环境变量

- `DATABASE_URL`
  默认：`sqlite://chat_platform.db`
- `FILE_STORAGE_PATH`
  默认：`./assets/files/`
- `TOKEN_EXPIRY_SECS`
  默认：`86400`
- `MAX_FILE_SIZE_MB`
  默认：`100`

## 认证模型

- 管理接口使用 `Authorization: Bearer {user_token}`
- 聊天 / 拉消息 / WebSocket 使用对应 `bot_token`
- token 格式：
  `{bot_id}:{timestamp}:{signature}`
- user token 格式：
  `u:{user_id}:{timestamp}:{signature}`
- 注册和登录都返回用户级临时 token
- 认证与用户资料响应不再暴露用户内部 `id`
- `name` 不强制填写，后端默认生成随机 UUID 风格昵称

## 主要接口

### 认证

- `POST /api/v1/auth/register`
- `POST /api/v1/auth/login`

注册返回示例：

```json
{
  "message": "注册成功",
  "name": "用户-1a2b3c4d",
  "phone": "13800138000",
  "token": "u:u_xxx:1710000000:signature",
  "created_at": "2026-03-20T10:00:00Z"
}
```

登录返回示例：

```json
{
  "message": "登录成功",
  "name": "张三",
  "phone": "13800138000",
  "token": "u:u_xxx:1710000000:signature"
}
```

注册和登录请求都支持这两种形式之一：

```json
{
  "phone": "13800138000"
}
```

```json
{
  "id_number": "110101199003071234"
}
```

也可以同时传两者；后端要求至少提供一项。

### 用户资料

- `GET /api/v1/users/me`
- `PUT /api/v1/users/me`

资料页支持字段：

- `name`
- `phone`
- `avatar_url`

规则：

- `name` 可修改，但不能为空字符串
- `phone` 为当前资料页可编辑的登录手机号
- 头像可以直接保存 URL，也可以先走文件上传接口拿到 URL

### Bot

- `POST /api/v1/bots`
- `GET /api/v1/bots`
- `GET /api/v1/bots/:bot_id`
- `PUT /api/v1/bots/:bot_id`
- `DELETE /api/v1/bots/:bot_id`

Bot 支持字段：

- `name`
- `description`
- `avatar_url`

### 群聊

- `POST /api/v1/groups`
- `GET /api/v1/groups`
- `GET /api/v1/groups/:group_id`
- `DELETE /api/v1/groups/:group_id`
- `POST /api/v1/groups/join`
- `DELETE /api/v1/groups/:group_id/leave`
- `GET /api/v1/groups/:group_id/members`
- `DELETE /api/v1/groups/:group_id/members/:bot_id`

创建群支持：

```json
{
  "name": "开发群",
  "description": "讨论开发问题",
  "group_code": "DEV001",
  "bot_id": "b_xxx"
}
```

加群支持按 `group_id` 或 `group_code`，并可显式指定自己的 Bot：

```json
{
  "group_code": "DEV001",
  "bot_id": "b_xxx"
}
```

### 消息

- `POST /api/v1/message/send`
- `GET /api/v1/groups/:group_id/messages`

发送消息：

```json
{
  "group_id": "g_xxx",
  "content": {
    "text": "你好"
  },
  "msg_type": "text",
  "bot_id": "b_xxx"
}
```

拉消息支持分页：

- `limit`
- `offset`
- `bot_id`

说明：

- 后端会强制 `limit <= 100`
- `bot_id` 用于指定“以自己名下哪个 Bot 的身份校验拉取权限”

### 文件

- `POST /api/v1/file/upload`
- `GET /api/v1/file/download/:file_id`

上传要求：

- 使用用户 token 鉴权
- multipart 表单上传
- 文件会落盘到 `FILE_STORAGE_PATH`
- 后端会计算文件哈希并复用相同内容文件

### WebSocket

- `GET /ws?token={bot_token}`

当前事件：

- `connection`
- `message`

说明：

- 连接使用单个 `bot_token`
- 一个 WebSocket 连接只代表这一个 Bot
- 该连接只会收到“这个 Bot 当前所在群”的实时消息
- 如果前端切换到另一个 Bot，需要用那个 Bot 的 token 重新建立连接

## 权限规则

- 用户只能删除自己的 Bot
- 用户只能更新自己的 Bot
- 用户只能把自己名下的 Bot 加入或移出群
- 用户只能删除自己创建的群
- 用户只能修改自己的资料
- 发消息时，`bot_id` 如果显式传入，必须属于当前用户且已经在群里
- 拉消息时，校验通过的 Bot 必须是群成员

## 测试

主流程测试已经迁到 Rust 集成测试，不再依赖旧 Python 接口脚本：

```bash
cargo test
```

关键测试：

- [chat_flow_integration.rs](/home/harkerhand/codes/rust-bochat/tests/chat_flow_integration.rs)

如果只想监听实时消息，可以用：

```bash
pip install websockets
python3 scripts/ws_monitor.py '<bot_token>'
```

## 文档索引

- [API指南.md](/home/harkerhand/codes/rust-bochat/API指南.md)
- [BOT接入指南.md](/home/harkerhand/codes/rust-bochat/BOT接入指南.md)
- [scripts/README.md](/home/harkerhand/codes/rust-bochat/scripts/README.md)
