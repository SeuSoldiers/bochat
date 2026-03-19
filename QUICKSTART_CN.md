# 快速开始指南

## 前置要求

- Rust 1.70+ ([安装 Rust](https://rustup.rs/))
- Git
- 文本编辑器或 IDE

## 安装与设置

### 1. 克隆仓库
```bash
cd /home/harkerhand/codes/rust-bochat
```

### 2. 设置环境
```bash
# 复制环境文件示例
cp .env.example .env

# 可选：用你的设置更新 .env
# 默认值适用于本地开发
```

### 3. 构建项目
```bash
# 检查编译（快速）
cargo check

# 构建调试二进制文件
cargo build

# 构建发布二进制文件（优化）
cargo build --release
```

### 4. 运行测试
```bash
# 运行所有测试
cargo test

# 运行特定测试文件
cargo test --test token_tests

# 运行并显示输出
cargo test -- --nocapture
```

### 5. 启动服务器
```bash
# 在开发模式运行
cargo run

# 在发布模式运行
cargo run --release

# 自定义日志
RUST_LOG=debug cargo run
```

服务器将在 `http://127.0.0.1:8080` 启动

## 快速测试

### 测试用户注册
```bash
curl -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "email": "alice@example.com",
    "password": "password123"
  }'
```

预期响应：
```json
{
  "user_id": "u_...",
  "username": "alice",
  "email": "alice@example.com",
  "bot_id": "b_...",
  "token": "b_...:timestamp:signature",
  "created_at": "..."
}
```

### 测试用户登录
```bash
curl -X POST http://127.0.0.1:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "password": "password123"
  }'
```

### 测试消息发送
```bash
# 首先，注册两个用户并获取他们的 token
# 然后从用户1向用户2发送消息

TOKEN="b_...:...:..."  # 来自注册的 Token

curl -X POST http://127.0.0.1:8080/api/v1/message/send \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "to_id": "b_...",
    "content": {"text": "你好，世界！"},
    "msg_type": "text"
  }'
```

### 测试健康检查
```bash
curl http://127.0.0.1:8080/health
# 预期响应：OK
```

## 目录结构

```
chat-platform-rs/
├── Cargo.toml              # 项目清单
├── Cargo.lock              # 锁定的依赖
├── .env.example            # 环境变量示例
├── .gitignore              # Git 忽略规则
│
├── src/                    # 源代码
│   ├── main.rs             # 服务器入口
│   ├── lib.rs              # 库根
│   ├── config.rs           # 配置
│   ├── error.rs            # 错误类型
│   │
│   ├── db/                 # 数据库层
│   │   ├── schema.rs       # 表创建
│   │   ├── pool.rs         # 连接池
│   │   └── migrations.rs   # 迁移助手
│   │
│   ├── models/             # 数据模型
│   │   ├── user.rs         # 用户模型
│   │   ├── bot.rs          # Bot 模型
│   │   ├── message.rs      # 消息模型
│   │   └── file.rs         # 文件模型
│   │
│   ├── handlers/           # HTTP 处理器
│   │   ├── auth.rs         # 登录/注册
│   │   ├── message.rs      # 消息发送
│   │   ├── file.rs         # 文件上传/下载
│   │   └── ws.rs           # WebSocket
│   │
│   ├── services/           # 业务逻辑
│   │   ├── bot.rs
│   │   ├── message.rs
│   │   └── file.rs
│   │
│   ├── utils/              # 工具函数
│   │   ├── token.rs        # Token 生成/验证
│   │   └── id.rs           # ID 生成
│   │
│   ├── ws/                 # WebSocket
│   │   ├── manager.rs
│   │   └── mod.rs
│   │
│   └── middlewares/        # HTTP 中间件
│       ├── auth.rs
│       └── mod.rs
│
├── tests/                  # 集成测试
│   └── token_tests.rs      # Token 测试
│
├── README.md               # 项目文档
├── QUICKSTART.md           # 快速开始指南
├── IMPLEMENTATION_SUMMARY.md  # 实现总结
└── DEVELOPMENT_GUIDE.md    # 开发指南
```

## 开发技巧

### 代码格式化
```bash
# 使用 rustfmt 格式化代码
cargo fmt

# 检查格式而不更改
cargo fmt -- --check
```

### 代码检查
```bash
# 运行 clippy 获取建议
cargo clippy
```

### 构建时间

- 首次构建：~30-40 秒（包括依赖编译）
- 增量构建：~1-5 秒
- 发布构建：~60+ 秒（优化）

在开发期间使用 `cargo check`（比完整构建快）。

## 数据库

### 自动模式创建
数据库模式在首次运行时自动创建：
- `users` 表用于用户账户
- `bots` 表用于 Bot 身份
- `messages` 表用于消息存储
- `files` 表用于文件元数据

### 检查数据库
```bash
# 如果没有安装 SQLite，请安装
# macOS：brew install sqlite3
# Ubuntu：sudo apt-get install sqlite3

# 打开数据库
sqlite3 chat_platform.db

# SQLite 命令
.tables

# 查看用户
SELECT * FROM users;

# 查看 bots
SELECT * FROM bots;

# 查看消息
SELECT * FROM messages;
```

## 常见问题

### 构建失败
- 确保安装了 Rust：`rustc --version`
- 更新 Rust：`rustup update`
- 清除缓存：`cargo clean && cargo build`

### 服务器无法启动
- 检查端口 8080 是否被占用：`lsof -i :8080`
- 在 `.env` 中更改端口：`SERVER_PORT=8081`
- 检查数据库权限

### 数据库问题
- 删除旧数据库：`rm chat_platform.db`
- 服务器启动时会重新创建
- 检查目录文件权限

## 配置文件

### `.env`（本地开发）
```env
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DATABASE_URL=sqlite:chat_platform.db
JWT_SECRET=your-secret-key
RUST_LOG=info
```

### `.env.example`（模板）
显示所有可用的配置选项

## 下一步

1. **阅读文档**
   - `README.md` - 项目概览
   - `IMPLEMENTATION_SUMMARY.md` - 实现了什么
   - `DEVELOPMENT_GUIDE.md` - 下一阶段

2. **探索代码**
   - 从 `src/main.rs` 开始
   - 查看 `src/handlers/auth.rs` 了解处理器示例
   - 检查 `src/models/` 了解数据结构

3. **进行更改**
   - 创建新分支：`git checkout -b feature/my-feature`
   - 进行更改并测试：`cargo test`
   - 工作时提交：`git add . && git commit -m "message"`

4. **下一阶段**
   - WebSocket 实现（见 DEVELOPMENT_GUIDE.md）
   - 文件上传/下载
   - 速率限制与 Redis

## 有用的命令

```bash
# 检查代码而不构建
cargo check

# 构建调试版本
cargo build

# 构建优化版本
cargo build --release

# 运行测试
cargo test

# 运行带日志记录
RUST_LOG=debug cargo run

# 格式化代码
cargo fmt

# 检查代码
cargo clippy

# 生成文档
cargo doc --open

# 清除构建缓存
cargo clean

# 更新依赖
cargo update

# 检查安全漏洞
cargo audit
```

## 文档链接

- [Actix-web 文档](https://docs.rs/actix-web/)
- [SQLx 文档](https://docs.rs/sqlx/)
- [Tokio 文档](https://docs.rs/tokio/)
- [Serde 文档](https://docs.rs/serde/)
- [Tracing 文档](https://docs.rs/tracing/)

## 支持与问题

如果遇到问题：
1. 查看 README.md 了解详细信息
2. 查看 IMPLEMENTATION_SUMMARY.md 了解当前状态
3. 查看 DEVELOPMENT_GUIDE.md 了解实现细节
4. 查看测试文件了解使用示例

---

**祝你编码愉快！** 🦀
