# 聊天平台后端

一个使用 Rust 构建的高性能、异步聊天平台后端，具有以下特点：

- **Rust + Actix-web**：高性能、异步网络框架
- **SQLite**：轻量级无服务器数据库持久化
- **WebSocket**：实时消息传递
- **身份解耦**：基于 Actor 模型的统一 Bot 身份
- **Token 认证**：基于 HMAC-SHA256 的 Token 系统
- **速率限制**：使用 Redis 的令牌桶算法
- **文件管理**：安全的文件上传/下载，带所有权跟踪

## 项目状态

**第1阶段**：✅ 项目初始化和基础框架
**第2阶段**：✅ P0 - 身份转换逻辑（核心）
**第3阶段**：🔄 P1 - 统一消息网关（进行中）
**第4阶段**：⏳ P2 - 文件关联和完成

## 架构概览

### 核心组件

1. **模型** (`src/models/`)
   - `User`：用户账户，具有唯一的用户名/邮箱
   - `Bot`：身份容器（个人或自定义）
   - `Message`：具有发送者/接收者跟踪的文本/文件消息
   - `File`：具有所有权和访问控制的文件元数据

2. **数据库** (`src/db/`)
   - SQLite 模式，具有自动迁移
   - 可配置限制的连接池
   - 优化性能的索引查询

3. **服务** (`src/services/`)
   - 业务逻辑层
   - 数据库查询和转换
   - 特定于服务的操作

4. **处理器** (`src/handlers/`)
   - HTTP 请求处理器
   - Token 验证和授权
   - 请求/响应序列化

5. **WebSocket** (`src/ws/`)
   - 实时传递的连接管理器
   - 向在线用户广播消息
   - 连接生命周期管理

6. **认证** (`src/utils/token.rs`)
   - Token 生成：`{bot_id}:{timestamp}:{signature}`
   - HMAC-SHA256 签名验证
   - 可配置的 Token 过期

## API 端点

### 认证

```
POST /api/v1/auth/register
  请求：{ "username": "...", "email": "...", "password": "..." }
  响应：{ "user_id": "...", "bot_id": "...", "token": "..." }

POST /api/v1/auth/login
  请求：{ "username": "...", "password": "..." }
  响应：{ "user_id": "...", "bot_id": "...", "token": "..." }
```

### 消息

```
POST /api/v1/message/send
  请求头：Authorization: Bearer {token}
  请求：{ "to_id": "...", "content": {...}, "msg_type": "text|file" }
  响应：{ "msg_id": ..., "sender_id": "...", "created_at": "..." }
```

### 文件

```
POST /api/v1/file/upload
  请求头：Authorization: Bearer {token}
  请求：FormData（文件）
  响应：{ "file_id": "...", "url": "..." }

GET /api/v1/file/download/{file_id}
  请求头：Authorization: Bearer {token}
  响应：文件内容
```

### WebSocket

```
WS /ws?token={token}
  消息：{ "type": "message", "msg_id": ..., "sender_id": "...", "content": {...} }
```

## 开发设置

### 前置需求

- Rust 1.70+（从 https://rustup.rs/ 安装）
- SQLite3（通常已包含）
- Redis 6.0+（可选，用于高级速率限制）

### 安装

1. 克隆仓库：
```bash
git clone <repository-url>
cd rust-bochat
```

2. 创建环境文件：
```bash
cp .env.example .env
```

3. 使用你的配置更新 `.env`：
```env
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DATABASE_URL=sqlite:chat_platform.db
JWT_SECRET=your-super-secret-key
```

4. 构建并运行：
```bash
cargo build --release
cargo run --release
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试文件
cargo test --test token_tests

# 运行并显示输出
cargo test -- --nocapture
```

## 项目结构

```
chat-platform-rs/
├── Cargo.toml              # 依赖和元数据
├── src/
│   ├── main.rs             # 应用入口
│   ├── lib.rs              # 库根
│   ├── config.rs           # 配置管理
│   ├── error.rs            # 错误类型和处理
│   ├── db/                 # 数据库层
│   │   ├── mod.rs
│   │   ├── schema.rs       # 表定义
│   │   └── pool.rs         # 连接池
│   ├── models/             # 数据模型
│   │   ├── user.rs
│   │   ├── bot.rs
│   │   ├── message.rs
│   │   └── file.rs
│   ├── handlers/           # HTTP 处理器
│   │   ├── auth.rs
│   │   ├── message.rs
│   │   ├── file.rs
│   │   └── ws.rs
│   ├── services/           # 业务逻辑
│   │   ├── bot.rs
│   │   ├── message.rs
│   │   └── file.rs
│   ├── utils/              # 工具函数
│   │   ├── token.rs        # Token 生成/验证
│   │   └── id.rs           # ID 生成
│   ├── ws/                 # WebSocket
│   │   └── manager.rs
│   └── middlewares/        # HTTP 中间件
│       └── auth.rs
├── tests/                  # 集成测试
└── .env.example            # 配置示例
```

## 关键功能

### P0：身份解耦（✅ 已实现）

1. **用户注册**
   - 创建用户账户
   - 自动创建类型为 `personal` 的个人 Bot
   - 颁发初始访问 Token

2. **用户登录**
   - 验证凭据
   - 检索个人 Bot
   - 生成新的访问 Token

3. **Bot 身份**
   - 每个用户有一个个人 Bot（ID：`u_*`）
   - 用户可以创建自定义 Bot（ID：`b_*`）
   - 所有消息通过 Bot 身份发送

### P1：统一消息网关（🔄 进行中）

1. **消息发送**
   - 所有消息类型的统一 REST API
   - 基于 Bot 的认证
   - 消息类型支持：`text`、`file`
   - SQLite 中的持久存储

2. **消息路由**
   - 接收者 Bot 存在性验证
   - 消息格式标准化
   - 按接收者索引查询

### P2：文件管理（⏳ 计划中）

1. **文件上传**
   - 带认证的安全上传
   - 文件大小验证
   - 所有权跟踪
   - MIME 类型检测

2. **文件访问**
   - 带授权的下载
   - 消息中的引用验证
   - 用户删除时级联删除

## 配置

### 环境变量

```env
# 服务器
SERVER_HOST=127.0.0.1          # 绑定地址
SERVER_PORT=8080               # 绑定端口
SERVER_WORKERS=4               # 工作线程数

# 数据库
DATABASE_URL=sqlite:chat_platform.db
DB_MAX_CONNECTIONS=10
DB_MIN_CONNECTIONS=2

# Redis（可选）
REDIS_URL=redis://127.0.0.1:6379
REDIS_POOL_SIZE=10

# 安全
JWT_SECRET=your-secret-key     # Token 签名密钥
TOKEN_EXPIRY_SECS=86400        # Token TTL（24小时）

# 文件
MAX_FILE_SIZE_MB=100

# 速率限制
RATE_LIMIT_PER_SECOND=10

# 日志
RUST_LOG=info,chat_platform=debug
```

## 数据库模式

### users（用户表）
```sql
CREATE TABLE users (
  user_id TEXT PRIMARY KEY,
  username TEXT UNIQUE NOT NULL,
  email TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### bots（机器人表）
```sql
CREATE TABLE bots (
  bot_id TEXT PRIMARY KEY,
  bot_type TEXT NOT NULL,  -- 'personal' | 'custom'
  owner_id TEXT FOREIGN KEY,
  name TEXT NOT NULL,
  token TEXT UNIQUE NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### messages（消息表）
```sql
CREATE TABLE messages (
  msg_id INTEGER PRIMARY KEY AUTOINCREMENT,
  sender_id TEXT FOREIGN KEY,
  to_id TEXT FOREIGN KEY,
  content TEXT NOT NULL,
  msg_type TEXT DEFAULT 'text',
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_messages_to_id_created_at ON messages(to_id, created_at DESC);
```

### files（文件表）
```sql
CREATE TABLE files (
  file_id TEXT PRIMARY KEY,
  owner_id TEXT FOREIGN KEY,
  filename TEXT NOT NULL,
  size INTEGER NOT NULL,
  mime_type TEXT NOT NULL,
  storage_path TEXT NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## 错误处理

所有错误返回带状态码的结构化 JSON 响应：

```json
{
  "error": "错误描述",
  "status": 400
}
```

### HTTP 状态码

- **200 OK**：请求成功
- **201 Created**：资源已创建
- **400 Bad Request**：无效输入
- **401 Unauthorized**：无效/缺少凭据
- **403 Forbidden**：访问被拒绝
- **404 Not Found**：资源未找到
- **409 Conflict**：重复资源
- **429 Too Many Requests**：速率限制
- **500 Internal Server Error**：服务器错误

## 性能考虑

1. **数据库**
   - SQLite 连接池
   - 频繁访问列的索引查询
   - 用于 SQL 注入防护的预编译语句

2. **WebSocket**
   - 向在线连接高效广播消息
   - 自动清理已关闭的连接
   - 按 Bot 的连接管理

3. **速率限制**
   - Redis 中的令牌桶算法
   - 按 Bot 的速率限制
   - 可配置的限制

## 测试

### 单元测试
```bash
cargo test
```

### 集成测试
```bash
cargo test --test token_tests
```

### Token 测试
- Token 生成和验证
- 签名验证
- Token 格式和结构

## 安全

1. **密码哈希**：SHA256（生产中升级到 bcrypt/argon2）
2. **Token 签名**：带可配置密钥的 HMAC-SHA256
3. **授权**：基于 Token 的 Bearer 认证
4. **输入验证**：处理前验证所有输入
5. **SQL 注入防护**：参数化查询

## 日志

使用 `tracing` 的结构化日志：
```rust
tracing::info!("用户已注册：{}", user_id);
tracing::warn!("无效登录尝试：{}", username);
tracing::error!("数据库错误：{}", error);
```

使用 `RUST_LOG` 环境变量配置：
```bash
RUST_LOG=info,chat_platform=debug cargo run
```

## 未来增强

- [ ] 使用 actix-web-actors 的 WebSocket 实现
- [ ] 基于 Redis 的速率限制
- [ ] 带流的文件上传/下载
- [ ] 消息历史分页
- [ ] 用户在线状态
- [ ] 消息已读回执
- [ ] 群聊支持
- [ ] 消息加密
- [ ] 审计日志
- [ ] OpenAPI/Swagger API 文档

## 贡献

请确保所有代码：
- 编译时无警告：`cargo check`
- 通过所有测试：`cargo test`
- 正确格式化：`cargo fmt`
- 通过检查：`cargo clippy`

## 许可

MIT License

## 支持

如有问题和疑问，请在仓库上提出 issue。
