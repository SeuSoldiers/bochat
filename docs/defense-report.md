## Q1：你们是否为项目前端设计了纸质原型/线框图/高保真模型/可操作原型？（10分）

由于本项目的直接客户端是 Bot 程序（而非人类用户直接操作的 GUI），我们没有设计传统的纸质原型或高保真 UI 模型。但我们在以下层面做了等价的原型设计：

1. **API 协议原型（等价于线框图）**：
   - 所有请求/响应的 JSON Schema 通过 Rust 类型系统定义（Serde 序列化，类型安全）
   - 消息类型、错误码、审计日志格式全部在 `models/` 中预先建模

2. **SDK 原型**：项目在 `crates/bochat_sdk/` 中提供了 Bot SDK，包含 `basic_flow`、`ws_session`、`ws_observe` 三个示例程序，用于演示完整的 bot 交互流程

3. **可操作原型**：项目本身即可运行——执行 `cargo run` 启动服务，通过 SDK 示例即可体验完整流程（注册→创建 bot→建群→发消息→WebSocket 接收）

**结论**：本项目作为后端服务型产品，用户交互原型体现为 API 协议设计和 SDK 样例程序，而非传统的 UI 线框图。

---

## Q2：你们前端的 MVC 组件（如果有的话）是什么？（10分）

本项目的"前端"不是传统的浏览器 GUI，而是 Bot 客户端程序。系统架构本质上也是一个 MVC 变体：

```
┌──────────────────────────────────────────────────────┐
│  Model（模型层）                                      │
│  src/models/         数据模型定义（User, Bot, Group,   │
│                       Message, File, Notification）    │
│  src/repositories/   数据访问层（CRUD 操作封装）        │
├──────────────────────────────────────────────────────┤
│  View（视图层）                                       │
│  JSON 响应序列化     通过 Serde 将模型转为 JSON         │
│  WebSocket 事件推送  将 MessageResponse 推送给客户端    │
│  SDK 示例程序        basic_flow / ws_session 等样例    │
├──────────────────────────────────────────────────────┤
│  Controller（控制器层）                                │
│  src/handlers/       路由处理函数（参数解析、校验、调用） │
│  src/middlewares/    认证中间件（UserAuth / BotAuth）   │
│  src/services/       业务逻辑（authz, audit, file_scan）│
└──────────────────────────────────────────────────────┘
```

**数据流**：

```
HTTP Request → Middleware(认证) → Handler(参数解析)
  → Service(业务逻辑) → Repository(数据访问)
  → JSON Response / WebSocket Event
```

以消息发送为例：
- **Controller**：`handlers/message.rs → send_message()` 接收 POST 请求，解析 `CreateMessageRequest`
- **Model**：`MessageWithSenderRow` 数据模型，`MessageRecordManager` 缓存管理
- **View**：`MessageResponse` 序列化为 JSON 返回，同时通过 WebSocket 广播 `WsEvent`

---

## Q3：你们评估测试项目用户体验的计划是什么？（10分）

### 3.1 自动化测试层

| 测试层级 | 覆盖范围 | 状态 |
|----------|----------|------|
| 单元测试 | Token 生成/验证（`utils/token.rs`） | 已实现 2 个用例 |
| 集成测试 | 完整聊天流程（`tests/chat_flow_integration.rs`） | 已编写，需 PG/Redis 环境 |
| 集成测试 | 文件引用生命周期（`tests/file_reference_integration.rs`） | 已编写，需 PG/Redis 环境 |

### 3.2 用户体验验证计划

1. **API 契约测试**：
   - 编写 JSON Schema 验证所有响应格式
   - 验证错误码和中文错误消息的一致性

2. **SDK 集成测试**：
   - 通过 `bochat_sdk` 示例程序完成端到端流程测试
   - 覆盖场景：注册→登录→创建 Bot→建群→加入→发消息→收消息→文件上传/下载→退群

3. **性能基准测试**：
   - 消息发送吞吐量压力测试（目标 > 1000 msg/s）
   - WebSocket 并发连接数测试（目标 > 1000 连接）
   - 缓存命中率监控（目标 > 95%）

4. **冒烟测试清单**：
   - 注册/登录/Token 过期处理
   - 公开群直接加入 vs 私密群审批流程
   - 消息幂等性验证（相同 idempotency_key 重复发送）
   - 文件上传去重验证（相同内容秒传）
   - 网络断开重连后 WebSocket 行为

5. **异常场景测试**：
   - PostgreSQL 不可用时的降级行为
   - Redis 不可用时消息缓存回退
   - 超大文件上传拒绝验证
   - Token 过期后的错误处理

---

## Q4：你们项目的组件图是怎样的？（10分）

系统组件图（UML Component Diagram）：

```
┌─────────────────────────────────────────────────────────────┐
│                     BoChat Platform                          │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐ │
│  │ Auth     │  │ Group    │  │ Message  │  │ File       │ │
│  │ Handler  │  │ Handler  │  │ Handler  │  │ Handler    │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬───────┘ │
│       │             │             │             │          │
│  ┌────┴─────────────┴─────────────┴─────────────┴───────┐  │
│  │              Middleware Layer                         │  │
│  │  ┌──────────────────┐  ┌──────────────────────┐     │  │
│  │  │ require_user_auth│  │ require_bot_auth     │     │  │
│  │  └────────┬─────────┘  └──────────┬───────────┘     │  │
│  └───────────┼───────────────────────┼─────────────────┘  │
│              │                       │                     │
│  ┌───────────┴───────────────────────┴─────────────────┐  │
│  │              Service Layer                           │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────────────┐ │  │
│  │  │ Authz    │ │ Audit    │ │ FileScan (async)     │ │  │
│  │  │ Service  │ │ Service  │ │ Service              │ │  │
│  │  └──────────┘ └──────────┘ └──────────────────────┘ │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────────────┐ │  │
│  │  │ Bot      │ │ Group    │ │ Notification         │ │  │
│  │  │ Service  │ │ Service  │ │ Service              │ │  │
│  │  └──────────┘ └──────────┘ └──────────────────────┘ │  │
│  │  ┌──────────────────────────────────────────────┐   │  │
│  │  │ FileManager (文件引用生命周期管理)             │   │  │
│  │  └──────────────────────────────────────────────┘   │  │
│  └───────────────────────┬─────────────────────────────┘  │
│                          │                                 │
│  ┌───────────────────────┴─────────────────────────────┐  │
│  │    MessageRecordManager (消息缓存管理器)              │  │
│  │    ┌──────────────────────────────┐                 │  │
│  │    │ RedisMessageCache            │                 │  │
│  │    │ msg:{id} / grp:{id} / idem:* │                 │  │
│  │    └──────────────────────────────┘                 │  │
│  └───────────────────────┬─────────────────────────────┘  │
│                          │                                 │
│  ┌───────────────────────┴─────────────────────────────┐  │
│  │              Repository Layer                        │  │
│  │  UserRepo | BotRepo | GroupRepo | MessageRepo       │  │
│  │  FileRepo | AuditRepo | NotifRepo | AuthzRepo       │  │
│  └───────────────────────┬─────────────────────────────┘  │
│                          │                                 │
└──────────────────────────┼─────────────────────────────────┘
                           │
              ┌────────────┴────────────┐
              │                         │
       ┌──────┴──────┐          ┌──────┴──────┐
       │ PostgreSQL   │          │   Redis     │
       │ (持久存储)   │          │ (消息缓存)  │
       └─────────────┘          └─────────────┘
              │                         │
              │          ┌──────────────┴──────┐
              │          │  File System         │
              │          │  (assets/files/)     │
              │          └─────────────────────┘
              │
       ┌──────┴──────────────────────┐
       │  WebSocket Manager          │
       │  Arc<RwLock<HashMap>>       │
       │  mpsc::UnboundedSender      │
       └─────────────────────────────┘
```

---

## Q5：你们项目的部署图是怎样的？（10分）

### 部署架构

```
                          ┌──────────────┐
                          │   Nginx /    │
                          │   Caddy      │
                          │ (反向代理)   │
                          └──────┬───────┘
                                 │ TLS 终止
                                 │ 静态文件服务
                    ┌────────────┼────────────┐
                    │            │            │
              ┌─────┴─────┐     │     ┌──────┴──────┐
              │  HTTP API │     │     │  WebSocket  │
              │  :8080    │     │     │  /ws        │
              └─────┬─────┘     │     └──────┬──────┘
                    │           │            │
         ┌──────────┴───────────┴────────────┴──────────┐
         │              BoChat Platform                  │
         │          (Rust Binary, 4 workers)             │
         │                                               │
         │  ┌──────────┐  ┌──────────┐  ┌────────────┐ │
         │  │ AppState │  │ WsMgr    │  │ MsgRecMgr  │ │
         │  │ Config   │  │ Channels │  │ RedisCache │ │
         │  │ PgPool   │  │          │  │ AtomicI64  │ │
         │  └──────────┘  └──────────┘  └────────────┘ │
         └──────────┬─────────────────────┬─────────────┘
                    │                     │
         ┌──────────┴──────┐   ┌─────────┴──────────┐
         │   PostgreSQL    │   │      Redis          │
         │   :5432         │   │      :6379          │
         │                 │   │                     │
         │ ┌─────────────┐ │   │ msg:{id}  (String)  │
         │ │ 12 tables   │ │   │ grp:{gid} (ZSET)    │
         │ │ 30+ indexes │ │   │ idem:*    (String)  │
         │ └─────────────┘ │   └─────────────────────┘
         └─────────────────┘
                    │
         ┌──────────┴──────────────┐
         │   File Storage          │
         │   ./assets/files/       │
         │   {file_id}/{filename}  │
         └─────────────────────────┘
```

### 最小部署方案（开发/测试）

单机部署所有组件：

| 组件 | 端口 | 资源需求 |
|------|------|----------|
| BoChat Platform | 8080 | 2 CPU, 512MB RAM |
| PostgreSQL 16+ | 5432 | 1 CPU, 256MB RAM |
| Redis 7+ | 6379 | 0.5 CPU, 128MB RAM |
| 文件存储 | 本地磁盘 | 按需（100MB/文件上限） |


## Q6：你们项目的包图是怎样的？（10分）

### Rust Crate 包图

```
┌─────────────────────────────────────────────────────┐
│                   Workspace                          │
│                  bochat (root)                        │
│                  Cargo.toml [workspace]               │
│                  resolver = "2"                       │
├──────────────────────────┬──────────────────────────┤
│                          │                           │
│  ┌────────────────────┐  │  ┌─────────────────────┐ │
│  │  chat_platform     │  │  │  bochat_sdk          │ │
│  │  (bin crate)       │  │  │  (lib crate)         │ │
│  │                    │  │  │  edition = 2024      │ │
│  │  ┌──────────────┐  │  │  │                      │ │
│  │  │ lib.rs       │◄─┼──┼──┤  uses chat_platform  │ │
│  │  │ (核心逻辑)   │  │  │  │  API (HTTP/WS)       │ │
│  │  └──────────────┘  │  │  │                      │ │
│  │  ┌──────────────┐  │  │  │  features:           │ │
│  │  │ main.rs      │  │  │  │  - http (default)    │ │
│  │  │ (启动入口)   │  │  │  │  - ws                │ │
│  │  └──────────────┘  │  │  └─────────────────────┘ │
│  └────────────────────┘  └──────────────────────────┘
│                                                       │
└───────────────────────────────────────────────────────┘
```

### chat_platform 内部模块依赖图

```
                         ┌─────────────┐
                         │   main.rs   │
                         │   lib.rs    │ (路由组装, AppState)
                         └──────┬──────┘
                                │
          ┌─────────────────────┼─────────────────────┐
          │                     │                     │
    ┌─────┴─────┐        ┌──────┴──────┐       ┌──────┴──────┐
    │ handlers/ │        │  services/  │       │   cache/    │
    │ (8 files) │───────>│  (9 files)  │──────>│ (2 files)   │
    │           │        │             │       │ RedisCache  │
    │ auth      │        │ authz       │       └─────────────┘
    │ bot       │        │ audit       │
    │ group     │        │ bootstrap   │       ┌─────────────┐
    │ message   │        │ bot         │       │ repositories│
    │ file      │        │ file        │──────>│ (12 files)  │
    │ user      │        │ file_manager│       │             │
    │ audit     │        │ file_scan   │       │ user/bot/   │
    │ notif     │        │ group       │       │ group/msg/  │
    │ ws        │        │ message     │       │ file/audit/ │
    └─────┬─────┘        │ notification│       │ notif/      │
          │              │ msg_rec_mgr │       │ authz/      │
          │              └─────────────┘       │ bootstrap   │
          │                                    └──────┬──────┘
          │                                           │
    ┌─────┴─────┐                              ┌──────┴──────┐
    │middlewares│                              │    db/      │
    │ mod.rs    │                              │ (3 files)   │
    │ UserAuth  │                              │ pool/schema │
    │ BotAuth   │                              └─────────────┘
    └───────────┘
          │
    ┌─────┴─────┐     ┌──────────┐     ┌─────────────┐
    │   http.rs │     │ config.rs│     │  error.rs   │
    │ (token    │     │ (环境变量 │     │ (AppError   │
    │  解析)    │     │  配置)    │     │  统一错误)  │
    └───────────┘     └──────────┘     └─────────────┘
                              │
    ┌──────────┐     ┌────────┴──────┐     ┌──────────────┐
    │  utils/  │     │   models/     │     │    ws/        │
    │ id/token │     │ (10 files)    │     │ manager.rs    │
    └──────────┘     └───────────────┘     └──────────────┘
```

### 外部依赖包

| 类别 | Crate | 版本 | 用途 |
|------|-------|------|------|
| Web 框架 | axum | 0.8 | HTTP 路由 + WebSocket + Multipart |
| 中间件 | tower-http | 0.6 | CORS |
| 异步运行时 | tokio | 1.52 | 全异步调度 |
| 数据库 | sqlx | 0.8 | PostgreSQL 异步驱动 |
| 缓存 | redis | 0.31 | Redis 异步客户端 |
| 序列化 | serde / serde_json | 1 | JSON 序列化 |
| 密码学 | hmac / sha2 / hex | - | Token HMAC 签名 |
| 时间 | chrono | 0.4 | RFC 3339 时间戳 |
| 日志 | tracing / tracing-subscriber | 0.3 | 结构化日志 + 文件落盘 |
| ID 生成 | uuid | 1.23 | v4 UUID |
| URL 编码 | percent-encoding | 2.3 | 文件名 URL 编码 |
| 配置 | config | 0.15 | 多源配置加载 |
| 错误处理 | thiserror | 2 | 派生 Error trait |

---

## Q7：是否有使用虚拟化技术的计划？（10分）

### 已规划

**1. Docker 容器化**

当前项目尚未包含 `Dockerfile`，规划添加如下：

```dockerfile
# 多阶段构建
FROM rust:1.95-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/chat_platform /usr/local/bin/
ENV SERVER_HOST=0.0.0.0
EXPOSE 8080
CMD ["chat_platform"]
```

**2. Docker Compose 本地开发环境**（规划添加 `docker-compose.yml`）：

```yaml
services:
  postgres:
    image: postgres:17-alpine
    environment:
      POSTGRES_USER: chat_user
      POSTGRES_PASSWORD: chat_pass
      POSTGRES_DB: chat_platform
    ports: ["5432:5432"]
    volumes: ["pgdata:/var/lib/postgresql/data"]

  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]

  app:
    build: .
    ports: ["8080:8080"]
    depends_on: [postgres, redis]
    environment:
      DATABASE_URL: postgres://chat_user:chat_pass@postgres:5432/chat_platform
      REDIS_URL: redis://redis:6379
      JWT_SECRET: ${JWT_SECRET}
    volumes: ["./assets:/app/assets"]

volumes:
  pgdata:
```

**3. CI/CD 集成**：
- GitHub Actions / GitLab CI 自动构建 Docker 镜像
- 推送至容器镜像仓库（Docker Hub / GHCR）

---

## Q8：项目中是否存在潜在的安全风险？你们将如何降低这些风险？（10分）

### 8.1 已识别的安全风险

| 风险 | 严重程度 | 描述 |
|------|----------|------|
| **JWT 密钥硬编码** | 高 | `config.rs` 中 `jwt_secret` 默认值为 `"your-secret-key"` |
| **超级管理员默认密码** | 高 | `bootstrap.rs` 中默认密码为 `"Admin123456"` |
| **CORS 全开放** | 中 | `CorsLayer::permissive()` 允许任意来源 |
| **无请求速率限制** | 中 | `RATE_LIMIT_PER_SECOND` 配置存在但未实现中间件 |
| **Token 无撤销机制** | 中 | Token 签发后无法主动失效 |
| **WebSocket 无连接数限制** | 中 | Bot 可无限建立 WebSocket 连接 |
| **文件内容扫描非异步** | 低 | `tokio::fs::read` 读取整个文件到内存进行扫描 |
| **审计日志为 Best-Effort** | 低 | 审计日志写入失败不阻塞业务流程 |

### 8.2 风险缓解方案

**1. JWT 密钥 / 超级管理员密码**（已部分实现）

```rust
// bootstrap.rs 启动时日志提醒
if !seed_result.password_source_env {
    tracing::warn!(
        "⚠️ 未设置 SUPER_ADMIN_PASSWORD，当前使用默认初始密码，..."
    );
}
```

**改进方案**：
- 启动时检测默认值，若未通过环境变量覆盖则**拒绝启动**（生产模式）
- 支持通过文件挂载方式注入密钥（Kubernetes Secret / Docker Secret）

**2. CORS 限制**

```rust
// 当前：任意来源
CorsLayer::permissive()

// 改进：通过环境变量配置允许的来源
let allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
    .unwrap_or_else(|_| "*".to_string());
CorsLayer::new()
    .allow_origin(allowed_origins.parse::<AllowOrigin>()?)
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);
```

**3. 请求速率限制**（待实现）

```rust
use tower::limit::RateLimitLayer;
// 基于 IP 或 bot_id 的速率限制中间件
let rate_limit = RateLimitLayer::new(
    config.security.rate_limit_per_second as u64,
    Duration::from_secs(1),
);
```

**4. Token 撤销机制**（待实现）

方案：在 `bots` 表增加 `secret_version` 字段，更新 secret 即可使旧 token 失效。

**5. WebSocket 连接数限制**

```rust
// ws_handler 中添加每个 bot_id 的最大连接数检查
let current_count = state.ws_manager.get_online_count(&bot_id).await;
if current_count >= MAX_CONNECTIONS_PER_BOT {
    return Err(AppError::RateLimitExceeded);
}
```

**6. 文件扫描优化**

当前实现的扫描引擎 `bochat-simple-scan-v1` 基于规则（扩展名黑名单、魔数检测、关键词匹配），不依赖外部杀毒引擎。可扩展对接 ClamAV 等专业扫描服务。

**7. 其他安全措施（已实现）**

- 所有 SQL 查询使用参数化绑定（`$1, $2...`），防 SQL 注入
- 文件名严格过滤（剔除路径分隔符和控制字符），防目录遍历
- HMAC-SHA256 签名保证 Token 不可伪造
- SHA256 内容哈希实现文件去重，防止存储重复恶意文件

---

## Q9：未来你们产品的峰值需求容量将是多少？（10分）

### 预估模型

假设一个中等规模的 Bot 群聊平台：

| 指标 | 日均值 | 峰值 (QPS) | 说明 |
|------|--------|------------|------|
| 注册用户 | 10,000 | 100/min | 用户注册 |
| 活跃群聊 | 100 | - | 同时存在群聊数 |
| 群内 Bot | 30/群 | - | 平均每群 30 个 Bot |
| 消息发送 | 100,000/天 | 500/min (8.3/s) | 消息吞吐 |
| 消息查询 | 500,000/天 | 2000/min (33/s) | 消息历史查询 |
| WebSocket 连接 | 300 同时在线 | - | Bot 长连接 |
| 文件上传 | 1,000/天 | 20/min | 文件上传 |
| 存储容量 | ~10GB/月 | - | 消息 + 文件 |

### 容量计算

**数据库**：
- 消息表：10 万条/天 × 365 天 = 3650 万条/年
- 每条消息约 2KB，年增长约 73GB
- 审计日志：2 万条/天 × 365 = 730 万条/年

**Redis 缓存**：
- 活跃群聊 1000 个，每群平均 500 条热消息 = 50 万条缓存
- 每条消息 JSON 约 1KB，缓存约 500MB

**文件存储**：
- 1000 次上传/天 × 平均 5MB = 5GB/天
- 考虑 SHA256 去重率 30%，实际增长约 3.5GB/天

### 峰值容量设计目标

| 资源 | 目标容量 |
|------|----------|
| API QPS | 支持 500 QPS（单实例） |
| WebSocket 并发 | 支持 2000 连接 |
| 消息写入延迟 | P99 < 50ms（Redis 写入） |
| 数据库连接池 | 20 连接（单实例） |
| Redis 内存 | 2GB（含缓存 + 缓冲） |
| 磁盘 IOPS | 1000（文件上传/下载） |

---

## Q10：基于潜在的需求/容量，项目中是否存在性能瓶颈？你们将如何改进？（10分）

### 10.1 已识别的瓶颈

#### 瓶颈 1：单实例 msg_id 生成（关键瓶颈）

```rust
// 当前实现
static NEXT_MSG_ID: AtomicI64 = AtomicI64::new(0);
pub fn next_id(&self) -> i64 {
    NEXT_MSG_ID.fetch_add(1, Ordering::SeqCst)
}
```

**问题**：`AtomicI64` 只在单进程内唯一。多实例部署会导致 msg_id 冲突。

**改进方案**：
- **方案 A（推荐）**：使用 PostgreSQL 序列 `BIGINT GENERATED ALWAYS AS IDENTITY`，INSERT 返回 msg_id，改为"先落库后写缓存"
- **方案 B**：引入雪花算法（Snowflake ID），用 instance_id 占位 + 时间戳 + 序号
- **方案 C**：用 Redis `INCR msg_id_seq` 生成全局唯一 ID

#### 瓶颈 2：群消息预热全量加载

```rust
// 首次查询某群消息时，全量加载到 Redis
async fn preload_group(&self, group_id: &str) -> AppResult<()> {
    let msgs = MessageRepository::list_all_enriched_by_group(&self.pool, group_id).await?;
    self.cache.preload_group(&msgs).await
}
```

**问题**：超大群（10 万+ 条消息）首次加载耗时长、内存占用大。

**改进方案**：
- 首次加载只预取最近 N 条（如 2000 条）
- 后续访问历史消息时增量回填
- 设置 ZSET 最大长度，超过时 `ZREMRANGEBYRANK` 清理冷数据

#### 瓶颈 3：文件扫描全量读入内存

```rust
let bytes = tokio::fs::read(&job.storage_path).await?;
```

**问题**：100MB 文件读取会占用大量内存。

**改进方案**：
- 只读取文件头 N 字节（如 64KB）做魔数/关键词检测
- 流式读取文本内容检测，设置上限（当前已限制 64KB 采样）

#### 瓶颈 4：WebSocket 广播 O(N) 线性复杂度

```rust
pub async fn broadcast_message(&self, bot_id: &str, message: WsEvent) {
    let connections = self.connections.read().await;
    if let Some(conns) = connections.get(bot_id) {
        for tx in conns {
            let _ = tx.send(message.clone());  // 每条消息 clone N 次
        }
    }
}
```

**问题**：一个群有大量 Bot 成员时，消息广播需要 O(N) clone。

**改进方案**：
- 使用 `Arc<WsEvent>` 避免 clone，`tx.send(Arc::clone(&msg))`
- 考虑按 group_id 索引连接，替代按 bot_id 索引后逐条广播

#### 瓶颈 5：Redis 单连接多路复用

```rust
pub struct RedisMessageCache {
    conn: MultiplexedConnection,
}
```

**问题**：所有 Redis 操作通过一个 `MultiplexedConnection`，高并发时可能成为瓶颈。

**改进方案**：
- 使用 Redis 连接池（`r2d2-redis` 或 `bb8-redis`）
- 或升级到 Redis Cluster 客户端（`redis::cluster::ClusterClient`）

### 10.2 改进优先级矩阵

| 瓶颈 | 影响面 | 修复难度 | 优先级 |
|------|--------|----------|--------|
| msg_id 单实例限制 | 多实例部署 | 中 | P0 |
| 群消息预热全量加载 | 大群性能 | 低 | P1 |
| 文件扫描内存占用 | 大文件安全 | 低 | P1 |
| WebSocket 广播 clone | 大群实时性 | 低 | P2 |
| Redis 单连接 | 极高并发 | 中 | P2 |
| Token 无撤销机制 | 安全 | 低 | P2 |
| CORS 全开放 | 安全 | 低 | P2 |
| 无速率限制实现 | 安全/DDoS | 中 | P1 |
