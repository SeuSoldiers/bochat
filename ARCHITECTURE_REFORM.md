# 聊天平台架构改革说明

## 改革概述

本次改革重新设计了聊天平台的身份认证和消息系统，实现了**用户-Bot分离模式**。

### 改革前的问题
- 用户通过用户名和密码认证，直接对应一个个人Bot
- 一个用户只能有一个Bot
- 系统设计不够灵活

### 改革后的设计
- 用户通过实名认证（身份证号）进行身份验证
- 用户可以创建和管理多个Bot
- 只有Bot才能通过API发送消息
- 支持不同Bot之间的消息交互

## 核心概念

### 1. 用户（User）

**身份认证方式：**
- 使用身份证号进行实名认证（18位数字）
- 身份证号全局唯一，不可重复注册
- 注册时提供：姓名、身份证号、手机号

**用户的作用：**
- 作为Bot的所有者和管理者
- 可以创建、查看和管理多个Bot
- 与具体的消息发送解耦

**数据模型：**
```rust
pub struct User {
    pub user_id: String,           // u_{uuid}
    pub name: String,              // 用户姓名
    pub id_number: String,         // 身份证号（18位）
    pub phone: String,             // 手机号
    pub created_at: String,        // 创建时间
    pub updated_at: String,        // 更新时间
}
```

### 2. Bot（机器人）

**概念定义：**
Bot是真正的API消息发送者。每个Bot：
- 有独立的Bot ID和Token
- 有自己的状态（active/inactive）
- 属于某个用户（通过owner_id关联）
- 可以接收和发送消息

**一个User可以有多个Bot的场景：**
- 同一家公司创建多个客服Bot（对应不同的部门）
- 创建不同用途的Bot（客服、营销、技术支持等）
- A用户可以给B用户委托管理一个Bot

**数据模型：**
```rust
pub struct Bot {
    pub bot_id: String,           // b_{uuid}
    pub owner_id: String,         // 所属用户ID
    pub name: String,             // Bot名称
    pub description: Option<String>, // Bot描述
    pub status: String,           // active / inactive
    pub token: String,            // 认证Token，格式：{bot_id}:{timestamp}:{signature}
    pub secret: String,           // Bot密钥，用于生成Token签名
    pub created_at: String,       // 创建时间
    pub updated_at: String,       // 更新时间
}
```

### 3. 消息（Message）

**消息发送规则：**
- 只有Bot才能发送消息
- 消息的sender_id是Bot ID
- 消息的to_id也是Bot ID（接收方）
- 实现Bot到Bot的通信

**数据模型：**
```rust
pub struct Message {
    pub msg_id: i64,              // 消息ID
    pub sender_id: String,        // 发送方Bot ID
    pub to_id: String,            // 接收方Bot ID
    pub content: String,          // 消息内容（JSON）
    pub msg_type: String,         // 消息类型（text/file）
    pub created_at: String,       // 创建时间
}
```

## API 改革

### 移除的端点
```
POST /api/v1/auth/login           # 移除登录接口
```

### 新增端点

#### 1. Bot管理

```bash
# 创建Bot
POST /api/v1/bots
Authorization: Bearer {bot_token}
Content-Type: application/json

{
  "name": "Bot名称",
  "description": "Bot描述（可选）"
}
```

```bash
# 查询用户的所有Bot
GET /api/v1/bots
Authorization: Bearer {bot_token}
```

```bash
# 获取Bot详情
GET /api/v1/bots/{bot_id}
```

### 改变的端点

#### 1. 用户注册

**改革前：**
```bash
POST /api/v1/auth/register
{
  "username": "alice",
  "email": "alice@example.com",
  "password": "password123"
}
```

**改革后：**
```bash
POST /api/v1/auth/register
{
  "name": "Alice",
  "id_number": "110101199003071234",  # 18位身份证号
  "phone": "13800138000"
}
```

**返回变化：**
- 新增：user_id, bot_id, bot_token
- 移除：email, password_hash相关信息
- 响应状态码：201 Created

## 数据库改革

### 用户表（users）

**改革前：**
```sql
CREATE TABLE users (
    user_id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME,
    updated_at DATETIME
);
```

**改革后：**
```sql
CREATE TABLE users (
    user_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    id_number TEXT NOT NULL UNIQUE,
    phone TEXT NOT NULL,
    created_at DATETIME,
    updated_at DATETIME
);

CREATE INDEX idx_users_id_number ON users(id_number);
```

### Bot表（bots）

**改革前：**
```sql
CREATE TABLE bots (
    bot_id TEXT PRIMARY KEY,
    bot_type TEXT NOT NULL,          -- 'personal' 或 'custom'
    owner_id TEXT NOT NULL,
    name TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    created_at DATETIME,
    FOREIGN KEY (owner_id) REFERENCES users(user_id)
);
```

**改革后：**
```sql
CREATE TABLE bots (
    bot_id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    token TEXT NOT NULL UNIQUE,
    secret TEXT NOT NULL,             -- 新增：用于Token签名
    created_at DATETIME,
    updated_at DATETIME,              -- 新增
    FOREIGN KEY (owner_id) REFERENCES users(user_id)
);

CREATE INDEX idx_bots_owner_id ON bots(owner_id);
```

### 消息表（messages）

**保持不变：**
消息表的结构保持一致，sender_id和to_id都是Bot ID。

## 认证流程改革

### 旧流程（用户名+密码）
```
用户 → 输入用户名和密码 → 登录接口 → 生成Token → 返回Bot Token
```

### 新流程（实名认证）
```
用户 → 输入身份证号 → 注册接口 → 自动创建默认Bot → 返回Bot Token
       ↓
     创建更多Bot → Bot列表接口 → 获取Bot Token
       ↓
     使用Bot Token → API请求 → 使用对应Bot发送消息
```

## 错误处理改革

### 新增错误类型
```rust
#[error("Invalid ID number")]
InvalidIdNumber,                  // 身份证号位数不正确

#[error("ID number already exists")]
IdNumberConflict,                 // 身份证号已被注册

#[error("Bot permission denied")]
BotPermissionDenied,              // Bot权限不足
```

### 移除错误类型
```rust
#[error("Invalid credentials")]
InvalidCredentials,               // 已移除

#[error("Username already exists")]
UsernameConflict,                 // 已移除

#[error("Email already exists")]
EmailConflict,                    // 已移除
```

## 向前兼容性

**本次改革不考虑向前兼容性。**

已部署的系统如需升级，需要进行数据迁移：
1. 为每个旧用户生成一个身份证号（如使用用户ID的哈希值）
2. 创建新的user表记录
3. 迁移bot表关联关系
4. 测试消息系统是否正常工作

## 安全性改进

### 1. 实名认证
- 身份证号作为唯一标识，增强了用户信息的真实性
- 减少了虚假账户的创建

### 2. 多Bot支持
- 一个Bot泄露不会影响用户的其他Bot
- 可以轻易禁用和删除单个Bot

### 3. 令牌分离
- 每个Bot有独立的Token和Secret
- 提高了安全性和灵活性

### 4. 状态管理
- Bot可以被禁用而不是删除
- 便于审计和恢复

## 迁移指南

### 对于新用户
直接使用新的API进行注册和使用。

### 对于现有用户
如果有现有的chat_platform.db数据库，需要重新初始化：

```bash
# 备份旧数据库
cp chat_platform.db chat_platform.db.backup

# 删除旧数据库
rm chat_platform.db

# 重新启动服务器，自动创建新表结构
cargo run --release
```

## 测试用例

使用提供的测试脚本验证新系统：

```bash
# 安装依赖
pip install requests

# 运行测试
python3 scripts/test_new_architecture.py
```

## API变化总结

| 功能 | 旧API | 新API | 变化说明 |
|-----|-------|-------|--------|
| 注册用户 | POST /api/v1/auth/register | POST /api/v1/auth/register | 参数改为身份证认证 |
| 用户登录 | POST /api/v1/auth/login | ❌ 移除 | 不再需要登录接口 |
| 创建Bot | N/A | POST /api/v1/bots | 新增 |
| 查询Bot | N/A | GET /api/v1/bots | 新增 |
| 获取Bot详情 | N/A | GET /api/v1/bots/{bot_id} | 新增 |
| 发送消息 | POST /api/v1/message/send | POST /api/v1/message/send | Token认证方式改为Bot Token |

## 常见问题

### Q: 一个用户可以有多少个Bot？
A: 目前没有限制，但建议根据实际需求合理创建。

### Q: Bot Token泄露了怎么办？
A: 可以删除该Bot并创建一个新的Bot，立即切换到新Token使用。

### Q: 身份证号存储安全吗？
A: 建议在生产环境中：
- 使用HTTPS加密传输
- 在数据库中加密存储
- 限制访问权限
- 定期审计

### Q: 如何实现Bot到User的消息通知？
A: 当Bot接收到消息时，可以：
1. 通过WebSocket实时推送
2. 通过Webhook回调通知
3. 用户定期轮询获取消息

### Q: 支持删除Bot吗？
A: 当前版本支持通过status字段禁用Bot。完整的删除功能可在后续版本实现。

## 后续改进方向

1. ✅ **Bot状态管理**：完全实现enable/disable/delete操作
2. ✅ **权限管理**：支持Bot之间的权限分组
3. ✅ **消息查询**：实现消息历史查询接口
4. ✅ **WebSocket实时通知**：实时推送Bot消息
5. ✅ **Webhook**：支持Bot消息回调
6. ✅ **审计日志**：记录所有操作日志
7. ✅ **速率限制**：按Bot进行速率限制
