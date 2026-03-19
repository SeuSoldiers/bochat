# Chat Platform Backend

A high-performance, asynchronous chat platform backend built with Rust, featuring:

- **Rust + Actix-web**: High-performance, async web framework
- **SQLite**: Lightweight, serverless database for persistence
- **WebSocket**: Real-time message delivery
- **Identity Decoupling**: Actor-based architecture with unified Bot identity
- **Token Authentication**: HMAC-SHA256 based token system
- **Rate Limiting**: Token bucket algorithm using Redis
- **File Management**: Secure file upload/download with ownership tracking

## Project Status

**Phase 1**: ✅ Project Initialization & Basic Framework
**Phase 2**: ✅ P0 - Identity Transformation Logic (Core)
**Phase 3**: 🔄 P1 - Unified Message Gateway (In Progress)
**Phase 4**: ⏳ P2 - File Association & Completion

## Architecture Overview

### Core Components

1. **Models** (`src/models/`)
   - `User`: User account with unique username/email
   - `Bot`: Identity container (personal or custom)
   - `Message`: Text/file messages with sender/recipient tracking
   - `File`: File metadata with ownership and access control

2. **Database** (`src/db/`)
   - SQLite schema with automatic migrations
   - Connection pooling with configurable limits
   - Indexed queries for optimal performance

3. **Services** (`src/services/`)
   - Business logic layer
   - Database queries and transformations
   - Service-specific operations

4. **Handlers** (`src/handlers/`)
   - HTTP request handlers
   - Token verification and authorization
   - Request/response serialization

5. **WebSocket** (`src/ws/`)
   - Connection manager for real-time delivery
   - Message broadcasting to online users
   - Connection lifecycle management

6. **Authentication** (`src/utils/token.rs`)
   - Token generation: `{bot_id}:{timestamp}:{signature}`
   - HMAC-SHA256 signature verification
   - Configurable token expiration

## API Endpoints

### Authentication
```
POST /api/v1/auth/register
  Request: { "username": "...", "email": "...", "password": "..." }
  Response: { "user_id": "...", "bot_id": "...", "token": "..." }

POST /api/v1/auth/login
  Request: { "username": "...", "password": "..." }
  Response: { "user_id": "...", "bot_id": "...", "token": "..." }
```

### Messages
```
POST /api/v1/message/send
  Header: Authorization: Bearer {token}
  Request: { "to_id": "...", "content": {...}, "msg_type": "text|file" }
  Response: { "msg_id": ..., "sender_id": "...", "created_at": "..." }
```

### Files
```
POST /api/v1/file/upload
  Header: Authorization: Bearer {token}
  Request: FormData (file)
  Response: { "file_id": "...", "url": "..." }

GET /api/v1/file/download/{file_id}
  Header: Authorization: Bearer {token}
  Response: File content
```

### WebSocket
```
WS /ws?token={token}
  Message: { "type": "message", "msg_id": ..., "sender_id": "...", "content": {...} }
```

## Development Setup

### Prerequisites
- Rust 1.70+ (Install from https://rustup.rs/)
- SQLite3 (usually bundled)
- Redis 6.0+ (optional, for advanced rate limiting)

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd rust-bochat
```

2. Create environment file:
```bash
cp .env.example .env
```

3. Update `.env` with your configuration:
```env
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DATABASE_URL=sqlite:chat_platform.db
JWT_SECRET=your-super-secret-key
```

4. Build and run:
```bash
cargo build --release
cargo run --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test token_tests

# Run with output
cargo test -- --nocapture
```

## Project Structure

```
chat-platform-rs/
├── Cargo.toml              # Dependencies and metadata
├── src/
│   ├── main.rs             # Application entry point
│   ├── lib.rs              # Library root
│   ├── config.rs           # Configuration management
│   ├── error.rs            # Error types and handling
│   ├── db/                 # Database layer
│   │   ├── mod.rs
│   │   ├── schema.rs       # Table definitions
│   │   └── pool.rs         # Connection pooling
│   ├── models/             # Data models
│   │   ├── user.rs
│   │   ├── bot.rs
│   │   ├── message.rs
│   │   └── file.rs
│   ├── handlers/           # HTTP handlers
│   │   ├── auth.rs
│   │   ├── message.rs
│   │   ├── file.rs
│   │   └── ws.rs
│   ├── services/           # Business logic
│   │   ├── bot.rs
│   │   ├── message.rs
│   │   └── file.rs
│   ├── utils/              # Utilities
│   │   ├── token.rs        # Token generation/verification
│   │   └── id.rs           # ID generation
│   ├── ws/                 # WebSocket
│   │   └── manager.rs
│   └── middlewares/        # HTTP middlewares
│       └── auth.rs
├── tests/                  # Integration tests
└── .env.example            # Example configuration
```

## Key Features

### P0: Identity Decoupling (✅ Implemented)

1. **User Registration**
   - Create user account
   - Automatically create personal Bot with type=`personal`
   - Issue initial access token

2. **User Login**
   - Verify credentials
   - Retrieve personal Bot
   - Generate new access token

3. **Bot Identity**
   - Each user has a personal Bot (ID: `u_*`)
   - Users can create custom Bots (ID: `b_*`)
   - All messages sent through Bot identity

### P1: Unified Message Gateway (🔄 In Progress)

1. **Message Sending**
   - Unified REST API for all message types
   - Bot-based authentication
   - Message type support: `text`, `file`
   - Persistent storage in SQLite

2. **Message Routing**
   - Recipient bot existence validation
   - Message format standardization
   - Indexed queries by recipient

### P2: File Management (⏳ Planned)

1. **File Upload**
   - Secure upload with auth
   - File size validation
   - Ownership tracking
   - MIME type detection

2. **File Access**
   - Download with authorization
   - Reference validation in messages
   - Cascade deletion on user deletion

## Configuration

### Environment Variables

```env
# Server
SERVER_HOST=127.0.0.1          # Bind address
SERVER_PORT=8080               # Bind port
SERVER_WORKERS=4               # Number of worker threads

# Database
DATABASE_URL=sqlite:chat_platform.db
DB_MAX_CONNECTIONS=10
DB_MIN_CONNECTIONS=2

# Redis (optional)
REDIS_URL=redis://127.0.0.1:6379
REDIS_POOL_SIZE=10

# Security
JWT_SECRET=your-secret-key     # Token signing key
TOKEN_EXPIRY_SECS=86400        # Token TTL (24h)

# Files
MAX_FILE_SIZE_MB=100

# Rate Limiting
RATE_LIMIT_PER_SECOND=10

# Logging
RUST_LOG=info,chat_platform=debug
```

## Database Schema

### users
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

### bots
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

### messages
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

### files
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

## Error Handling

All errors return structured JSON responses with status codes:

```json
{
  "error": "Error description",
  "status": 400
}
```

### HTTP Status Codes

- **200 OK**: Successful request
- **201 Created**: Resource created
- **400 Bad Request**: Invalid input
- **401 Unauthorized**: Invalid/missing credentials
- **403 Forbidden**: Access denied
- **404 Not Found**: Resource not found
- **409 Conflict**: Duplicate resource
- **429 Too Many Requests**: Rate limited
- **500 Internal Server Error**: Server error

## Performance Considerations

1. **Database**
   - SQLite with connection pooling
   - Indexed queries on frequently-accessed columns
   - Prepared statements for SQL injection prevention

2. **WebSocket**
   - Efficient message broadcasting to online connections
   - Automatic cleanup of closed connections
   - Per-bot connection management

3. **Rate Limiting**
   - Token bucket algorithm in Redis
   - Per-bot rate limits
   - Configurable limits

## Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
cargo test --test token_tests
```

### Token Tests
- Token generation and verification
- Signature validation
- Token format and structure

## Security

1. **Password Hashing**: SHA256 (upgrade to bcrypt/argon2 in production)
2. **Token Signing**: HMAC-SHA256 with configurable secret
3. **Authorization**: Token-based bearer authentication
4. **Input Validation**: All inputs validated before processing
5. **SQL Injection Prevention**: Parameterized queries

## Logging

Structured logging with `tracing`:
```rust
tracing::info!("User registered: {}", user_id);
tracing::warn!("Invalid login attempt: {}", username);
tracing::error!("Database error: {}", error);
```

Configure with `RUST_LOG` environment variable:
```bash
RUST_LOG=info,chat_platform=debug cargo run
```

## Future Enhancements

- [ ] WebSocket implementation with actix-web-actors
- [ ] Redis-based rate limiting
- [ ] File upload/download with streaming
- [ ] Message history pagination
- [ ] User presence/online status
- [ ] Message read receipts
- [ ] Group chat support
- [ ] Message encryption
- [ ] Audit logging
- [ ] API documentation with OpenAPI/Swagger

## Contributing

Please ensure all code:
- Compiles without warnings: `cargo check`
- Passes all tests: `cargo test`
- Is properly formatted: `cargo fmt`
- Passes linting: `cargo clippy`

## License

MIT License

## Support

For issues and questions, please open an issue on the repository.
