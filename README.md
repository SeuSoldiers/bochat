# 聊天平台 - Chat Platform

一个使用Rust + SQLite构建的高性能群聊平台后端，支持多Bot管理和实时消息系统。

## 🚀 特性

- **实名认证**：使用身份证号进行用户身份认证（18位数字）
- **多Bot管理**：用户可以创建和管理多个Bot
- **群聊系统**：用户创建群聊，Bot加入群聊后才能发送和查看消息
- **基于权限的消息访问**：只有群内的Bot才能查看群聊消息
- **高性能**：使用Rust和async/await实现高并发
- **类型安全**：使用SQLx进行编译时查询验证
- **RESTful API**：完整的API接口支持
- **WebSocket实时通信**：支持实时消息推送

## 📋 系统架构

### 核心概念

#### 用户（User）
- 使用身份证号进行实名认证
- 身份证号全局唯一
- 作为Bot的所有者和群聊的创建者
- 用户创建群聊，不直接参与群聊交互

#### 群聊（Group）
- 由用户创建
- 包含一个或多个Bot成员
- Bot必须先加入群聊才能与其他Bot通信
- 群聊有群ID（g_前缀）和自定义群号

#### Bot（机器人）
- 实际的消息发送和接收实体
- 每个Bot有独立的Token和Secret
- 支持active/inactive状态
- 需要加入群聊才能发送/查看消息

#### 消息（Message）
- Bot在群聊中的通信
- sender_id是Bot ID
- group_id确定消息所在的群聊
- 支持text/file类型
- 只有群内的Bot才能查看消息

## 🔧 快速开始

### 前置要求

- Rust 1.70+
- cargo
- Python 3.7+ (用于测试脚本)

### 安装

```bash
# 克隆项目
git clone <repository>
cd rust-bochat

# 复制环境配置
cp .env.example .env

# 构建项目
cargo build --release
```

### 启动服务器

```bash
cargo run --release
```

服务器将在 `http://127.0.0.1:8080` 启动

## 📡 API 使用示例

### 1. 用户注册

```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "张三",
    "id_number": "110101199003071234",
    "phone": "13800138000"
  }'
```

**响应示例**：
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

### 2. 创建群聊

```bash
curl -X POST http://localhost:8080/api/v1/groups \
  -H "Authorization: Bearer {bot_token}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "开发团队群",
    "description": "用于讨论开发事项",
    "group_code": "dev-team-001"
  }'
```

### 3. 创建新Bot

```bash
curl -X POST http://localhost:8080/api/v1/bots \
  -H "Authorization: Bearer {bot_token}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "我的客服Bot",
    "description": "用于处理客户服务的机器人"
  }'
```

### 4. Bot加入群聊

```bash
curl -X POST http://localhost:8080/api/v1/groups/join \
  -H "Authorization: Bearer {bot_token}" \
  -H "Content-Type: application/json" \
  -d '{
    "group_id": "g_550e8400-e29b-41d4-a716-446655440002",
    "group_code": "dev-team-001"
  }'
```

### 5. 在群聊中发送消息

```bash
curl -X POST http://localhost:8080/api/v1/messages \
  -H "Authorization: Bearer {bot_token}" \
  -H "Content-Type: application/json" \
  -d '{
    "group_id": "g_550e8400-e29b-41d4-a716-446655440002",
    "content": "大家好，这是一条群消息",
    "msg_type": "text"
  }'
```

### 6. 查看群聊消息（需要Bot在群内）

```bash
curl http://localhost:8080/api/v1/groups/{group_id}/messages \
  -H "Authorization: Bearer {bot_token}"
```

**重要**：只有群内的Bot才能查看群聊消息历史。Bot必须先加入群聊才能发送和查看消息。

### 7. 查询用户的所有群聊

```bash
curl http://localhost:8080/api/v1/groups \
  -H "Authorization: Bearer {bot_token}"
```

## 🧪 测试

使用提供的Python测试脚本：

```bash
# 安装依赖
pip install requests

# 运行测试
python3 scripts/test_chat.py
```

测试脚本会自动：
- 注册两个用户（Alice和Bob）
- 为每个用户创建多个Bot
- 创建测试群聊
- Bot加入群聊
- 在群聊中交换消息
- 验证只有群内Bot才能查看消息
- 显示测试结果

## 📁 项目结构

```
src/
├── main.rs              # 应用入口
├── lib.rs               # 库根模块
├── config.rs            # 配置管理
├── error.rs             # 错误处理
├── db/                  # 数据库模块
│   ├── mod.rs
│   ├── pool.rs         # 连接池
│   └── schema.rs       # 数据库Schema
├── models/              # 数据模型
│   ├── user.rs
│   ├── bot.rs
│   ├── message.rs
│   └── file.rs
├── handlers/            # HTTP处理器
│   ├── auth.rs         # 身份认证
│   ├── bot.rs          # Bot管理
│   ├── message.rs      # 消息发送
│   ├── file.rs         # 文件操作
│   └── ws.rs           # WebSocket
├── services/            # 业务逻辑
├── utils/               # 工具函数
│   ├── id.rs           # ID生成
│   └── token.rs        # Token管理
├── middlewares/         # 中间件
└── ws/                  # WebSocket管理
```

## 🔐 安全特性

### 实名认证
- 身份证号作为唯一标识
- 增强用户信息真实性
- 防止虚假账户

### Token管理
- 每个Bot有独立Token和Secret
- Token格式：`{bot_id}:{timestamp}:{signature}`
- HMAC-SHA256签名验证

### Bot权限
- Bot状态管理（active/inactive）
- 细粒度访问控制
- 快速禁用和恢复

## 📊 数据库设计

### users表
```sql
CREATE TABLE users (
    user_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    id_number TEXT NOT NULL UNIQUE,
    phone TEXT NOT NULL,
    created_at DATETIME,
    updated_at DATETIME
);
```

### bots表
```sql
CREATE TABLE bots (
    bot_id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    token TEXT NOT NULL UNIQUE,
    secret TEXT NOT NULL,
    created_at DATETIME,
    updated_at DATETIME,
    FOREIGN KEY (owner_id) REFERENCES users(user_id)
);
```

### groups表
```sql
CREATE TABLE groups (
    group_id TEXT PRIMARY KEY,
    group_code TEXT NOT NULL,
    creator_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at DATETIME,
    updated_at DATETIME,
    FOREIGN KEY (creator_id) REFERENCES users(user_id)
);
```

### group_members表
```sql
CREATE TABLE group_members (
    group_id TEXT NOT NULL,
    member_id TEXT NOT NULL,
    member_type TEXT NOT NULL,
    joined_at DATETIME,
    PRIMARY KEY (group_id, member_id),
    FOREIGN KEY (group_id) REFERENCES groups(group_id)
);
```

### messages表
```sql
CREATE TABLE messages (
    msg_id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    content TEXT NOT NULL,
    msg_type TEXT DEFAULT 'text',
    created_at DATETIME,
    FOREIGN KEY (group_id) REFERENCES groups(group_id),
    FOREIGN KEY (sender_id) REFERENCES bots(bot_id)
);
```

## 🛠️ 环境配置

在`.env`文件中配置以下参数：

```env
# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
SERVER_WORKERS=4

# 数据库配置
DATABASE_URL=sqlite:chat.db
DB_MAX_CONNECTIONS=10
DB_MIN_CONNECTIONS=2

# 安全配置
JWT_SECRET=your-super-secret-key-change-in-production
TOKEN_EXPIRY_SECS=86400

# 文件上传配置
MAX_FILE_SIZE_MB=100

# 速率限制
RATE_LIMIT_PER_SECOND=10

# 日志
RUST_LOG=info,chat_platform=debug
```

## 📚 文档

- [API_GUIDE_CN.md](API_GUIDE_CN.md) - 完整API文档
- [ARCHITECTURE_REFORM.md](ARCHITECTURE_REFORM.md) - 架构改革说明

## 🐛 常见问题

### Q: 身份证号存储安全吗？
A: 建议在生产环境中：
- 使用HTTPS加密传输
- 在数据库中加密存储
- 限制访问权限
- 定期审计日志

### Q: 一个用户最多可以创建多少个群聊和Bot？
A: 目前没有限制，可根据实际需求合理管理。

### Q: Bot Token泄露了怎么办？
A: 可以删除该Bot并创建新的Bot，立即切换到新Token使用。

### Q: Bot如何查看群聊消息？
A: Bot必须先加入群聊（通过join接口），然后才能通过认证查看该群聊的消息历史。未加入的群聊返回403权限错误。

### Q: 只有群内Bot才能查看消息吗？
A: 是的。出于安全考虑，只有已加入该群聊的Bot才能查看群聊的消息历史。这防止了未授权的Bot访问群聊信息。

### Q: 支持WebSocket实时通知吗？
A: 当前版本有WebSocket框架，实时通知功能待实现。

### Q: 群聊和一对一消息有区别吗？
A: 是的。系统现在基于群聊模型，Bot必须先加入群聊才能交互。不再支持直接的一对一Bot消息。

## 🚀 后续改进方向

- [x] 群聊系统（Group Management）
- [x] 群组成员管理（Group Membership）
- [x] 消息查看权限鉴权（Message Access Control）
- [ ] 完整的Bot权限管理系统
- [ ] WebSocket实时推送
- [ ] Webhook回调通知
- [ ] 消息加密端到端
- [ ] 审计日志系统
- [ ] 按Bot速率限制
- [ ] 分布式消息队列集成

## 📝 许可证

MIT

## 👤 作者

Claude Code Assistant
