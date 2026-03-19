# 开发指南 - 下一阶段

## 第3阶段续：WebSocket 实现

### 概览
使用 WebSocket 连接实现实时消息传递。消息应立即传递给在线客户端，并为离线客户端排队。

### 架构
1. **WsManager**：跟踪每个 bot_id 的活动连接
2. **WsActor**：处理单个 WebSocket 连接
3. **消息广播**：将消息传递给接收者的所有在线连接
4. **离线队列**：为离线用户存储消息

### 实现步骤

#### 步骤 1：安装额外依赖
更新 `Cargo.toml`：
```toml
actix-actors = "0.4"
async-trait = "0.1"
```

#### 步骤 2：实现 WebSocket Actor
创建 `src/ws/actor.rs`：
```rust
use actix::prelude::*;
use actix_web_actors::ws::{WebsocketContext, ProtocolError, Message as WsMessage};
use crate::ws::manager::WsManager;

pub struct WsActor {
    bot_id: String,
    ws_manager: web::Data<WsManager>,
}

impl Actor for WsActor {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        // 在管理器中注册连接
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        // 添加到管理器
    }
}

impl StreamHandler<Result<WsMessage, ProtocolError>> for WsActor {
    fn handle(&mut self, msg: Result<WsMessage, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(WsMessage::Text(text)) => {
                // 处理文本消息
            }
            Ok(WsMessage::Binary(bin)) => {
                // 处理二进制消息
            }
            _ => {}
        }
    }
}
```

#### 步骤 3：更新消息发送
发送消息后，在数据库存储时：
1. 从数据库获取接收者 Bot
2. 调用 `ws_manager.broadcast_message(recipient_bot_id, message)`
3. 消息传递给接收者的所有在线连接

#### 步骤 4：更新 WebSocket 处理器
更新 `src/handlers/ws.rs`：
```rust
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    ws_manager: web::Data<WsManager>,
) -> Result<impl Responder, Error> {
    // 从查询参数提取 Token
    let token = req.query::<TokenQuery>().ok()?
        .token;

    // 验证 Token
    let token_payload = verify_token(&token, &config.security.jwt_secret, ...)?;

    // 创建 WebSocket actor
    let actor = WsActor {
        bot_id: token_payload.bot_id.clone(),
        ws_manager,
    };

    // 启动 WebSocket 连接
    ws::start(actor, &req, stream)
}
```

### 测试 WebSocket
创建 `tests/ws_tests.rs`：
```rust
#[actix_web::test]
async fn test_websocket_connection() {
    // 创建测试客户端
    // 连接到 WebSocket
    // 发送消息
    // 验证消息已接收
}
```

---

## 第4阶段：文件管理实现

### 概览
实现带适当访问控制的安全文件上传、存储和下载。

### 架构
1. **文件上传**：多部分表单数据处理
2. **文件存储**：本地文件系统或 S3
3. **文件访问**：下载前的授权检查
4. **文件引用**：验证消息中的 file_id

### 实现步骤

#### 步骤 1：更新文件处理器
更新 `src/handlers/file.rs`：
```rust
pub async fn upload_file(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    http_req: HttpRequest,
    mut payload: web::Payload,
) -> AppResult<HttpResponse> {
    // 验证 Token
    let token = extract_token(&http_req)?;
    let token_payload = verify_token(token, ...)?;

    // 从 bot_id 获取用户
    let user_id = get_user_from_bot(&pool, &token_payload.bot_id).await?;

    // 创建存储目录
    let file_dir = format!("./data/files/{}", user_id);
    fs::create_dir_all(&file_dir)?;

    // 生成 file_id
    let file_id = generate_file_id();
    let storage_path = format!("{}/{}", file_dir, file_id);

    // 读取并保存文件
    let mut file = fs::File::create(&storage_path)?;
    while let Some(chunk) = payload.next().await {
        let data = chunk?;
        file.write_all(&data)?;
    }

    // 获取文件大小
    let size = file.metadata()?.len() as i64;

    // 在数据库中存储元数据
    sqlx::query(
        "INSERT INTO files (file_id, owner_id, filename, size, mime_type, storage_path, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&file_id)
    .bind(&user_id)
    .bind(&filename)
    .bind(size)
    .bind(&mime_type)
    .bind(&storage_path)
    .bind(now())
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(json!({
        "file_id": file_id,
        "url": format!("/api/v1/file/download/{}", file_id),
    })))
}

pub async fn download_file(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    http_req: HttpRequest,
    file_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    // 验证 Token
    let token = extract_token(&http_req)?;
    let token_payload = verify_token(token, ...)?;

    // 获取文件并验证访问权限
    let file = sqlx::query_as::<_, File>(
        "SELECT * FROM files WHERE file_id = ?"
    )
    .bind(file_id.as_str())
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(AppError::FileNotFound)?;

    // 验证用户有访问权限（所有者或消息接收者）
    verify_file_access(&pool, &token_payload.bot_id, &file).await?;

    // 流式传输文件
    let file_data = fs::read(&file.storage_path)?;

    Ok(HttpResponse::Ok()
        .content_type(file.mime_type)
        .body(file_data))
}
```

#### 步骤 2：添加文件引用验证
更新 `src/handlers/message.rs`：
```rust
// 当 msg_type == 'file' 时
if msg_type == "file" {
    // 从内容提取 file_id
    let file_id = msg_req.content.get("file_id")
        .and_then(|v| v.as_str())
        .ok_or(AppError::BadRequest("缺少 file_id".to_string()))?;

    // 验证文件存在且属于发送者
    let file_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM files WHERE file_id = ? AND owner_id = ?)"
    )
    .bind(file_id)
    .bind(&sender_user_id)
    .fetch_one(pool.get_ref())
    .await?;

    if !file_exists {
        return Err(AppError::BadRequest("无效的文件引用".to_string()));
    }
}
```

#### 步骤 3：添加多部分支持
更新 `Cargo.toml`：
```toml
actix-multipart = "0.4"
```

---

## 第5阶段：速率限制实现

### 概览
使用 Redis 和令牌桶算法实现分布式速率限制。

### 实现步骤

#### 步骤 1：Redis 集成
在处理器中更新以检查速率限制：
```rust
pub async fn send_message(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    redis: web::Data<redis::aio::ConnectionManager>,
    http_req: HttpRequest,
    msg_req: web::Json<CreateMessageRequest>,
) -> AppResult<HttpResponse> {
    // 验证 Token
    let token_payload = verify_token(...)?;

    // 检查速率限制
    let rate_key = format!("rate_limit:{}:{}",
        token_payload.bot_id,
        Utc::now().timestamp()
    );

    let current_count: i32 = redis.get(&rate_key).await.unwrap_or(0);

    if current_count >= config.security.rate_limit_per_second as i32 {
        return Err(AppError::RateLimitExceeded);
    }

    // 增加计数器，1 秒过期
    redis.incr(&rate_key, 1).await?;
    redis.expire(&rate_key, 1).await?;

    // 继续消息发送...
}
```

---

## 测试策略

### 单元测试
```bash
cargo test
```

### 集成测试
```bash
# 测试认证流
cargo test --test auth_tests

# 测试消息流
cargo test --test message_tests

# 测试文件操作
cargo test --test file_tests

# 测试 WebSocket
cargo test --test ws_tests
```

### 性能测试
```bash
# 带并发连接的负载测试
ab -n 10000 -c 100 http://localhost:8080/api/v1/message/send
```

---

## 开发检查清单

### 对于 WebSocket 阶段
- [ ] 安装 actix-web-actors
- [ ] 在 `src/ws/actor.rs` 创建 WsActor
- [ ] 更新 `src/ws/manager.rs` 的连接跟踪
- [ ] 实现消息广播
- [ ] 更新消息处理器以广播
- [ ] 创建 WebSocket 测试
- [ ] 更新文档

### 对于文件阶段
- [ ] 在 Cargo.toml 添加多部分支持
- [ ] 实现文件上传处理器
- [ ] 实现文件下载处理器
- [ ] 添加文件引用验证
- [ ] 创建文件目录结构
- [ ] 添加用户删除时的文件清理
- [ ] 创建文件测试

### 对于速率限制阶段
- [ ] 在 main.rs 连接 Redis
- [ ] 用速率限制检查更新处理器
- [ ] 添加 Redis 键过期
- [ ] 创建速率限制测试
- [ ] 监控速率限制指标

---

## 性能优化技巧

1. **数据库查询**
   - 使用预编译语句（已使用 sqlx）
   - 在频繁查询的列添加索引
   - 使用连接池（已配置）

2. **WebSocket**
   - 使用二进制帧获得更好的性能
   - 实现消息批处理
   - 添加背压处理

3. **文件操作**
   - 对大文件使用流式传输
   - 实现分块上传
   - 添加压缩支持

4. **缓存**
   - 缓存 Bot 信息
   - 缓存用户权限
   - 缓存文件元数据

---

## 安全考虑

1. **文件上传**
   - 在存储前验证文件大小
   - 扫描恶意软件（可选）
   - 验证 MIME 类型
   - 在 Web 根目录外存储

2. **速率限制**
   - 每个用户的限制
   - 每个 IP 的限制
   - 可配置的阈值

3. **认证**
   - 考虑升级到 bcrypt/argon2
   - 在登录尝试上添加速率限制
   - 考虑为将来添加 2FA

---

## 监控与可观测性

1. **日志**
   - 使用 tracing 结构化日志
   - 添加请求/响应日志
   - 添加错误跟踪

2. **指标**
   - 消息吞吐量
   - 活动连接
   - 错误率
   - 响应时间

3. **健康检查**
   - 数据库连通性
   - Redis 连通性
   - 磁盘空间
   - 内存使用

---

## 部署考虑

1. **Docker**
   - 为容器化创建 Dockerfile
   - 使用 SQLite/Redis 设置 docker-compose
   - 为数据持久化配置卷挂载

2. **环境**
   - 开发/暂存/生产的单独 .env 文件
   - 安全的密钥管理
   - 数据库备份策略

3. **监控**
   - 添加结构化日志（JSON 日志）
   - 设置 APM/追踪
   - 配置告警

---

## 参考与资源

- [Actix-web 文档](https://actix.rs/)
- [SQLx 文档](https://github.com/launchbadge/sqlx)
- [Tokio 文档](https://tokio.rs/)
- [WebSocket RFC 6455](https://tools.ietf.org/html/rfc6455)
- [HMAC RFC 2104](https://tools.ietf.org/html/rfc2104)

---

**文档创建时间**：2026-03-19
**当前阶段**：第1和第2阶段完成，第3阶段准备开始
