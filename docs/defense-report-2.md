## Q1：你的项目中使用了哪种 IDE？（10分）

本项目使用 **RustRover** 作为后端开发 IDE，并使用 **VS Code** 作为前端开发和文档编辑的 IDE。

社区部分，大多使用 **PyCharm** 进行 Python 开发，部分使用 **VS Code**。


## Q2：你的项目中是否使用了高级 IDE？（10分）

同上

## Q3：你是否在项目中使用了任何现有的包或组件？（10分）

是的，本项目复用了大量成熟的生态组件，分为三个子项目：

### 一、后端核心（Rust — chat_platform crate）

| 类别 | 包名 | 版本 | 用途 |
|------|------|------|------|
| **Web 框架** | axum | 0.8 | HTTP 服务、路由、WebSocket、文件上传 |
| **HTTP 中间件** | tower-http | 0.6 | CORS 跨域处理 |
| **异步运行时** | tokio | 1.52 | 全异步 I/O 调度、spawn 后台任务 |
| **数据库** | sqlx | 0.8 | PostgreSQL 异步驱动、编译期 SQL 检查、连接池 |
| **缓存** | redis | 0.31 | Redis 异步客户端、多路复用连接 |
| **序列化** | serde / serde_json | 1 | JSON 请求/响应序列化、Redis 数据序列化 |
| **HMAC 签名** | hmac / sha2 / hex | 0.12 / 0.10 / 0.4 | Token 生成与验证 |
| **时间处理** | chrono | 0.4 | RFC 3339 时间戳生成 |
| **UUID** | uuid | 1.23 | 分布式唯一 ID 生成（user/bot/group/file 等） |
| **URL 编码** | percent-encoding | 2.3 | 文件名 URL 编码（中文名兼容） |
| **配置管理** | config | 0.15 | 多源配置加载（环境变量、文件） |
| **错误处理** | thiserror | 2 | `#[derive(Error)]` 派生宏 |
| **日志** | tracing / tracing-subscriber | 0.3 | 结构化日志、按天切分、异步非阻塞写入 |
| **日志文件** | tracing-appender | 0.2 | 日志文件自动轮转 |

### 二、Rust SDK（bochat_sdk crate）

| 类别 | 包名 | 版本 | 用途 |
|------|------|------|------|
| HTTP 客户端 | reqwest | 0.13.2 | 调用后端 API（JSON + multipart 文件上传） |
| WebSocket | tokio-tungstenite | 0.29 | SDK 端 WebSocket 长连接 |
| 序列化 | serde / serde_json | 1 | 请求/响应序列化 |
| URL 编码 | percent-encoding | 2.3 | 文件名 URL 编码 |
| 错误处理 | thiserror | 2 | SDK 自定义错误类型 |
| URL 处理 | url | 2 | URL 解析与构建 |

### 三、Python SDK（python-sdk/）

| 类别 | 包名 | 版本范围 | 用途 |
|------|------|----------|------|
| **异步 HTTP** | httpx | >=0.27, <1.0 | 异步 API 调用（含 HTTP/2 支持） |
| **WebSocket** | websockets | >=12, <16 | 异步 WebSocket 长连接（可选，`ws` 扩展） |
| **构建工具** | hatchling | >=1.25 | PEP 621 兼容的 Python 包构建 |

Python SDK 使用标准库模块：`asyncio`（异步运行时）、`dataclasses`（数据模型）、`json`（序列化）、`urllib.parse`（URL 编码）、`pathlib`（文件路径）、`itertools`（分页迭代）。

### 四、前端 Web（ui-vue/）

#### 运行时依赖

| 类别 | 包名 | 版本 | 用途 |
|------|------|------|------|
| **前端框架** | vue | ^3.5.30 | 渐进式 UI 框架（Composition API） |
| **路由** | vue-router | ^5.0.4 | SPA 前端路由 |
| **状态管理** | pinia | ^3.0.4 | Vue 3 官方状态管理 |
| **UI 组件库** | naive-ui | ^2.44.1 | 企业级 Vue 3 组件库（60+ 组件） |
| **HTTP 客户端** | axios | ^1.13.6 | 浏览器 HTTP 请求 |
| **图标** | lucide-vue-next | ^1.0.0 | 开源 SVG 图标库 |

#### 开发依赖

| 类别 | 包名 | 版本 | 用途 |
|------|------|------|------|
| **构建工具** | vite | ^8.0.1 | 极速 HMR 开发服务器 + 打包 |
| **Vue 编译** | @vitejs/plugin-vue | ^6.0.5 | Vite Vue SFC 编译插件 |
| **类型检查** | typescript | ^5.9.3 | TypeScript 类型系统 |
| **Vue 类型检查** | vue-tsc | ^3.2.6 | Vue 模板类型检查 |
| **CSS 预处理器** | sass | ^1.98.0 | SCSS 样式编译 |
| **类型辅助** | @types/node | ^25.5.0 | Node.js 类型定义 |
| **TS 配置** | @vue/tsconfig | ^0.9.0 | Vue 项目 TS 配置预设 |
| **运行时校验** | zod | ^4.3.6 | TypeScript 优先的 schema 校验 |

---

## Q4：使用这些框架或组件的好处是什么？（10分）

| 组件 | 核心优势 |
|------|----------|
| **Axum** | 类型安全提取器（编译期检查参数）、原生 WebSocket、零成本抽象 |
| **Tokio** | 全异步非阻塞、`spawn` 轻量后台任务（落库/扫描/通知）、工作窃取调度 |
| **SQLx** | 编译期 SQL 语法验证、异步连接池自动管理、直接写 SQL 无 ORM 开销 |
| **Redis** | ZSET 天然游标分页、多路复用连接低开销、O(1)~O(log N) 读写 |
| **Serde** | 编译期生成序列化代码、`serde_json::Value` 灵活处理动态 JSON |
| **HMAC+SHA2** | 密码学强度不可伪造、无状态 Token、每 Bot 独立密钥 |
| **UUID** | 全球唯一、前缀分类（`u_/b_/g_/f_/ntf_`）快速识别实体 |
| **Tracing** | 结构化 JSON 日志、异步非阻塞写入、按天自动切分 |
| **Thiserror** | `#[derive(Error)]` 消除样板代码、`AppError` 枚举统一错误管理 |
| **Vue 3** | Composition API、响应式数据绑定、SFC 单文件组件 |
| **Naive UI** | 60+ 现成组件、Tree Shaking 按需加载、TypeScript 完整类型 |
| **Vite** | 原生 ESM 极速 HMR、Rollup 打包、开箱即用 TypeScript |
| **httpx** | 异步 HTTP/2、连接池复用、兼容 requests API |
| **websockets** | 异步 WS 长连接、自动重连、简洁回调式 API |

---

## Q5：你项目的类图是什么样的？（10分）

由于 Rust 不是 OOP 语言（没有继承，使用 trait + struct），以下展示等价的**类型结构图**：

```
┌─────────────────────────────────────────────────────────────────┐
│                        models/                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌───────────────┐     │
│  │ User         │    │ Bot          │    │ Group         │     │
│  ├──────────────┤    ├──────────────┤    ├───────────────┤     │
│  │ user_id: Str │    │ bot_id: Str  │    │ group_id: Str │     │
│  │ name: Str    │    │ owner_id: Str│    │ group_code:Opt│     │
│  │ account: Opt │    │ name: Str    │    │ creator_id:Str│     │
│  │ password: Opt│    │ description  │    │ name: Str     │     │
│  │ id_number:Str│    │ avatar_url   │    │ description   │     │
│  │ avatar_url   │    │ status: Str  │    │ avatar_url    │     │
│  │ created_at   │    │ token: Str   │    │ is_public:Bool│     │
│  │ updated_at   │    │ secret: Str  │    │ status: Str   │     │
│  └──────┬───────┘    │ created_at   │    │ created_at    │     │
│         │            │ updated_at   │    │ updated_at    │     │
│         │ 1          └──────┬───────┘    └───────┬───────┘     │
│         │ owns              │                    │              │
│         │ many              │ member_of          │ contains     │
│         │                   │                    │              │
│  ┌──────┴───────────────────┴────────────────────┴──────────┐  │
│  │                      Message                              │  │
│  ├───────────────────────────────────────────────────────────┤  │
│  │ msg_id: i64        group_id: Str     sender_id: Str       │  │
│  │ content: Str       msg_type: Str     idempotency_key: Opt │  │
│  │ created_at: Str                                           │  │
│  └──────────────────────────┬────────────────────────────────┘  │
│                             │ references                        │
│  ┌──────────────────────────┴───────────────────┐              │
│  │ File                                         │              │
│  ├──────────────────────────────────────────────┤              │
│  │ file_id: Str       owner_id: Str             │              │
│  │ content_hash: Str  filename: Str             │              │
│  │ size: i64          mime_type: Str            │              │
│  │ storage_path: Str  created_at: Str           │              │
│  └──────────────────┬───────────────────────────┘              │
│                     │ has                                       │
│  ┌──────────────────┴──────────────┐                           │
│  │ FileScanRecord                  │                           │
│  ├─────────────────────────────────┤                           │
│  │ file_id: Str (FK)               │                           │
│  │ status: Str (pending/           │                           │
│  │   scanning/clean/suspicious)    │                           │
│  │ risk_level: Str (low/med/high)  │                           │
│  │ scan_result: Opt<Str>           │                           │
│  └─────────────────────────────────┘                           │
│                                                                 │
│  ┌──────────────────────────────────────────────┐              │
│  │ Notification                                  │              │
│  ├──────────────────────────────────────────────┤              │
│  │ notification_id: Str   recipient_user_id: Str│              │
│  │ kind: Str              title/content: Str    │              │
│  │ requires_action: Bool  is_resolved/read: Bool│              │
│  │ action_payload: Opt   related_*: Opt         │              │
│  └──────────────────────────────────────────────┘              │
│                                                                 │
│  ┌──────────────────────────────────────────────────┐          │
│  │ AuditLog                                          │          │
│  ├──────────────────────────────────────────────────┤          │
│  │ log_id: i64      actor_type/actor_id: Str        │          │
│  │ action: Str       resource_type/id: Str          │          │
│  │ user_id/bot_id/group_id: Opt<Str>                │          │
│  │ details: Opt<Str>  created_at: Str               │          │
│  └──────────────────────────────────────────────────┘          │
│                                                                 │
│  ┌─────────────────────────────────────────────────┐           │
│  │ GroupJoinRequest                                 │           │
│  ├─────────────────────────────────────────────────┤           │
│  │ request_id: Str    group_id: Str                │           │
│  │ bot_id: Str        requester_user_id: Str       │           │
│  │ approver_user_id   request_type: Str            │           │
│  │ request_reason     status: pending/approved/... │           │
│  └─────────────────────────────────────────────────┘           │
└─────────────────────────────────────────────────────────────────┘

关系说明：
  User 1──N Bot       (一个用户拥有多个 Bot)
  Bot  N──M Group     (Bot 与群多对多，通过 group_members 关联)
  Bot  1──N Message   (一个 Bot 发送多条消息)
  Group 1──N Message  (一个群包含多条消息)
  Message N──M File   (消息通过 file_references 关联文件)
  File  1──1 FileScanRecord  (每个文件一条扫描记录)
```

---

## Q6：你项目的对象图是什么样的？（10分）

以下展示**运行时的对象实例关系**（以一次典型消息发送为例）：

```
┌─────────────────────────────────────────────────────────────────────┐
│                       运行时对象实例                                   │
│                                                                      │
│  ┌─────────────────────────┐                                         │
│  │ AppState (全局单例)     │                                         │
│  ├─────────────────────────┤                                         │
│  │ config: Config          │                                         │
│  │ pool: PgPool ───────────┼──→ 10 个 DB 连接                         │
│  │ ws_manager: WsManager ──┼──→ Arc<RwLock<HashMap>>                 │
│  │ msg_rec_mgr ────────────┼──→ MessageRecordManager                 │
│  └───────────┬─────────────┘                                         │
│              │                                                        │
│  ┌───────────┴───────────────────────────────────────────────────┐  │
│  │ MessageRecordManager                                           │  │
│  ├────────────────────────────────────────────────────────────────┤  │
│  │ pool: PgPool (clone)                                           │  │
│  │ cache: RedisMessageCache ───→ MultiplexedConnection ──→ Redis  │  │
│  │                                                                  │  │
│  │ Redis 数据:                                                      │  │
│  │  msg:102 ──→ {"msg_id":102, "group_id":"g_abc", ...}            │  │
│  │  grp:g_abc ──→ ZSET {score=100→msg100, score=101→msg101, ...}  │  │
│  │  idem:b_x:g_abc:key123 ──→ 102                                  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌────────────────────────────────────────────────────────┐         │
│  │ WsManager                                               │         │
│  ├────────────────────────────────────────────────────────┤         │
│  │ connections: HashMap {                                  │         │
│  │   "b_001" → [UnboundedSender, UnboundedSender]         │         │
│  │   "b_002" → [UnboundedSender]                          │         │
│  │ }                                                       │         │
│  └────────────────────────────────────────────────────────┘         │
│                                                                      │
│  ┌──────────────────────────────────────────┐                       │
│  │ 当前请求实例 (send_message)               │                       │
│  ├──────────────────────────────────────────┤                       │
│  │ BotAuth {                                │                       │
│  │   bot_id: "b_001"                        │                       │
│  │   owner_id: "u_abc"                      │                       │
│  │   name: "我的Bot"                         │                       │
│  │   avatar_url: None                       │                       │
│  │ }                                        │                       │
│  │                                          │                       │
│  │ CreateMessageRequest {                   │                       │
│  │   group_id: "g_xyz"                      │                       │
│  │   content: {"text": "你好"}               │                       │
│  │   msg_type: "text"                       │                       │
│  │   idempotency_key: "req_20240512_001"    │                       │
│  │ }                                        │                       │
│  └──────────────────────────────────────────┘                       │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ Database (PostgreSQL)                                          │   │
│  │                                                                │   │
│  │ users: "u_abc" → {name:"张三", account:"admin", ...}          │   │
│  │ bots:  "b_001" → {name:"我的Bot", owner_id:"u_abc", ...}      │   │
│  │ groups: "g_xyz" → {name:"技术群", creator_id:"u_abc", ...}    │   │
│  │ group_members: ("g_xyz", "b_001")                              │   │
│  │ messages: 102 → {group_id:"g_xyz", content:"...", ...}        │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────┐        │
│  │ File System                                               │        │
│  │ ./assets/files/                                           │        │
│  │   f_001/                                                  │        │
│  │     report.pdf                                            │        │
│  │   f_002/                                                  │        │
│  │     avatar.png                                            │        │
│  └─────────────────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────────────┘
```

**生命周期说明**：
- `AppState`：应用级单例，整个进程生命周期
- `PgPool`：连接池复用，请求间共享
- `MultiplexedConnection`：Redis 连接复用
- `BotAuth`/`UserAuth`：请求级，由中间件注入，请求结束即销毁
- `WsManager`：应用级，管理所有 WebSocket 长连接

---

## Q7：你项目的顺序图是什么样的？（10分）

### 消息发送完整顺序图

```
  Bot Client        Axum Router      Middleware       Handler          MsgRecMgr       Redis        PostgreSQL     WsManager      Audit/Notif
     │                  │                │               │                │              │               │              │              │
     │ POST /msg/send   │                │               │                │              │               │              │              │
     │─────────────────>│                │               │                │              │               │              │              │
     │                  │                │               │                │              │               │              │              │
     │                  │ Bearer token   │               │                │              │               │              │              │
     │                  │───────────────>│               │                │              │               │              │              │
     │                  │                │               │                │              │               │              │              │
     │                  │                │ 1.提取 token  │                │              │               │              │              │
     │                  │                │ 2.解析 bot_id │                │              │               │              │              │
     │                  │                │ 3.查 DB 拿    │                │              │               │              │              │
     │                  │                │   bot secret  │                │              │               │              │              │
     │                  │                │──────────────>│                │              │               │              │              │
     │                  │                │<──────────────│                │              │               │              │              │
     │                  │                │               │                │              │               │              │              │
     │                  │                │ 4.验证 HMAC   │                │              │               │              │              │
     │                  │                │ 5.注入BotAuth │                │              │               │              │              │
     │                  │                │──────────────>│                │              │               │              │              │
     │                  │                │               │                │              │               │              │              │
     │                  │                │               │ 6.检查群存在   │              │               │              │              │
     │                  │                │               │──────────────>│              │               │              │              │
     │                  │                │               │<──────────────│              │               │              │              │
     │                  │                │               │ (PostgreSQL)  │              │               │              │              │
     │                  │                │               │               │              │               │              │              │
     │                  │                │               │ 7.检查Bot     │              │               │              │              │
     │                  │                │               │   是群成员    │              │               │              │              │
     │                  │                │               │──────────────>│              │               │              │              │
     │                  │                │               │<──────────────│              │               │              │              │
     │                  │                │               │               │              │               │              │              │
     │                  │                │               │ 8.幂等检查    │              │               │              │              │
     │                  │                │               │──────────────>│              │               │              │              │
     │                  │                │               │               │ GET idem:*   │               │              │              │
     │                  │                │               │               │─────────────>│               │              │              │
     │                  │                │               │               │<─────────────│ (miss)        │              │              │
     │                  │                │               │               │              │               │              │              │
     │                  │                │               │               │ 回退查 PG    │               │              │              │
     │                  │                │               │               │─────────────────────────────>│              │              │
     │                  │                │               │               │<─────────────────────────────│ (miss)       │              │              │
     │                  │                │               │<──────────────│              │               │              │              │
     │                  │                │               │               │              │               │              │              │
     │                  │                │               │ 9.分配msg_id  │              │               │              │              │
     │                  │                │               │──────────────>│              │               │              │              │
     │                  │                │               │               │ AtomicI64    │               │              │              │
     │                  │                │               │               │ fetch_add(1) │               │              │              │
     │                  │                │               │               │              │               │              │              │
     │                  │                │               │ 10.写Redis    │              │               │              │              │
     │                  │                │               │──────────────>│              │               │              │              │
     │                  │                │               │               │ SET msg:103  │               │              │              │
     │                  │                │               │               │ ZADD grp:*   │               │              │              │
     │                  │                │               │               │ SET idem:*   │               │              │              │
     │                  │                │               │               │─────────────>│               │              │              │
     │                  │                │               │               │<─────────────│ (OK)          │              │              │
     │                  │                │               │<──────────────│              │               │              │              │
     │                  │                │               │               │              │               │              │              │
     │                  │                │               │ 11.异步落库   │              │               │              │              │
     │                  │                │               │──────────────>│              │               │              │              │
     │                  │                │               │               │ tokio::spawn │               │              │              │
     │                  │                │               │               │─────────────────────────────>│              │              │
     │                  │                │               │               │ INSERT INTO messages ...     │              │              │
     │                  │                │               │               │ (异步，不阻塞响应)            │              │              │
     │                  │                │               │               │              │               │              │              │
     │                  │                │               │ 12.触发副作用 │              │               │              │              │
     │                  │                │               │──────────────>│              │               │              │              │
     │                  │                │               │               │ FileManager::on_msg_persisted│             │              │
     │                  │                │               │               │   → file_references INSERT   │             │              │
     │                  │                │               │               │ notify_group_owner_scan      │             │              │
     │                  │                │               │               │   → FileScanRepository       │             │              │
     │                  │                │               │               │              │               │              │              │
     │                  │                │               │ 13.WebSocket  │              │               │              │              │
     │                  │                │               │   广播        │              │               │              │              │
     │                  │                │               │──────────────────────────────────────────────────────────>│              │
     │                  │                │               │               │              │               │ broadcast()  │              │
     │                  │                │               │               │              │               │ to members   │              │
     │                  │                │               │               │              │               │ + superadmin │              │
     │                  │                │               │               │              │               │              │              │
     │                  │                │               │ 14.审计日志   │              │               │              │              │
     │                  │                │               │───────────────────────────────────────────────────────────────────────>│
     │                  │                │               │               │              │               │              │ record()     │
     │                  │                │               │               │              │               │              │ (best-effort)│
     │                  │                │               │               │              │               │              │              │
     │                  │                │               │ 15.返回201    │              │               │              │              │
     │                  │<───────────────│<──────────────│<──────────────│              │               │              │              │
     │<─────────────────│                │               │               │              │               │              │              │
     │  {"msg_id":103,  │                │               │               │              │               │              │              │
     │   "content":..., │                │               │               │              │               │              │              │
     │   "created_at":..│                │               │               │              │               │              │              │
     │  }               │                │               │               │              │               │              │              │
```

### 消息读取顺序图（首次访问触发懒加载）

```
  Bot Client        Handler         MsgRecMgr        Redis          PostgreSQL
     │                 │                │               │                │
     │ GET /messages   │                │               │                │
     │────────────────>│                │               │                │
     │                 │ 验证群成员      │               │                │
     │                 │───(DB)────────>│               │                │
     │                 │                │               │                │
     │                 │ get_group_msgs │               │                │
     │                 │───────────────>│               │                │
     │                 │                │ ZCARD grp:*   │                │
     │                 │                │──────────────>│                │
     │                 │                │<──────────────│ (0, 无缓存)    │
     │                 │                │               │                │
     │                 │                │ preload_group │                │
     │                 │                │ SELECT * FROM │                │
     │                 │                │  messages     │                │
     │                 │                │  WHERE grp=.. │                │
     │                 │                │──────────────>│                │
     │                 │                │<──────────────│ (500 条消息)   │
     │                 │                │               │                │
     │                 │                │ 批量 SET +    │                │
     │                 │                │ ZADD + SET    │                │
     │                 │                │──────────────>│                │
     │                 │                │<──────────────│ (OK)           │
     │                 │                │               │                │
     │                 │                │ ZREVRANGEBYSCORE              │
     │                 │                │  grp:* base_id-1 0 LIMIT 50   │
     │                 │                │──────────────>│                │
     │                 │                │<──────────────│ (50 条)        │
     │                 │<───────────────│               │                │
     │<────────────────│ (JSON 响应)    │               │                │
```

---

## Q8：你的项目中是否使用了任何设计模式？（10分）

共使用了 5 种设计模式：

1. **Repository 模式**：每个数据库实体对应一个独立的 Repository 结构体（如 `GroupRepository`、`MessageRepository`），封装所有 SQL 操作，业务层不直接接触 SQL，切换数据库只需修改 Repository 层。

2. **依赖注入模式**：通过 Axum 的 `State` 提取器注入共享状态（连接池、配置），通过 `Extension` 提取器注入中间件产出的认证信息，编译期类型安全检查，无运行时反射。

3. **参数对象模式**：将多参数聚合为结构体（如 `UpdateGroupProfile`、`NewMessage`），避免长参数列表，调用处语义清晰。

4. **外观模式**：`BotService`、`GroupService`、`FileManager` 等 Service 对外暴露简洁接口，内部组合多个 Repository 协作完成复杂操作（如 Bot 删除时级联清理文件引用）。

5. **单例模式**：全局消息 ID 计数器 `NEXT_MSG_ID` 使用 `static AtomicI64` 实现进程级唯一，启动时从数据库加载当前最大值初始化。

---

## Q9：你将来如何处理组件变更或功能扩展？（10分）

核心方法是**模块化分层 + 编译期保障 + 渐进式演进**：

1. **影响范围分析法**：任何变更首先通过模块边界判断影响面——修改数据模型只影响 models/ 和 repositories/，修改业务规则只影响 services/，修改接口只影响 handlers/。Rust 编译器会精确报告所有受影响的调用点，无需人工追踪。

2. **开闭原则**：新增功能通过新增模块实现（新 handler、新 service、新 repository），不修改已有核心逻辑。例如新增"群投票"功能只需新增 `models/poll.rs` + `repositories/poll_repository.rs` + `handlers/poll.rs`，不触碰消息、用户等已有模块。

3. **接口向后兼容**：API 响应字段只增不删，旧字段保留并标记 `#[deprecated]`，给客户端充分的迁移时间。废弃字段至少保留一个大版本周期。

4. **测试回归保障**：每次变更后运行全量测试（单元测试 + 集成测试 + clippy），CI 流水线自动检查零警告零错误。


---

## Q10：随着项目迭代，你如何处理遗留代码？（10分）

核心思路是**渐进式演进，绝不一次性大规模重写**：

1. **废弃而非删除**：旧接口、旧字段先标记为废弃，在文档和日志中给出替代方案，保留至少一个大版本周期后再移除，给所有依赖方充分的迁移时间。

2. **适配器过渡**：内部实现重构时，保留旧函数签名作为适配器，内部委托到新实现，调用方无需立即修改。

3. **静态分析自动化**：将代码质量检查集成到 CI 流水线——未使用的变量和导入、过长函数、重复代码等坏味道在提交阶段即被拦截，防止遗留代码积累。

4. **测试先行重构**：重构前确保有足够的测试覆盖核心流程，重构后运行全量测试，通过即证明行为未被破坏。

5. **模块边界隔离变更**：分层架构天然限制变更波及范围——替换数据库只改数据访问层，替换缓存只改缓存模块，业务逻辑层不受影响。

6. **原子提交与可追溯**：每次提交只做一件事，提交信息写清楚"为什么"。重构前打 tag，出问题时可以精确回退到任意历史版本。

7. **小步提交、频繁合并**：大功能拆分为多个小 PR，每个都能独立合并、独立测试、独立回滚。未完成功能用 Feature Flag 隐藏，不影响主干稳定性。
