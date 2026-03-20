# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

## 🚀 快速开始命令

### 构建与开发
```bash
# 检查代码而不构建（最快）
cargo check

# 以调试模式构建（编译快，运行慢）
cargo build

# 以发布模式构建（编译慢，运行快）
cargo build --release

# 运行应用程序
cargo run --release

# 格式化代码
cargo fmt

# 运行 clippy 代码检查
cargo clippy

# 运行测试
cargo test
```

### 数据库与迁移
```bash
# 数据库文件：chat_platform.db (SQLite)
# 迁移在启动时自动运行
# 重置数据库：rm chat_platform.db && cargo run
```

### 测试
```bash
# 运行综合测试脚本
python3 scripts/test_chat.py

# 启动后端 + UI
./start.sh          # Linux/macOS
./start.bat         # Windows
```

### 日志输出
```bash
# 显示调试日志
RUST_LOG=debug cargo run --release

# 显示特定模块的日志
RUST_LOG=chat_platform::handlers=debug cargo run --release
```

## 🏗️ 高层架构

### 系统概览
这是一个用 Rust 构建的**群聊平台**，具有实时消息功能，使用：
- **框架**：Actix-web（支持 WebSocket 的异步 HTTP 服务器）
- **数据库**：SQLite + SQLx（编译时 SQL 验证）
- **实时通信**：Tokio 异步运行时 + WebSocket 连接

### 核心实体与关系

1. **用户（User）** (`user_id` = UUID)
   - 通过 18 位身份证号识别 - 必须唯一
   - 可以拥有多个机器人和群组
   - 真实身份绑定用于身份验证

2. **机器人（Bot）** (`bot_id` = UUID，前缀为 `b_`）
   - 由用户拥有的 API 代理
   - 拥有唯一的 `token` 和 `secret`（用于 HMAC-SHA256 签名）
   - 状态：`active` 或 `inactive`
   - 只有机器人可以通过 API 发送消息

3. **群组（Group）** (`group_id` = UUID，前缀为 `g_`）
   - 由用户创建以组织对话
   - 包含多个机器人作为成员
   - 消息是群组范围内的

4. **消息（Message）**
   - 从机器人发送到群组
   - 类型：`text` 或 `file`
   - 存储发送者 ID、群组 ID、内容、时间戳

### 请求流程
```
HTTP/WebSocket 客户端
         ↓
    中间件（令牌验证）
         ↓
    处理层（认证、机器人、群组、消息、文件、ws）
         ↓
    服务层（业务逻辑）
         ↓
    数据库层（SQLx 编译时验证）
         ↓
    SQLite 数据库
```

## 📂 项目结构

```
src/
├── main.rs                    # 应用入口点，HTTP 服务器设置
├── lib.rs                     # 库根，模块导出
├── config.rs                  # 从 .env 加载配置
├── error.rs                   # 错误类型和处理
│
├── db/                        # 数据库层
│   ├── mod.rs                 # 连接池初始化和迁移运行器
│   ├── pool.rs                # 连接池设置
│   ├── schema.rs              # SQL 模式定义
│   └── migrations.rs          # 数据库迁移逻辑
│
├── models/                    # 数据结构
│   ├── user.rs                # 用户模型和操作
│   ├── bot.rs                 # 机器人模型和操作
│   ├── group.rs               # 群组模型和操作
│   ├── message.rs             # 消息模型和操作
│   ├── file.rs                # 文件模型和操作
│   └── mod.rs                 # 模块导出
│
├── handlers/                  # HTTP 请求处理器（控制层）
│   ├── auth.rs                # 注册/登录端点
│   ├── user.rs                # 用户管理端点
│   ├── bot.rs                 # 机器人 CRUD 端点
│   ├── group.rs               # 群组 CRUD 端点
│   ├── message.rs             # 消息发送端点
│   ├── file.rs                # 文件上传/下载端点
│   ├── ws.rs                  # WebSocket 升级处理器
│   └── mod.rs                 # 路由设置和处理器导出
│
├── services/                  # 业务逻辑
│   ├── bot.rs                 # 机器人服务操作
│   ├── message.rs             # 消息处理
│   ├── file.rs                # 文件处理
│   └── mod.rs                 # 服务导出
│
├── middlewares/               # HTTP 中间件
│   ├── auth.rs                # 令牌验证中间件
│   └── mod.rs                 # 中间件设置
│
├── utils/                     # 帮助函数
│   ├── id.rs                  # 带前缀的 UUID 生成
│   ├── token.rs               # 令牌创建和 HMAC 验证
│   └── mod.rs                 # 工具导出
│
└── ws/                        # WebSocket 管理
    ├── manager.rs             # WebSocket 连接管理器
    └── mod.rs                 # WS 导出
```

## 🔑 关键技术模式

### 令牌系统
- 格式：`{bot_id}:{timestamp}:{signature}`
- 签名 = HMAC-SHA256(bot_secret, `{bot_id}:{timestamp}`)
- 用于 API 调用中的机器人身份验证

### ID 生成
- 用户：`u_` + UUID（例如 `u_550e8400-e29b-41d4-a716-446655440000`）
- 机器人：`b_` + UUID
- 群组：`g_` + UUID
- ID 前缀可识别类型

### 数据库访问
- 所有查询都使用 SQLx 编译时验证（SELECT/INSERT/UPDATE/DELETE 在编译时检查）
- 支持可配置最小/最大连接数的连接池
- 通过 `db::run_migrations()` 在启动时自动运行迁移

### 异步架构
- 基于 Tokio 运行时的非阻塞 I/O
- 所有数据库操作和 HTTP 处理器都是 `async` 的
- WebSocket 连接由 `WsManager` 管理

## ⚙️ 配置

`.env` 文件控制：
- 服务器主机/端口和工作线程
- 数据库 URL 和连接池设置
- JWT 密钥和令牌过期时间
- 文件上传大小限制
- 速率限制（每秒请求数）
- 通过 `RUST_LOG` 设置 Rust 日志级别

## 🔒 安全说明

1. **身份证号存储**：包含敏感的 18 位身份证号
   - 生产环境：加密存储、使用 HTTPS 传输、审计访问

2. **机器人令牌密钥**：每个机器人都有唯一的密钥用于 HMAC 签名
   - 保持密钥安全；删除机器人会使令牌失效

3. **速率限制**：当前配置为每秒 10 个请求
   - 未来可改为每个机器人单独限制

## 🔄 常见开发任务

### 添加新的 API 端点
1. 在 `models/` 中定义请求/响应模型（或如果简单可在处理器文件中）
2. 在适当的 `handlers/*.rs` 中添加处理器函数
3. 在 `main.rs` 的 `App::new()` 中添加路由
4. 如果使用数据库：在 `services/` 中创建服务或在处理器中内联

### 添加数据库操作
1. 在 `models/` 中定义模型
2. 使用 SQLx 查询（编译时检查）
3. 如果创建新表：将 SQL 添加到 `db/schema.rs`
4. 迁移自动运行；重启应用以应用模式更改

### 使用 WebSocket
- 所有连接由 `src/ws/manager.rs` 中的 `WsManager` 跟踪
- 升级端点：`/ws/{group_id}`
- 管理器处理连接/断开连接/广播消息

## 📚 重要文件

- **Cargo.toml**：依赖和项目元数据
- **API_GUIDE.md**：完整的 API 端点文档
- **BOT_INTEGRATION_GUIDE.md**：机器人集成说明
- **UI_STARTUP_GUIDE.md**：前端设置和使用
- **PROJECT_COMPLETE.md**：项目完成状态和功能
- **scripts/test_chat.py**：演示所有功能的综合测试脚本

## 🧪 测试策略

运行 `python3 scripts/test_chat.py` 来：
- 创建两个测试用户（Alice, Bob）
- 创建多个机器人
- 创建测试群组
- 在群组中的机器人之间发送消息
- 验证数据库持久化

## 🎯 开发技巧

1. **类型安全**：利用 Rust 的类型系统；为 ID 使用强类型（不仅仅是字符串）
2. **编译时验证**：SQLx 在编译时捕获 SQL 错误 — 信任它
3. **异步优先**：所有 I/O 都应该是异步的；只在测试中阻塞
4. **错误处理**：使用 Result 类型；在 `error.rs` 中定义自定义错误变体
5. **日志记录**：使用 `tracing::info!()`、`debug!()`、`error!()` 进行可观察性
