# 群聊平台后端架构文档

> 基于 Rust + Axum + PostgreSQL + Redis 的群聊消息平台。

---

## 一、系统架构

### 1.1 整体架构分层

```
┌─────────────────────────────────────────────────┐
│                   HTTP / WS                      │
├─────────────────────────────────────────────────┤
│              Middleware 层                        │
│    require_user_auth / require_bot_auth          │
├─────────────────────────────────────────────────┤
│              Handler 层                          │
│    auth / bot / group / message / file / ...     │
├─────────────────────────────────────────────────┤
│              Service 层                          │
│    BotService / GroupService / MessageService    │
│    FileManager / FileScan / Notification / Audit │
├──────────────────┬──────────────────────────────┤
│  MessageRecord   │  Repository 层                │
│  Manager (Redis) │  *Repository (PostgreSQL)     │
├──────────────────┴──────────────────────────────┤
│               DB / Cache                         │
│        PostgreSQL + Redis                        │
└─────────────────────────────────────────────────┘
```

- **Middleware 层**：基于 Axum `middleware::from_fn_with_state`，通过 `Extension` 注入认证信息
- **Handler 层**：路由处理函数，负责参数解析、权限校验、调用 Service、返回响应
- **Service 层**：业务逻辑封装，如 Bot 生命周期管理、文件引用清理、审计日志
- **Message Record Manager**：独立于 Service 层的消息缓存管理器，对接 Redis
- **Repository 层**：数据访问层，封装 SQL 查询
- **Cache 层**：Redis 消息缓存，提供三种索引结构

### 1.2 路由设计

| 路由 | 方法 | 认证 | 说明 |
|------|------|------|------|
| `/health` | GET | 无 | 健康检查 |
| `/api/v1/auth/register` | POST | 无 | 用户注册（自动创建默认 Bot） |
| `/api/v1/auth/login` | POST | 无 | 用户登录 |
| `/api/v1/bots/{bot_id}` | GET | 无 | 公开 Bot 信息 |
| `/api/v1/groups/search` | GET | 无 | 按群号搜索群聊 |
| `/api/v1/groups/{group_id}` | GET | 无 | 公开群聊信息 |
| `/api/v1/file/download/{file_id}/{filename}` | GET | 无 | 文件下载/预览 |
| `/api/v1/users/me` | GET/PUT | User | 当前用户信息 |
| `/api/v1/users/delete` | DELETE | User | 删除用户 |
| `/api/v1/bots` | POST/GET | User | 创建/列出 Bot |
| `/api/v1/bots/search` | GET | User | 搜索 Bot |
| `/api/v1/bots/{bot_id}` | PUT/DELETE | User | 更新/删除 Bot |
| `/api/v1/groups` | POST/GET | User | 创建/列出群聊 |
| `/api/v1/groups/{group_id}` | PUT/DELETE | User | 更新/删除群聊 |
| `/api/v1/groups/join` | POST | User | 加入群聊 |
| `/api/v1/groups/{group_id}/leave` | DELETE | User | 退群 |
| `/api/v1/groups/{group_id}/members` | GET | User | 群成员列表 |
| `/api/v1/groups/{group_id}/members/{bot_id}` | DELETE | User | 移除群成员 |
| `/api/v1/audit/logs` | GET | User | 审计日志列表 |
| `/api/v1/audit/logs/export` | GET | User | 导出审计日志 CSV |
| `/api/v1/notifications` | GET | User | 通知列表 |
| `/api/v1/notifications/{id}/read` | POST | User | 标记通知已读 |
| `/api/v1/notifications/{id}/approve` | POST | User | 审批加群申请 |
| `/api/v1/notifications/{id}/reject` | POST | User | 拒绝加群申请 |
| `/api/v1/groups/{group_id}/messages` | GET | Bot | 获取群消息历史 |
| `/api/v1/message/send` | POST | Bot | 发送消息 |
| `/api/v1/file/upload` | POST | Bot | 上传文件 |
| `/api/v1/file/{file_id}` | DELETE | Bot | 删除文件 |
| `/ws` | GET | - | WebSocket 连接 |

### 1.3 数据库表结构

#### users（用户表）
| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| user_id | TEXT | PK | `u_{uuid}` |
| name | TEXT | NOT NULL | 用户昵称 |
| account | TEXT | UNIQUE | 登录账号 |
| password_hash | TEXT | - | SHA256(salt + password + pepper) |
| id_number | TEXT | UNIQUE | 预留实名认证字段 |
| avatar_url | TEXT | - | 头像 URL |
| created_at | TEXT | - | RFC 3339 |
| updated_at | TEXT | - | RFC 3339 |

#### bots（Bot 表）
| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| bot_id | TEXT | PK | `b_{uuid}` |
| owner_id | TEXT | FK→users | 所有者用户 ID |
| name | TEXT | NOT NULL | Bot 名称 |
| description | TEXT | - | Bot 描述 |
| avatar_url | TEXT | - | 头像 URL |
| status | TEXT | DEFAULT 'active' | active/inactive |
| token | TEXT | UNIQUE | 预生成的 Bot Token |
| secret | TEXT | NOT NULL | HMAC 签名密钥 |
| created_at | TEXT | - | RFC 3339 |
| updated_at | TEXT | - | RFC 3339 |

#### groups（群聊表）
| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| group_id | TEXT | PK | `g_{uuid}` |
| group_code | TEXT | UNIQUE | 自定义群号 |
| creator_id | TEXT | FK→users | 创建者 |
| name | TEXT | NOT NULL | 群名称 |
| description | TEXT | - | 群描述 |
| avatar_url | TEXT | - | 群头像 |
| is_public | BOOLEAN | DEFAULT FALSE | 是否公开 |
| status | TEXT | DEFAULT 'active' | active/inactive |
| created_at | TEXT | - | RFC 3339 |
| updated_at | TEXT | - | RFC 3339 |

#### group_members（群成员表）
| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| group_id | TEXT | PK, FK→groups | 群 ID |
| member_id | TEXT | PK, FK→bots | Bot ID |
| member_type | TEXT | DEFAULT 'bot' | 成员类型 |
| joined_at | TEXT | - | RFC 3339 |

#### messages（消息表）
| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| msg_id | BIGINT | PK, IDENTITY | 自增消息 ID |
| group_id | TEXT | FK→groups | 群 ID |
| sender_id | TEXT | FK→bots | 发送者 Bot ID |
| content | TEXT | NOT NULL | JSON 内容 |
| msg_type | TEXT | DEFAULT 'text' | text/file |
| idempotency_key | TEXT | - | 幂等键 |
| created_at | TEXT | - | RFC 3339 |

#### notifications（通知表）
| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| notification_id | TEXT | PK | `ntf_{uuid}` |
| recipient_user_id | TEXT | FK→users | 接收者 |
| kind | TEXT | NOT NULL | 通知类型 |
| title | TEXT | NOT NULL | 标题 |
| content | TEXT | NOT NULL | 内容 |
| requires_action | BOOLEAN | DEFAULT FALSE | 是否需要操作 |
| is_resolved | BOOLEAN | DEFAULT FALSE | 是否已处理 |
| is_read | BOOLEAN | DEFAULT FALSE | 是否已读 |
| action_payload | TEXT | - | JSON 操作数据 |
| related_* | TEXT | - | 关联 ID |
| created_at | TEXT | NOT NULL | RFC 3339 |

此外还有 `audit_logs`、`files`、`file_scan_records`、`file_uploaders`、`file_references`、`group_join_requests` 等辅助表，详见 `db/schema.rs`。

### 1.4 Redis 缓存结构

用于消息缓存的三种 Redis 数据结构：

| 键模式 | 类型 | 说明 |
|--------|------|------|
| `msg:{msg_id}` | String | JSON 序列化的 `MessageWithSenderRow`，用于按 ID 快速查询 |
| `grp:{group_id}` | ZSET | Score=msg_id，用于群消息的游标分页查询 |
| `idem:{sender_id}:{group_id}:{key}` | String | 值为 msg_id，用于幂等性检查 |

---

## 二、用户体验设计

### 2.1 API 设计风格

- **RESTful 风格**：资源导向的路由设计，标准 HTTP 方法（GET/POST/PUT/DELETE）
- **统一响应格式**：所有响应均为 JSON
- **统一错误格式**：
  ```json
  {
    "code": "error_code",
    "message": "中文错误描述",
    "status": 400
  }
  ```
- **Token 认证**：通过 `Authorization: Bearer <token>` 头部传递

### 2.2 用户注册与登录体验

- 注册即创建用户 + 自动创建默认 Bot，一步完成
- 登录返回用户 token 和是否是超级管理员标识
- 账号密码校验在前端友好提示（长度、字符集规则明确）
- 超级管理员通过环境变量配置，启动时自动 seed

### 2.3 群聊交互

- 支持公开群（直接加入）和私密群（审批加入）
- 同时支持 group_id 和 group_code 两种标识加入群聊
- 超级管理员的 Bot 拥有全局群聊访问权限
- 群消息游标分页（cursor-based pagination），返回旧→新顺序
- 分页参数自动 clamp（limit 1~100）

### 2.4 消息发送

- 支持 text 和 file 两种消息类型
- **幂等性保证**：通过 `idempotency_key` 防止重复发送
- 消息内容为 JSON 格式，灵活扩展
- 发送成功后消息通过 WebSocket 实时推送给所有群成员和超级管理员 Bot

### 2.5 文件处理

- 文件上传后自动计算 SHA256 哈希，相同内容的文件自动复用（秒传）
- 文件下载支持 `Content-Disposition: inline` 浏览器预览
- 中文文件名通过 `percent-encoding` 和 `filename*` 兼容
- 文本类文件自动追加 `charset=utf-8`
- 支持断点续传友好的 Multipart 上传

### 2.6 通知系统

- 用户可查看通知列表及未读/待处理统计
- 加群申请的通知支持直接审批（同意/拒绝）
- 审批后自动通知申请方结果
- 文件安全扫描异常时通知群主和 Bot 所有者

### 2.7 审计日志

- 全局操作均记录审计日志
- 超级管理员可查询和导出 CSV
- 多维过滤（用户、Bot、群聊、操作类型、时间范围）

### 2.8 WebSocket 实时推送

- 消息发送后实时推送到所有在线群成员
- Bot 可维持 WebSocket 长连接接收消息事件
- 事件格式：`{ type, payload, timestamp }`
- 支持多设备同时在线

---

## 三、安全性具体设计

### 3.1 认证体系

#### 双 Token 系统

系统使用两种独立 token，通过格式区分和中间件隔离：

**User Token**（用户身份）：
- 格式：`u:{user_id}:{timestamp}:{HMAC-SHA256签名}`
- 使用 JWT_SECRET 作为 pepper 参与签名
- 有效期从环境变量 `TOKEN_EXPIRY_SECS` 读取（默认 24 小时）
- 由 `require_user_auth` 中间件验证

**Bot Token**（Bot 身份）：
- 格式：`{bot_id}:{timestamp}:{HMAC-SHA256签名}`
- 使用 Bot 独立 `secret` 字段签名（每个 Bot 不同）
- 有效期同上
- 由 `require_bot_auth` 中间件验证
- 验证仓库中 Bot 状态，非 active 或已删除的 Bot 无法使用

#### 密码安全

```
password_hash = base16(SHA256(salt || password || pepper))
存储格式: "{salt}${hash}"
```

- salt：每个用户独立的随机 UUID（去横线）
- pepper：JWT_SECRET 环境变量
- 避免明文存储，防止彩虹表攻击

#### Token 验证流程

```rust
// Bot Token 验证
1. 提取 Authorization: Bearer <token>
2. 按 : 分割验证格式（3 段）
3. 提取 bot_id，查询 Bot 的 secret
4. 校验 HMAC 签名
5. 校验时间戳未过期
6. 校验 Bot 状态为 active
7. 注入 BotAuth { bot_id, owner_id, name, avatar_url } 到请求扩展
```

### 3.2 授权体系

#### 权限分级

| 角色 | 权限范围 |
|------|----------|
| 超级管理员 | 全局 Bot 管理、全量群聊查看、审计日志访问、Bot 全局群聊访问 |
| 普通用户 | 管理自己名下的 Bot、管理自己创建的群聊 |
| Bot | 在自己所在的群聊中发消息、查消息、上传文件 |

#### 关键授权检查点

- **群消息发送**：验证 Bot 是群成员（或超级管理员 Bot）
- **群消息查看**：同上
- **群聊管理**：只能由创建者或超级管理员操作
- **Bot 管理**：只能由 Bot 所有者或超级管理员操作
- **加入私密群**：需要群创建者或 Bot 所有者审批
- **文件删除**：只能删除自己上传的文件

### 3.3 输入验证

- **账号**：4-32 位，仅字母数字下划线
- **密码**：8-64 位，必须含字母和数字，仅可见 ASCII
- **文件名**：剔除控制字符和路径分隔符
- **文件大小**：受 `MAX_FILE_SIZE_MB` 限制（默认 100MB）
- **分页参数**：自动 clamp 防止不合理值

### 3.4 文件安全

#### 文件上传安全

- 内容哈希去重（SHA256），防止存储重复恶意文件
- 文件存储路径使用 UUID 隔离，避免路径遍历
- 上传后自动触发异步安全扫描

#### 文件安全扫描

上传的文件自动进行多维度检测：

| 检测项 | 方法 | 风险等级 |
|--------|------|----------|
| 可疑扩展名 | 黑名单：`.exe/.dll/.bat/.ps1/.jar` 等 14 种 | high |
| 双重扩展名伪装 | 如 `photo.jpg.exe` | high |
| 文件名风险关键词 | `keygen/crack/trojan/ransom` 等 8 种 | medium |
| 可执行文件魔数 | MZ(PE) / ELF 文件头检测 | high |
| 文本内容违规 | 中英文非法关键词（木马、malware 等 8 种） | high |
| 文件大小校验 | 实际大小与声明大小是否一致 | - |

扫描结果状态：`clean` / `suspicious` / `failed`

#### 扫描结果处置

- 扫描结果持久化到 `file_scan_records` 表
- 可疑文件通过通知系统告警群主和上传者
- 所有扫描操作记录审计日志

### 3.5 幂等性防护

- 每条消息需携带 `idempotency_key`
- 数据库唯一索引确保 (`sender_id`, `group_id`, `idempotency_key`) 组合唯一
- Redis 缓存层也维护幂等索引，实现快速检查
- 重复请求返回已存在的消息，不重复落库

### 3.6 审计日志

所有关键操作记录审计日志：

```
action = "auth.register" | "auth.login" | "bot.create" | "bot.delete" |
          "group.create" | "group.join_public" | "group.join_request.create" |
          "message.send" | "file.upload" | "file.delete" |
          "file.scan.queued" | "file.scan.completed" |
          "notification.join_request.approve" | ...
```

每条记录包含操作者类型、操作者 ID、用户/Bot/群聊关联、操作详情（JSON）。

### 3.7 外键约束与级联

- messages 引用 groups 和 bots（外键约束）
- 删除群聊时：先清理消息引用 → 移除群成员 → 删除群
- 文件引用计数器：引用数为零时自动清理物理文件
- Bot 删除关联的文件引用清理

---

## 四、性能具体设计

### 4.1 消息缓存架构（核心性能策略）

```
发送消息:
  Client → POST /message/send
          → 验证权限
          → 幂等检查 (Redis)
          → 分配 msg_id (AtomicI64)
          → 写入 Redis 缓存 (同步)
          → tokio::spawn 异步写入 PostgreSQL
          → WebSocket 广播
          → 返回响应

读取消息:
  Client → GET /groups/{id}/messages
          → 验证权限
          → 检查 Redis 是否有该群缓存
          → 无: 从 PostgreSQL 批量预加载到 Redis
          → 有: 从 Redis ZSET 游标分页读取
```

#### 写路径（先缓存后落库）

```
1. 验证通过后，通过 AtomicI64 分配 msg_id
2. 同步写入 Redis（三种索引同时更新）
3. tokio::spawn 异步持久化到 PostgreSQL
4. 立即返回 HTTP 201
```

关键优势：消息发送响应时间 = Redis 写入时间，不受 DB I/O 影响。

#### 读路径（懒加载 + 缓存优先）

```
1. 首次查询某群聊消息时，从 PostgreSQL 加载全部消息到 Redis
2. 后续查询全部从 Redis 读取
3. Redis ZSET 使用 ZREVRANGEBYSCORE 实现游标分页
```

#### 消息 ID 生成

```rust
static NEXT_MSG_ID: AtomicI64 = AtomicI64::new(0);
```

- 启动时 `SELECT COALESCE(MAX(msg_id), 0) FROM messages` 初始化
- 运行时原子递增，单进程无锁
- 支持 cache-first 写入模式（预分配 ID）

### 4.2 异步并发模型

- **Tokio 异步运行时**：全异步非阻塞 I/O
- **tokio::spawn 异步任务**：
  - 消息异步落库（不与 HTTP 响应同步等待）
  - 文件扫描任务（后台上执行多维度检测）
  - 通知创建（best-effort，失败仅记录日志）
- **连接池**：
  - PostgreSQL：sqlx `PgPool`，配置 `min_connections` / `max_connections`
  - Redis：`MultiplexedConnection`，多路复用连接
- **WebSocket**：`Arc<RwLock<HashMap>>` 管理连接，`mpsc::UnboundedSender` 发送事件

### 4.3 数据库性能

#### 索引策略

| 表 | 索引 | 说明 |
|----|------|------|
| messages | `(group_id, created_at)` | 群消息历史查询 |
| messages | `(sender_id, group_id, idempotency_key) WHERE idempotency_key IS NOT NULL` | 幂等键唯一性（部分索引） |
| users | `(id_number)` | 实名查询 |
| users | `(account) WHERE account IS NOT NULL` | 登录查询（部分唯一索引） |
| bots | `(owner_id)` | 用户下的 Bot 列表 |
| groups | `(creator_id)` | 用户创建的群聊 |
| groups | `(group_code)` | 群号搜索 |
| group_join_requests | `(approver_user_id, status, created_at)` | 审批人待处理列表 |
| group_join_requests | `(requester_user_id, status, created_at)` | 申请方状态查询 |
| group_join_requests | `(group_id, bot_id, approver_user_id, request_type) WHERE status='pending'` | 防重复申请（部分唯一） |
| notifications | `(recipient_user_id, created_at DESC)` | 通知列表 |
| notifications | `(recipient_user_id, requires_action, is_resolved, created_at DESC)` | 待处理通知筛选 |
| audit_logs | `(created_at)`、`(user_id, created_at)`、`(bot_id, created_at)`、`(group_id, created_at)` | 审计多维查询 |
| files | `(owner_id)`、`(content_hash)` | 文件查重和所属查询 |
| file_scan_records | `(status, updated_at DESC)` | 待扫描文件轮询 |

#### 连接池配置

```
DB_MIN_CONNECTIONS=2    # 最小维持连接数
DB_MAX_CONNECTIONS=10   # 最大连接数
```

### 4.4 缓存性能

#### Redis 操作复杂度

| 操作 | 数据结构 | 时间复杂度 |
|------|----------|------------|
| 插入消息 | String SET + ZSET ZADD | O(1) + O(log N) |
| 按 ID 查 | String GET | O(1) |
| 群消息分页 | ZSET ZREVRANGEBYSCORE | O(log N + M) |
| 幂等检查 | String GET | O(1) |
| 冷启动预加载 | 批量 insert | O(N × log N) |
| 清理群缓存 | ZRANGE + DEL | O(N) |

- ZSET 存储 JSON 而非 msg_id，减少一次内存查询
- 幂等索引独立存储，O(1) 快速去重

#### 懒加载预热

首次查询群消息时触发批量预加载：
```
1. SELECT * FROM messages WHERE group_id = ? ORDER BY msg_id ASC
2. 逐条写入 Redis（SET + ZADD + 幂等索引SET）
3. 后续查询直读 Redis
```

### 4.5 上下午文优化与连接复用

- **Redis `MultiplexedConnection`**：单连接多路复用，避免频繁建连
- **PostgreSQL 连接池**：连接复用，减少握手开销
- **Axum State**：所有共享状态（连接池、管理器）通过 `State` 提取器注入
- **Clone-on-write**：消息缓存通过 Clone 传递，无锁竞争

### 4.6 文件存储性能

- **内容去重**：相同 SHA256 的文件仅存储一次，节省磁盘空间
- **引用计数清理**：`FileManager` 统一管理文件引用，无引用时自动清理磁盘和数据库记录
- **目录结构**：`{storage_path}/{file_id}/{filename}`，按文件 ID 分目录

### 4.7 限流保护

- 通过 `RATE_LIMIT_PER_SECOND` 配置（默认 10）
- 预留的限流配置可对接中间件实现

### 4.8 日志性能

- **双输出**：同时输出到标准输出（终端）和文件（持久化）
- **按天切分**：`tracing_appender::rolling::daily`
- **异步非阻塞**：`tracing_appender::non_blocking` 封装
- **日志级别**：通过 `RUST_LOG` 和 `chat_platform=debug` 控制

### 4.9 配置热加载友好

所有配置通过环境变量注入，启动时读取：

```
# 核心性能参数
SERVER_WORKERS=4               # Axum worker 线程数
DB_MAX_CONNECTIONS=10          # PostgreSQL 最大连接数
DB_MIN_CONNECTIONS=2           # 最小维持连接数
TOKEN_EXPIRY_SECS=86400        # Token 有效期
MAX_FILE_SIZE_MB=100           # 文件上传大小限制
RATE_LIMIT_PER_SECOND=10       # 限流阈值
```

---

## 附录：关键依赖项

| 依赖 | 用途 |
|------|------|
| axum 0.8 | Web 框架 + WebSocket 支持 |
| sqlx 0.8 | PostgreSQL 异步数据库驱动 |
| redis 0.31 | Redis 异步客户端 |
| tokio 1 | 异步运行时 |
| serde / serde_json | 序列化 |
| chrono | 时间处理 |
| hmac / sha2 | Token HMAC 签名 |
| uuid | ID 生成 |
| tracing / tracing-subscriber | 结构化日志 |
| tower-http | CORS 中间件 |
| percent-encoding | 文件名 URL 编码 |
