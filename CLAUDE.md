# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🚀 Quick Start Commands

### Build & Development
```bash
# Check code without building (fastest)
cargo check

# Build in debug mode (fast compilation, slow runtime)
cargo build

# Build in release mode (slow compilation, fast runtime)
cargo build --release

# Run the application
cargo run --release

# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Run tests
cargo test
```

### Database & Migration
```bash
# Database file is: chat_platform.db (SQLite)
# Migrations run automatically on startup
# Reset database: rm chat_platform.db && cargo run
```

### Testing
```bash
# Run comprehensive test script
python3 scripts/test_chat.py

# Start backend + UI
./start.sh          # Linux/macOS
./start.bat         # Windows
```

### Logging
```bash
# Show debug logs
RUST_LOG=debug cargo run --release

# Show logs for specific module
RUST_LOG=chat_platform::handlers=debug cargo run --release
```

## 🏗️ High-Level Architecture

### System Overview
This is a **group chat platform** with real-time messaging capabilities built in Rust using:
- **Framework**: Actix-web (async HTTP server with WebSocket support)
- **Database**: SQLite with SQLx (compile-time SQL verification)
- **Real-time**: Tokio async runtime + WebSocket connections

### Core Entities & Relationships

1. **User** (`user_id` = UUID)
   - Identified by 18-digit ID card number (身份证号) - must be unique
   - Can own multiple Bots and Groups
   - Real-world identity binding for authentication

2. **Bot** (`bot_id` = UUID with `b_` prefix)
   - API agent owned by a user
   - Has unique `token` and `secret` (for HMAC-SHA256 signing)
   - Status: `active` or `inactive`
   - Only Bots can send messages via API

3. **Group** (`group_id` = UUID with `g_` prefix)
   - Created by users to organize conversations
   - Contains multiple Bots as members
   - Messages are group-scoped

4. **Message**
   - Sent from Bot to Group
   - Types: `text` or `file`
   - Stored with sender_id, group_id, content, timestamp

### Request Flow
```
HTTP/WebSocket Client
         ↓
    Middleware (Token validation)
         ↓
    Handler Layer (auth, bot, group, message, file, ws)
         ↓
    Service Layer (business logic)
         ↓
    Database Layer (SQLx with compile-time verification)
         ↓
    SQLite Database
```

## 📂 Project Structure

```
src/
├── main.rs                    # Application entry point, HTTP server setup
├── lib.rs                     # Library root, module exports
├── config.rs                  # Configuration loading from .env
├── error.rs                   # Error types and handling
│
├── db/                        # Database layer
│   ├── mod.rs                 # Pool initialization & migration runner
│   ├── pool.rs                # Connection pool setup
│   ├── schema.rs              # SQL schema definitions
│   └── migrations.rs          # Database migration logic
│
├── models/                    # Data structures
│   ├── user.rs                # User model & operations
│   ├── bot.rs                 # Bot model & operations
│   ├── group.rs               # Group model & operations
│   ├── message.rs             # Message model & operations
│   ├── file.rs                # File model & operations
│   └── mod.rs                 # Module exports
│
├── handlers/                  # HTTP request handlers (Controller layer)
│   ├── auth.rs                # Register/Login endpoints
│   ├── user.rs                # User management endpoints
│   ├── bot.rs                 # Bot CRUD endpoints
│   ├── group.rs               # Group CRUD endpoints
│   ├── message.rs             # Message sending endpoints
│   ├── file.rs                # File upload/download endpoints
│   ├── ws.rs                  # WebSocket upgrade handler
│   └── mod.rs                 # Route setup & handler exports
│
├── services/                  # Business logic
│   ├── bot.rs                 # Bot service operations
│   ├── message.rs             # Message processing
│   ├── file.rs                # File handling
│   └── mod.rs                 # Service exports
│
├── middlewares/               # HTTP middlewares
│   ├── auth.rs                # Token validation middleware
│   └── mod.rs                 # Middleware setup
│
├── utils/                     # Helper functions
│   ├── id.rs                  # UUID generation with prefixes
│   ├── token.rs               # Token creation & HMAC verification
│   └── mod.rs                 # Utils exports
│
└── ws/                        # WebSocket management
    ├── manager.rs             # WebSocket connection manager
    └── mod.rs                 # WS exports
```

## 🔑 Key Technical Patterns

### Token System
- Format: `{bot_id}:{timestamp}:{signature}`
- Signature = HMAC-SHA256(bot_secret, `{bot_id}:{timestamp}`)
- Used for bot authentication in API calls

### ID Generation
- Users: `u_` + UUID (e.g., `u_550e8400-e29b-41d4-a716-446655440000`)
- Bots: `b_` + UUID
- Groups: `g_` + UUID
- ID prefixes enable type identification

### Database Access
- All queries use SQLx with compile-time verification (SELECT/INSERT/UPDATE/DELETE checked at compile time)
- Connection pool with configurable min/max connections
- Automatic migrations run on startup via `db::run_migrations()`

### Async Architecture
- Built on Tokio runtime for non-blocking I/O
- All database operations and HTTP handlers are `async`
- WebSocket connections managed by `WsManager`

## ⚙️ Configuration

The `.env` file controls:
- Server host/port and worker threads
- Database URL and pool settings
- JWT secret and token expiry
- File upload size limits
- Rate limiting (per-second)
- Rust logging level via `RUST_LOG`

## 🔒 Security Notes

1. **ID Card Storage**: Contains sensitive 身份证号 (18-digit ID numbers)
   - In production: encrypt at rest, use HTTPS for transmission, audit access

2. **Bot Token Secrets**: Each bot has a unique secret for HMAC signing
   - Keep secrets safe; deletion of bot invalidates token

3. **Rate Limiting**: Currently 10 requests/second per config
   - Can be made per-bot in future improvements

## 🔄 Common Development Tasks

### Adding a New API Endpoint
1. Define request/response models in `models/` (or in handler file if simple)
2. Add handler function in appropriate `handlers/*.rs`
3. Add route in `main.rs` to `App::new()`
4. If using database: create service in `services/` or inline in handler

### Adding Database Operations
1. Define model in `models/`
2. Use SQLx queries (compile-time checked)
3. If creating new table: add SQL to `db/schema.rs`
4. Migrations are auto-run; restart app to apply schema changes

### Working with WebSocket
- All connections tracked by `WsManager` in `src/ws/manager.rs`
- Upgrade endpoint: `/ws/{group_id}`
- Manager handles connection/disconnection/broadcast messages

## 📚 Important Files to Know

- **Cargo.toml**: Dependencies and project metadata
- **API_GUIDE.md**: Complete API endpoint documentation
- **BOT_INTEGRATION_GUIDE.md**: Integration instructions for bots
- **UI_STARTUP_GUIDE.md**: Frontend setup and usage
- **PROJECT_COMPLETE.md**: Project completion status and features
- **scripts/test_chat.py**: Comprehensive test script demonstrating all features

## 🧪 Testing Strategy

Run `python3 scripts/test_chat.py` to:
- Create two test users (Alice, Bob)
- Create multiple bots
- Create test groups
- Send messages between bots in groups
- Verify database persistence

## 🎯 Development Tips

1. **Type Safety**: Leverage Rust's type system; use strong types for IDs (not just strings)
2. **Compile-time Verification**: SQLx catches SQL errors at compile time—trust it
3. **Async First**: All I/O should be async; block only in tests
4. **Error Handling**: Use Result types; define custom error variants in `error.rs`
5. **Logging**: Use `tracing::info!()`, `debug!()`, `error!()` for observability
