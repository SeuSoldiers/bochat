# 聊天平台 API 指南

本文档已按当前后端代码实现同步更新，基于仓库内路由与处理器行为整理。

## 基础信息

- Base URL: `http://127.0.0.1:8080`
- WebSocket: `ws://127.0.0.1:8080/ws`（请求头：`Authorization: Bearer {bot_token}`）
- 用户管理类接口鉴权：`Authorization: Bearer {user_token}`
- Bot 消息类接口鉴权：`Authorization: Bearer {bot_token}`

统一错误响应：

```json
{
  "code": "bad_request",
  "message": "错误信息",
  "status": 400
}
```

## 接口目录（可跳转）

| 模块 | 接口 URL | 作用 | 快速跳转 |
| --- | --- | --- | --- |
| 认证与用户 | `POST /api/v1/auth/register` | 注册账号并返回 user token | [1.1 注册](#api-auth-register) |
| 认证与用户 | `POST /api/v1/auth/login` | 账号登录并返回 user token | [1.2 登录](#api-auth-login) |
| 认证与用户 | `GET /api/v1/users/me` | 获取当前用户资料 | [1.3 获取当前用户信息](#api-users-me-get) |
| 认证与用户 | `PUT /api/v1/users/me` | 更新当前用户资料/密码/头像 | [1.4 更新当前用户信息](#api-users-me-put) |
| 认证与用户 | `DELETE /api/v1/users/delete` | 删除当前用户及其名下 Bot | [1.5 删除当前用户](#api-users-delete) |
| Bot | `POST /api/v1/bots` | 创建 Bot | [2.1 创建 Bot](#api-bots-create) |
| Bot | `GET /api/v1/bots` | 列出可见 Bot | [2.2 列出 Bot](#api-bots-list) |
| Bot | `GET /api/v1/bots/:bot_id` | 获取 Bot 详情 | [2.3 获取 Bot 详情](#api-bots-get) |
| Bot | `PUT /api/v1/bots/:bot_id` | 更新 Bot 信息 | [2.4 更新 Bot](#api-bots-update) |
| Bot | `DELETE /api/v1/bots/:bot_id` | 删除 Bot | [2.5 删除 Bot](#api-bots-delete) |
| Bot | `GET /api/v1/bots/search?bot_id=b_xxx` | 按编号精确搜索 Bot | [2.6 按编号搜索 Bot](#api-bots-search) |
| 群聊 | `POST /api/v1/groups` | 创建群并可选拉 Bot 入群 | [3.1 创建群](#api-groups-create) |
| 群聊 | `GET /api/v1/groups` | 查询当前用户可见群 | [3.2 查询当前用户可见的群](#api-groups-list) |
| 群聊 | `GET /api/v1/groups/:group_id` | 获取群详情 | [3.3 获取群详情](#api-groups-get) |
| 群聊 | `POST /api/v1/groups/join` | Bot 通过 `group_id/group_code` 加群 | [3.4 加群](#api-groups-join) |
| 群聊 | `DELETE /api/v1/groups/:group_id/leave?bot_id=b_xxx` | 指定 Bot 退群 | [3.5 退群](#api-groups-leave) |
| 群聊 | `DELETE /api/v1/groups/:group_id/members/:bot_id` | 从群中移除指定 Bot | [3.6 移除指定 Bot](#api-groups-remove-member) |
| 群聊 | `GET /api/v1/groups/:group_id/members` | 查询群成员 | [3.7 查询群成员](#api-groups-members) |
| 群聊 | `DELETE /api/v1/groups/:group_id` | 删除群 | [3.8 删除群](#api-groups-delete) |
| 群聊 | `GET /api/v1/groups/search?group_code=TECH` | 按群号前缀搜索群 | [3.9 按群号前缀搜索群](#api-groups-search) |
| 群聊 | `PUT /api/v1/groups/:group_id` | 更新群信息 | [3.10 更新群](#api-groups-update) |
| 消息 | `POST /api/v1/message/send` | 发送消息（支持幂等） | [4.1 发送消息](#api-message-send) |
| 消息 | `GET /api/v1/groups/:group_id/messages` | 拉取群消息历史 | [4.2 拉取群消息](#api-groups-messages) |
| 文件 | `POST /api/v1/file/upload` | 上传文件 | [5.1 上传文件](#api-file-upload) |
| 文件 | `GET /api/v1/file/download/:file_id/:filename` | 下载文件 | [5.2 下载文件](#api-file-download) |
| 文件 | `DELETE /api/v1/file/:file_id` | 删除文件 | [5.3 删除文件](#api-file-delete) |
| WebSocket | `GET /ws` | 建立 Bot 事件连接 | [6.1 建立连接](#api-ws-connect) |

## 1. 认证与用户

<a id="api-auth-register"></a>
### 1.1 注册

`POST /api/v1/auth/register`

请求：

```json
{
  "name": "我的昵称",
  "account": "zhangsan_01",
  "password": "Passw0rd!"
}
```

规则：

- `account` 必填，长度 `4-32`，仅支持字母、数字、下划线
- `password` 必填，长度 `8-64`，必须包含字母和数字，仅支持可见 ASCII 字符
- `name` 可选；为空或省略时，后端自动生成 `用户-xxxxxxxx`
- 注册成功后会自动创建一个默认 Bot
- 返回用户级 token，不返回内部 `user_id`

响应：

```json
{
  "message": "注册成功",
  "name": "用户-1a2b3c4d",
  "account": "zhangsan_01",
  "token": "u:u_xxx:1710000000:signature",
  "created_at": "2026-03-20T10:00:00Z"
}
```

<a id="api-auth-login"></a>
### 1.2 登录

`POST /api/v1/auth/login`

请求：

```json
{
  "account": "zhangsan_01",
  "password": "Passw0rd!"
}
```

响应：

```json
{
  "message": "登录成功",
  "name": "张三",
  "token": "u:u_xxx:1710000000:signature"
}
```

说明：

- 返回的是用户级 token
- Bot token 需要后续调用 `GET /api/v1/bots` 获取

<a id="api-users-me-get"></a>
### 1.3 获取当前用户信息

`GET /api/v1/users/me`

响应：

```json
{
  "name": "张三",
  "avatar_url": null,
  "created_at": "2026-03-20T10:00:00Z",
  "updated_at": "2026-03-20T12:00:00Z"
}
```

<a id="api-users-me-put"></a>
### 1.4 更新当前用户信息

`PUT /api/v1/users/me`

请求：

```json
{
  "name": "新的昵称",
  "password": "NewPassw0rd!",
  "avatar_url": ""
}
```

规则：

- `name` 可选，但如果传入则不能为空字符串
- `password` 可选；若传入，需满足与注册相同的密码规则
- `avatar_url` 可选；传空字符串会清空该值

响应：

```json
{
  "message": "用户信息更新成功",
  "name": "新的昵称",
  "avatar_url": null,
  "created_at": "2026-03-20T10:00:00Z",
  "updated_at": "2026-03-20T12:30:00Z"
}
```

<a id="api-users-delete"></a>
### 1.5 删除当前用户

`DELETE /api/v1/users/delete`

说明：

- 需要 user token
- 会删除当前用户及其名下全部 Bot

响应：

```json
{
  "message": "用户账户已删除"
}
```

## 2. Bot 接口

<a id="api-bots-create"></a>
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

<a id="api-bots-list"></a>
### 2.2 列出 Bot

`GET /api/v1/bots`

说明：

- 普通用户仅看到自己名下 Bot
- 超级管理员可看到所有 Bot

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

<a id="api-bots-get"></a>
### 2.3 获取 Bot 详情

`GET /api/v1/bots/:bot_id`

说明：

- 公开接口
- 返回单个 Bot 信息

<a id="api-bots-update"></a>
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

- 仅 Bot 所有者可更新
- 超级管理员可跨用户管理

<a id="api-bots-delete"></a>
### 2.5 删除 Bot

`DELETE /api/v1/bots/:bot_id`

权限：

- 仅 Bot 所有者可删除
- 超级管理员可跨用户管理

响应：

```json
{
  "message": "Bot 删除成功",
  "bot_id": "b_xxx"
}
```

<a id="api-bots-search"></a>
### 2.6 按编号搜索 Bot

`GET /api/v1/bots/search?bot_id=b_xxx`

说明：

- 需要 user token
- `bot_id` 为精确匹配（非前缀匹配）
- 找到返回单元素数组，未找到返回空数组

响应：

```json
{
  "bots": [
    {
      "bot_id": "b_xxx",
      "owner_id": "u_xxx",
      "name": "客服 Bot",
      "avatar_url": "https://example.com/avatar.png",
      "status": "active"
    }
  ]
}
```

## 3. 群聊接口

<a id="api-groups-create"></a>
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

规则：

- `name` 必填
- `bot_id` 可选
- 如果传入 `bot_id`，该 Bot 会在建群后自动入群
- 如果省略 `bot_id`，后端会自动选择当前用户最早创建的活跃 Bot 入群
- `bot_id` 必须属于当前用户；超级管理员可代管

响应：

```json
{
  "group_id": "g_xxx",
  "group_code": "TECH001",
  "creator_id": "u_xxx",
  "name": "技术讨论组",
  "description": "讨论技术问题",
  "avatar_url": "https://example.com/group.png",
  "status": "active",
  "created_at": "2026-03-20T10:00:00Z",
  "updated_at": "2026-03-20T10:00:00Z"
}
```

<a id="api-groups-list"></a>
### 3.2 查询当前用户可见的群

`GET /api/v1/groups`

返回范围：

- 自己创建的群
- 或自己名下任意 Bot 已加入的群
- 超级管理员可见全部群

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
      "avatar_url": "https://example.com/group.png",
      "status": "active",
      "created_at": "2026-03-20T10:00:00Z",
      "updated_at": "2026-03-20T10:00:00Z"
    }
  ]
}
```

<a id="api-groups-get"></a>
### 3.3 获取群详情

`GET /api/v1/groups/:group_id`

说明：

- 当前实现为公开读取接口
- 群不存在时返回 `400`，消息文本为 `Group not found`

<a id="api-groups-join"></a>
### 3.4 加群

`POST /api/v1/groups/join`

请求支持两种方式，且 `bot_id` 可选：

```json
{
  "group_id": "g_xxx",
  "bot_id": "b_xxx"
}
```

或：

```json
{
  "group_code": "TECH001",
  "bot_id": "b_xxx"
}
```

规则：

- `group_id` 与 `group_code` 至少提供一个
- `bot_id` 可选；省略时后端会选择当前用户最早创建的活跃 Bot
- 允许以下任一条件时邀请指定 Bot 入群：
  - 当前用户是该 Bot 的所有者（超级管理员可代管）
  - 当前用户是该群创建者（可邀请其他用户的 Bot）

响应：

```json
{
  "message": "成功加入群聊",
  "group_id": "g_xxx",
  "bot_id": "b_xxx"
}
```

<a id="api-groups-leave"></a>
### 3.5 退群

`DELETE /api/v1/groups/:group_id/leave?bot_id=b_xxx`

规则：

- 必须使用 user token
- 查询参数 `bot_id` 为必填
- 只能让自己名下的 Bot 退群；超级管理员可代管

响应：

```json
{
  "message": "Successfully left group",
  "group_id": "g_xxx",
  "bot_id": "b_xxx"
}
```

<a id="api-groups-remove-member"></a>
### 3.6 移除指定 Bot

`DELETE /api/v1/groups/:group_id/members/:bot_id`

规则：

- 必须使用 user token
- 只能移除自己名下的 Bot；超级管理员可代管

响应：

```json
{
  "message": "Bot removed from group successfully",
  "group_id": "g_xxx",
  "bot_id": "b_xxx"
}
```

<a id="api-groups-members"></a>
### 3.7 查询群成员

`GET /api/v1/groups/:group_id/members`

权限：

- 群创建者可查看
- 自己名下已有 Bot 在群内时可查看
- 超级管理员可查看

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

<a id="api-groups-delete"></a>
### 3.8 删除群

`DELETE /api/v1/groups/:group_id`

规则：

- 仅群创建者可删除
- 超级管理员可删除任意群
- 删除时会先清理消息与成员，再删除群记录

响应：

```json
{
  "message": "Group deleted successfully",
  "group_id": "g_xxx"
}
```

<a id="api-groups-search"></a>
### 3.9 按群号前缀搜索群

`GET /api/v1/groups/search?group_code=TECH`

说明：

- 当前实现为公开接口
- `group_code` 必填，按前缀匹配（`LIKE prefix%`）

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
      "avatar_url": "https://example.com/group.png",
      "status": "active",
      "created_at": "2026-03-20T10:00:00Z",
      "updated_at": "2026-03-20T10:00:00Z"
    }
  ]
}
```

<a id="api-groups-update"></a>
### 3.10 更新群

`PUT /api/v1/groups/:group_id`

请求：

```json
{
  "name": "新群名",
  "description": "新简介",
  "group_code": "TECH002",
  "avatar_url": "https://example.com/new-group.png"
}
```

规则：

- `name` 必填且不能为空
- `group_code` 可选；若传空字符串会被视为不设置
- `group_code` 若与其他群重复会返回 `400`
- 仅群创建者可编辑；超级管理员可代管

响应：

```json
{
  "group_id": "g_xxx",
  "group_code": "TECH002",
  "creator_id": "u_xxx",
  "name": "新群名",
  "description": "新简介",
  "avatar_url": "https://example.com/new-group.png",
  "status": "active",
  "created_at": "2026-03-20T10:00:00Z",
  "updated_at": "2026-03-20T11:00:00Z"
}
```

## 4. 消息接口

<a id="api-message-send"></a>
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
  "idempotency_key": "msg-20260403-0001"
}
```

规则：

- 必须使用 Bot token
- `idempotency_key` 必填，不能为空
- 发送 Bot 由请求头 token 决定，请求体中不接受 `bot_id`
- Bot 必须在群里，除非该 Bot 具备全局群访问权限
- 同一个 Bot 在同一群里使用相同 `idempotency_key` 重试时，会直接返回已存在消息

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

<a id="api-groups-messages"></a>
### 4.2 拉取群消息

`GET /api/v1/groups/:group_id/messages`

查询参数：

- `base_id`：排他游标，返回 `msg_id < base_id` 的消息
- `limit`：单次条数，后端强制截断到 `1..=100`，默认 `50`

示例：

```bash
curl "http://127.0.0.1:8080/api/v1/groups/g_xxx/messages?base_id=1000&limit=50" \
  -H "Authorization: Bearer {bot_token}"
```

规则：

- 必须使用 Bot token
- 只有群内 Bot 才能拉取，除非该 Bot 具备全局群访问权限
- 如果不传 `base_id`，会从最新消息往前取
- 返回的 `messages` 已按 `msg_id` 升序排列

响应：

```json
{
  "group_id": "g_xxx",
  "base_id": 1000,
  "limit": 50,
  "next_base_id": 951,
  "messages": [
    {
      "msg_id": 952,
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

<a id="api-file-upload"></a>
### 5.1 上传文件

`POST /api/v1/file/upload`

规则：

- 使用 multipart，字段名为 `file`
- 必须使用 Bot token
- 文件大小受 `max_file_size_mb` 配置限制
- 后端按内容哈希去重，重复上传相同内容会复用已有文件

响应：

```json
{
  "file_id": "f_xxx",
  "filename": "avatar.png",
  "url": "http://127.0.0.1:8080/api/v1/file/download/f_xxx/avatar.png",
  "created_at": "2026-03-20T10:00:00Z"
}
```

<a id="api-file-download"></a>
### 5.2 下载文件

`GET /api/v1/file/download/:file_id/:filename`

说明：

- 公开接口
- `filename` 必须与服务端记录一致，否则返回错误

<a id="api-file-delete"></a>
### 5.3 删除文件

`DELETE /api/v1/file/:file_id`

规则：

- 必须使用 Bot token
- 仅允许删除“当前 Bot 上传过”的文件关联
- 如果最后一个上传者关系被移除，会同时物理删除文件

响应：

```json
{
  "file_id": "f_xxx",
  "uploader_removed": true,
  "physical_deleted": true
}
```

## 6. WebSocket

<a id="api-ws-connect"></a>
### 6.1 建立连接

`GET /ws`

规则：

- 使用有效的 Bot token 建立连接（请求头：`Authorization: Bearer {bot_token}`）
- 一个连接只代表一个 Bot
- 连接后会收到该 Bot 当前可访问群的消息
- 具备全局群访问权限的 Bot 会收到全部群的消息

### 6.2 连接事件

连接建立后首先收到：

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

### 6.3 消息事件

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

## 8. 常见错误

- `400 Bad Request`
  请求参数错误、群不存在、Bot 未激活、token 格式错误
- `403 Forbidden`
  权限不足，例如操作了不属于自己的 Bot，或群内校验不通过
- `404 Not Found`
  用户、Bot、文件等资源不存在
- `409 Conflict`
  账号冲突
- `413 Payload Too Large`
  上传文件过大

常见错误码：

- `user_token_required`
- `bot_token_required`
- `invalid_user_token`
- `invalid_bot_token`
- `account_conflict`
- `invalid_credentials`
- `bot_inactive`
- `file_too_large`
- `bot_ownership_mismatch`
- `bot_not_in_group`
