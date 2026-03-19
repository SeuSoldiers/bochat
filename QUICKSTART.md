# Quick Start Guide

## Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Git
- A text editor or IDE

## Installation & Setup

### 1. Clone the Repository
```bash
cd /home/harkerhand/codes/rust-bochat
```

### 2. Setup Environment
```bash
# Copy example environment file
cp .env.example .env

# Optional: Update .env with your settings
# Default values work for local development
```

### 3. Build the Project
```bash
# Check compilation (fast)
cargo check

# Build debug binary
cargo build

# Build release binary (optimized)
cargo build --release
```

### 4. Run Tests
```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test token_tests

# Run with output
cargo test -- --nocapture
```

### 5. Start the Server
```bash
# Run in development mode
cargo run

# Run in release mode
cargo run --release

# With custom logging
RUST_LOG=debug cargo run
```

Server will start on `http://127.0.0.1:8080`

## Quick Testing

### Test User Registration
```bash
curl -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "email": "alice@example.com",
    "password": "password123"
  }'
```

Expected response:
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

### Test User Login
```bash
curl -X POST http://127.0.0.1:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "password": "password123"
  }'
```

### Test Message Sending
```bash
# First, register two users and get their tokens
# Then send a message from user1 to user2

TOKEN="b_...:...:..."  # Token from registration

curl -X POST http://127.0.0.1:8080/api/v1/message/send \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "to_id": "b_...",
    "content": {"text": "Hello, World!"},
    "msg_type": "text"
  }'
```

### Test Health Check
```bash
curl http://127.0.0.1:8080/health
# Expected response: "OK"
```

## Directory Structure

```
chat-platform-rs/
├── Cargo.toml              # Project manifest
├── Cargo.lock              # Locked dependencies
├── .env.example            # Example environment variables
├── .gitignore              # Git ignore rules
│
├── src/
│   ├── main.rs             # Server entry point
│   ├── lib.rs              # Library root
│   ├── config.rs           # Configuration
│   ├── error.rs            # Error types
│   │
│   ├── db/                 # Database layer
│   │   ├── mod.rs
│   │   ├── schema.rs       # Table creation
│   │   ├── pool.rs         # Connection pooling
│   │   └── migrations.rs   # Migration helpers
│   │
│   ├── models/             # Data models
│   │   ├── mod.rs
│   │   ├── user.rs         # User model
│   │   ├── bot.rs          # Bot model
│   │   ├── message.rs      # Message model
│   │   └── file.rs         # File model
│   │
│   ├── handlers/           # HTTP handlers
│   │   ├── mod.rs
│   │   ├── auth.rs         # Login/Register
│   │   ├── message.rs      # Message sending
│   │   ├── file.rs         # File upload/download
│   │   └── ws.rs           # WebSocket
│   │
│   ├── services/           # Business logic
│   │   ├── mod.rs
│   │   ├── bot.rs
│   │   ├── message.rs
│   │   └── file.rs
│   │
│   ├── utils/              # Utilities
│   │   ├── mod.rs
│   │   ├── token.rs        # Token generation/verification
│   │   └── id.rs           # ID generation
│   │
│   ├── ws/                 # WebSocket
│   │   ├── mod.rs
│   │   └── manager.rs      # Connection manager
│   │
│   └── middlewares/        # HTTP middlewares
│       ├── mod.rs
│       └── auth.rs         # Auth middleware
│
├── tests/                  # Integration tests
│   └── token_tests.rs      # Token testing
│
├── README.md               # Project documentation
├── IMPLEMENTATION_SUMMARY.md  # What's implemented
└── DEVELOPMENT_GUIDE.md    # Next phases guide
```

## Development Tips

### Code Formatting
```bash
# Format code with rustfmt
cargo fmt

# Check formatting without changing
cargo fmt -- --check
```

### Linting
```bash
# Run clippy for suggestions
cargo clippy
```

### Build Times

- First build: ~30-40 seconds (with dependency compilation)
- Incremental builds: ~1-5 seconds
- Release build: ~60+ seconds (optimized)

Use `cargo check` during development (faster than full build).

## Database

### Automatic Schema Creation
The database schema is automatically created on first run:
- `users` table for user accounts
- `bots` table for bot identities
- `messages` table for message storage
- `files` table for file metadata

### Inspecting Database
```bash
# Install SQLite if not present
# macOS: brew install sqlite3
# Ubuntu: sudo apt-get install sqlite3

# Open database
sqlite3 chat_platform.db

# List tables
.tables

# View users
SELECT * FROM users;

# View bots
SELECT * FROM bots;

# View messages
SELECT * FROM messages;
```

## Common Issues

### Build Fails
- Ensure Rust is installed: `rustc --version`
- Update Rust: `rustup update`
- Clear cache: `cargo clean && cargo build`

### Server Won't Start
- Check if port 8080 is in use: `lsof -i :8080`
- Change port in `.env`: `SERVER_PORT=8081`
- Check database permissions

### Database Issues
- Delete old database: `rm chat_platform.db`
- Server will recreate on startup
- Check file permissions in directory

## Configuration Files

### `.env` (Local Development)
```env
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DATABASE_URL=sqlite:chat_platform.db
JWT_SECRET=your-secret-key
RUST_LOG=info
```

### `.env.example` (Template)
Shows all available configuration options

## Next Steps

1. **Read Documentation**
   - `README.md` - Project overview
   - `IMPLEMENTATION_SUMMARY.md` - What's implemented
   - `DEVELOPMENT_GUIDE.md` - Next phases

2. **Explore Code**
   - Start with `src/main.rs`
   - Look at `src/handlers/auth.rs` for example handler
   - Check `src/models/` for data structures

3. **Make Changes**
   - Create new branch: `git checkout -b feature/my-feature`
   - Make changes and test: `cargo test`
   - Commit when working: `git add . && git commit -m "message"`

4. **Next Phase**
   - WebSocket implementation (see DEVELOPMENT_GUIDE.md)
   - File upload/download
   - Rate limiting with Redis

## Useful Commands

```bash
# Check code without building
cargo check

# Build debug version
cargo build

# Build optimized version
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Lint code
cargo clippy

# Generate documentation
cargo doc --open

# Clean build cache
cargo clean

# Update dependencies
cargo update

# Check for security vulnerabilities
cargo audit
```

## Documentation Links

- [Actix-web Docs](https://docs.rs/actix-web/)
- [SQLx Docs](https://docs.rs/sqlx/)
- [Tokio Docs](https://docs.rs/tokio/)
- [Serde Docs](https://docs.rs/serde/)
- [Tracing Docs](https://docs.rs/tracing/)

## Support & Questions

If you encounter issues:
1. Check the README.md for detailed information
2. Review IMPLEMENTATION_SUMMARY.md for current status
3. Check DEVELOPMENT_GUIDE.md for implementation details
4. Examine test files for example usage

---

**Happy coding!** 🦀
