```markdown
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
# 重置数据库：rm chat_platform.db && touch chat_platform.db
```

### 测试
```bash
# 运行全部单元测试 + 集成测试
cargo test

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

tests/                         # 集成测试
├── api_auth_test.rs           # 认证流程集成测试
├── api_bot_test.rs            # 机器人 CRUD 集成测试
├── api_group_test.rs          # 群组管理集成测试
├── api_message_test.rs        # 消息发送集成测试
└── api_ws_test.rs             # WebSocket 集成测试

scripts/
└── test_chat.py               # 端到端综合测试脚本
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

## 🧪 测试策略

### 开发工作流（必须遵守）

每次修改代码后，按以下顺序执行：

```
1. cargo check          # 确保编译通过
2. cargo clippy         # 代码质量检查，修复所有 warning
3. cargo test           # 运行全部单元测试 + 集成测试
4. cargo fmt            # 格式化代码
5. 更新相关测试          # 新增/修改功能必须同步新增/修改测试
6. 更新 UI              # 如果涉及 API 变更或新增端点，检查并更新前端调用
```

**核心原则：没有测试的代码不算完成。**

### 单元测试

- 每个 `models/` 和 `services/` 中的公共函数都应有对应的单元测试
- 测试文件放在被测模块同目录下，遵循 Rust 惯例写在 `#[cfg(test)]` 模块中
- 使用 `#[tokio::test]` 标注异步测试函数
- 数据库相关测试使用独立的测试数据库或事务回滚，避免污染开发数据
- 测试命名格式：`test_{函数名}_{场景}`，例如 `test_create_bot_with_duplicate_token_fails`

示例结构：
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_group_success() {
        // arrange: 准备测试数据
        // act: 调用被测函数
        // assert: 验证结果
    }

    #[tokio::test]
    async fn test_create_group_with_invalid_bot_id_returns_error() {
        // 测试错误路径
    }
}
```

### 集成测试

- 集成测试放在项目根目录的 `tests/` 文件夹下
- 覆盖完整的 HTTP 请求链路：请求 → 中间件 → 处理器 → 服务 → 数据库 → 响应
- 重点测试场景：
  - 认证流程（注册、登录、令牌过期）
  - 机器人 CRUD 操作
  - 群组创建与成员管理
  - 消息发送与 WebSocket 广播
  - 文件上传与下载
  - 错误处理（无效输入、权限不足、资源不存在）

示例结构：
```rust
// tests/api_bot_test.rs
use actix_web::{test, App};

#[actix_web::test]
async fn test_create_and_list_bots() {
    let app = test::init_service(
        App::new().configure(configure_routes)
    ).await;

    // 1. 先注册用户并获取 token
    // 2. 创建机器人
    // 3. 列出机器人，验证创建成功
    // 4. 删除机器人
    // 5. 再次列出，验证已删除
}
```

### Python 测试脚本

- `scripts/test_chat.py` 是端到端的综合测试脚本
- 新增 API 端点后，必须在该脚本中添加对应的测试用例
- 运行方式：`python3 scripts/test_chat.py`
- 测试脚本应覆盖：用户注册 → 机器人创建 → 群组创建 → 消息发送 → 验证数据库持久化

### 测试覆盖率目标

- `models/` 和 `services/` 层：目标 90%+ 覆盖率
- `handlers/` 层：通过集成测试覆盖主要路径
- 错误路径：每个可能的错误变体至少有一个测试用例

## 🔄 常见开发任务

### 添加新的 API 端点
1. 在 `models/` 中定义请求/响应模型（或如果简单可在处理器文件中）
2. 在适当的 `handlers/*.rs` 中添加处理器函数
3. 在 `main.rs` 的 `App::new()` 中添加路由
4. 如果使用数据库：在 `services/` 中创建服务或在处理器中内联
5. 编写对应的单元测试和集成测试
6. 在 `scripts/test_chat.py` 中添加端到端测试用例
7. 检查前端 UI 是否需要更新

### 添加数据库操作
1. 在 `models/` 中定义模型
2. 使用 SQLx 查询（编译时检查）
3. 如果创建新表：将 SQL 添加到 `db/schema.rs`
4. 迁移自动运行；重启应用以应用模式更改
5. 编写单元测试验证数据库操作的正确性

### 使用 WebSocket
- 所有连接由 `src/ws/manager.rs` 中的 `WsManager` 跟踪
- 升级端点：`/ws/{group_id}`
- 管理器处理连接/断开连接/广播消息
- 测试时使用多个模拟客户端验证广播行为

### 修改现有功能的检查清单

1. 修改 `models/` → 更新对应的单元测试
2. 修改 `services/` → 更新对应的单元测试 + 集成测试
3. 修改 `handlers/` → 更新集成测试 + 检查前端调用
4. 修改数据库 schema → 确认迁移逻辑正确，重启后数据完整
5. 修改 WebSocket 逻辑 → 手动测试多客户端连接场景
6. 修改中间件 → 测试认证通过和失败两种路径

### 新增功能的完整流程

```
1. 设计数据模型（models/）
   └── 编写模型的单元测试

2. 实现业务逻辑（services/）
   └── 编写服务层的单元测试

3. 添加 API 端点（handlers/ + main.rs 路由）
   └── 编写集成测试覆盖该端点

4. 更新 Python 测试脚本（scripts/test_chat.py）
   └── 添加新端点的端到端测试用例

5. 更新前端 UI
   └── 添加对应的页面/组件/调用逻辑
   └── 手动启动验证功能正常

6. 最终验证
   └── cargo test 全部通过
   └── python3 scripts/test_chat.py 全部通过
   └── cargo clippy 无 warning
   └── cargo fmt 无变更
```

### UI 更新检查清单

当后端发生以下变更时，必须检查并更新前端 UI：

- [ ] 新增 API 端点 → 前端添加对应的调用函数和 UI 入口
- [ ] 修改请求/响应结构 → 更新前端的类型定义和数据解析逻辑
- [ ] 修改认证流程 → 更新前端的登录/令牌管理逻辑
- [ ] 新增字段或枚举值 → 更新表单、列表展示、筛选条件
- [ ] 修改错误码或错误格式 → 更新前端的错误提示展示

**验证方式：**
```bash
# 启动完整环境进行手动验证
./start.sh    # Linux/macOS
./start.bat   # Windows

# 然后在浏览器中操作，确认：
# 1. 新功能可以正常使用
# 2. 已有功能未被破坏
# 3. 错误提示友好且准确
```

## 📚 重要文件

- **Cargo.toml**：依赖和项目元数据
- **API_GUIDE.md**：完整的 API 端点文档
- **BOT_INTEGRATION_GUIDE.md**：机器人集成说明
- **UI_STARTUP_GUIDE.md**：前端设置和使用
- **PROJECT_COMPLETE.md**：项目完成状态和功能
- **scripts/test_chat.py**：演示所有功能的综合测试脚本

## 🎯 开发技巧

1. **类型安全**：利用 Rust 的类型系统；为 ID 使用强类型（不仅仅是字符串）
2. **编译时验证**：SQLx 在编译时捕获 SQL 错误 — 信任它
3. **异步优先**：所有 I/O 都应该是异步的；只在测试中阻塞
4. **错误处理**：使用 Result 类型；在 `error.rs` 中定义自定义错误变体
5. **日志记录**：使用 `tracing::info!()`、`debug!()`、`error!()` 进行可观察性
6. **测试先行**：修改 bug 时，先写一个能复现 bug 的测试，再修复代码，确认测试由红变绿
7. **测试隔离**：每个测试应独立运行，不依赖其他测试的执行顺序或残留数据
8. **Mock 策略**：外部依赖（如文件系统、网络）应使用 trait 抽象，便于测试时注入 mock 实现
```