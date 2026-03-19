# Development Guide - Next Phases

## Phase 3 Continuation: WebSocket Implementation

### Overview
Implement real-time message delivery using WebSocket connections. Messages should be delivered immediately to online clients and queued for offline clients.

### Architecture
1. **WsManager**: Track active connections per bot_id
2. **WsActor**: Handle individual WebSocket connections
3. **Message Broadcast**: Deliver messages to all online connections of recipient
4. **Offline Queue**: Store messages for users who are offline

### Implementation Steps

#### Step 1: Install Additional Dependencies
Update `Cargo.toml`:
```toml
actix-actors = "0.4"
async-trait = "0.1"
```

#### Step 2: Implement WebSocket Actor
Create `src/ws/actor.rs`:
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
        // Register connection in manager
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        // Add to manager
    }
}

impl StreamHandler<Result<WsMessage, ProtocolError>> for WsActor {
    fn handle(&mut self, msg: Result<WsMessage, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(WsMessage::Text(text)) => {
                // Handle text message
            }
            Ok(WsMessage::Binary(bin)) => {
                // Handle binary message
            }
            _ => {}
        }
    }
}
```

#### Step 3: Update Message Sending
When sending a message, after storing in database:
1. Get recipient bot from database
2. Call `ws_manager.broadcast_message(recipient_bot_id, message)`
3. Message is delivered to all online connections

#### Step 4: Update WebSocket Handler
Update `src/handlers/ws.rs`:
```rust
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    ws_manager: web::Data<WsManager>,
) -> Result<impl Responder, Error> {
    // Extract token from query parameter
    let token = req.query::<TokenQuery>().ok()?
        .token;

    // Verify token
    let token_payload = verify_token(&token, &config.security.jwt_secret, ...)?;

    // Create WebSocket actor
    let actor = WsActor {
        bot_id: token_payload.bot_id.clone(),
        ws_manager,
    };

    // Start WebSocket connection
    ws::start(actor, &req, stream)
}
```

### Testing WebSocket
Create `tests/ws_tests.rs`:
```rust
#[actix_web::test]
async fn test_websocket_connection() {
    // Create test client
    // Connect to WebSocket
    // Send message
    // Verify message received
}
```

---

## Phase 4: File Management Implementation

### Overview
Implement secure file upload, storage, and download with proper access control.

### Architecture
1. **File Upload**: Multipart form data handling
2. **File Storage**: Local filesystem or S3
3. **File Access**: Authorization check before download
4. **File Reference**: Validate file_id in messages

### Implementation Steps

#### Step 1: Update File Handler
Update `src/handlers/file.rs`:
```rust
pub async fn upload_file(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    http_req: HttpRequest,
    mut payload: web::Payload,
) -> AppResult<HttpResponse> {
    // Verify token
    let token = extract_token(&http_req)?;
    let token_payload = verify_token(token, ...)?;

    // Get user from bot_id
    let user_id = get_user_from_bot(&pool, &token_payload.bot_id).await?;

    // Create storage directory
    let file_dir = format!("./data/files/{}", user_id);
    fs::create_dir_all(&file_dir)?;

    // Generate file_id
    let file_id = generate_file_id();
    let storage_path = format!("{}/{}", file_dir, file_id);

    // Read and save file
    let mut file = fs::File::create(&storage_path)?;
    while let Some(chunk) = payload.next().await {
        let data = chunk?;
        file.write_all(&data)?;
    }

    // Get file size
    let size = file.metadata()?.len() as i64;

    // Store metadata in database
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
    // Verify token
    let token = extract_token(&http_req)?;
    let token_payload = verify_token(token, ...)?;

    // Get file and verify access
    let file = sqlx::query_as::<_, File>(
        "SELECT * FROM files WHERE file_id = ?"
    )
    .bind(file_id.as_str())
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(AppError::FileNotFound)?;

    // Verify user has access (owner or message recipient)
    verify_file_access(&pool, &token_payload.bot_id, &file).await?;

    // Stream file
    let file_data = fs::read(&file.storage_path)?;

    Ok(HttpResponse::Ok()
        .content_type(file.mime_type)
        .body(file_data))
}
```

#### Step 2: Add File Reference Validation
Update `src/handlers/message.rs`:
```rust
// When msg_type == 'file'
if msg_type == "file" {
    // Extract file_id from content
    let file_id = msg_req.content.get("file_id")
        .and_then(|v| v.as_str())
        .ok_or(AppError::BadRequest("Missing file_id".to_string()))?;

    // Verify file exists and belongs to sender
    let file_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM files WHERE file_id = ? AND owner_id = ?)"
    )
    .bind(file_id)
    .bind(&sender_user_id)
    .fetch_one(pool.get_ref())
    .await?;

    if !file_exists {
        return Err(AppError::BadRequest("Invalid file reference".to_string()));
    }
}
```

#### Step 3: Add Multipart Support
Update `Cargo.toml`:
```toml
actix-multipart = "0.4"
```

---

## Phase 5: Rate Limiting Implementation

### Overview
Implement distributed rate limiting using Redis with token bucket algorithm.

### Implementation Steps

#### Step 1: Redis Integration
Update handlers to check rate limits:
```rust
pub async fn send_message(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    redis: web::Data<redis::aio::ConnectionManager>,
    http_req: HttpRequest,
    msg_req: web::Json<CreateMessageRequest>,
) -> AppResult<HttpResponse> {
    // Verify token
    let token_payload = verify_token(...)?;

    // Check rate limit
    let rate_key = format!("rate_limit:{}:{}",
        token_payload.bot_id,
        Utc::now().timestamp()
    );

    let current_count: i32 = redis.get(&rate_key).await.unwrap_or(0);

    if current_count >= config.security.rate_limit_per_second as i32 {
        return Err(AppError::RateLimitExceeded);
    }

    // Increment counter with 1 second expiry
    redis.incr(&rate_key, 1).await?;
    redis.expire(&rate_key, 1).await?;

    // Continue with message sending...
}
```

---

## Testing Strategy

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
# Test auth flow
cargo test --test auth_tests

# Test message flow
cargo test --test message_tests

# Test file operations
cargo test --test file_tests

# Test WebSocket
cargo test --test ws_tests
```

### Performance Tests
```bash
# Load testing with concurrent connections
ab -n 10000 -c 100 http://localhost:8080/api/v1/message/send
```

---

## Development Checklist

### For WebSocket Phase
- [ ] Install actix-web-actors
- [ ] Create WsActor in `src/ws/actor.rs`
- [ ] Update `src/ws/manager.rs` with connection tracking
- [ ] Implement message broadcasting
- [ ] Update message handler to broadcast
- [ ] Create WebSocket tests
- [ ] Update documentation

### For File Phase
- [ ] Add multipart support to Cargo.toml
- [ ] Implement file upload handler
- [ ] Implement file download handler
- [ ] Add file reference validation
- [ ] Create file directory structure
- [ ] Add file cleanup on user deletion
- [ ] Create file tests

### For Rate Limiting Phase
- [ ] Connect Redis in main.rs
- [ ] Update handlers with rate limit checks
- [ ] Add Redis key expiration
- [ ] Create rate limit tests
- [ ] Monitor rate limit metrics

---

## Performance Optimization Tips

1. **Database Queries**
   - Use prepared statements (already using sqlx)
   - Add indexes on frequently queried columns
   - Use connection pooling (already configured)

2. **WebSocket**
   - Use binary frames for better performance
   - Implement message batching
   - Add backpressure handling

3. **File Operations**
   - Use streaming for large files
   - Implement chunked uploads
   - Add compression support

4. **Caching**
   - Cache bot information
   - Cache user permissions
   - Cache file metadata

---

## Security Considerations

1. **File Uploads**
   - Validate file size before storing
   - Scan for malware (optional)
   - Validate MIME type
   - Store outside web root

2. **Rate Limiting**
   - Per-user limits
   - Per-IP limits
   - Configurable thresholds

3. **Authentication**
   - Consider upgrading to bcrypt/argon2
   - Add rate limiting on login attempts
   - Consider 2FA for future

---

## Monitoring & Observability

1. **Logging**
   - Structure logs with tracing
   - Add request/response logging
   - Add error tracking

2. **Metrics**
   - Message throughput
   - Active connections
   - Error rates
   - Response times

3. **Health Checks**
   - Database connectivity
   - Redis connectivity
   - Disk space
   - Memory usage

---

## Deployment Considerations

1. **Docker**
   - Create Dockerfile for containerization
   - Set up docker-compose with SQLite/Redis
   - Configure volume mounts for data persistence

2. **Environment**
   - Separate .env files for dev/staging/prod
   - Secure secret management
   - Database backup strategy

3. **Monitoring**
   - Add structured logging (JSON logs)
   - Set up APM/tracing
   - Configure alerts

---

## References & Resources

- [Actix-web Documentation](https://actix.rs/)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [Tokio Documentation](https://tokio.rs/)
- [WebSocket RFC 6455](https://tools.ietf.org/html/rfc6455)
- [HMAC RFC 2104](https://tools.ietf.org/html/rfc2104)

---

**Document Created**: 2026-03-19
**Current Phase**: Phase 1 & 2 Complete, Phase 3 Ready to Start
