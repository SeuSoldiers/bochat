# Docker Compose 部署说明

## 快速开始

### 1. 配置环境变量

复制 `.env.example` 文件为 `.env` 并修改配置：

```bash
cp .env.example .env
# 编辑 .env 文件修改配置
```

### 2. 启动服务

```bash
# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down
```

### 3. 访问服务

- 前端界面: http://localhost
- 后端 API: http://localhost:8080
- WebSocket: ws://localhost:8080/ws

## 开发环境

使用开发环境配置启动：

```bash
# 使用开发环境配置
docker-compose -f docker-compose.yml -f docker-compose.override.yml up -d

# 或者直接使用开发脚本
./scripts/docker-dev.sh
```

## 配置说明

### 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| POSTGRES_USER | PostgreSQL 用户名 | chat_user |
| POSTGRES_PASSWORD | PostgreSQL 密码 | chat_password |
| POSTGRES_DB | PostgreSQL 数据库名 | chat_db |
| REDIS_PASSWORD | Redis 密码 | redis_password |
| SERVER_HOST | 服务器监听地址 | 0.0.0.0 |
| SERVER_PORT | 服务器端口 | 8080 |
| JWT_SECRET | JWT 密钥 | your-super-secret-key-change-in-production |
| RUST_LOG | 日志级别 | info,chat_platform=debug |

### 服务端口

| 服务 | 端口 | 说明 |
|------|------|------|
| frontend | 80 | 前端界面 |
| backend | 8080 | 后端 API |
| postgres | 5432 | PostgreSQL 数据库 |
| redis | 6379 | Redis 缓存 |

## 数据持久化

- PostgreSQL 数据: `chat_postgres_data` 卷
- Redis 数据: `chat_redis_data` 卷
- 文件存储: `./assets/files/` 目录
- 日志文件: `./logs/` 目录

## 备份与恢复

### 备份数据库

```bash
docker-compose exec postgres pg_dump -U chat_user chat_db > backup.sql
```

### 恢复数据库

```bash
docker-compose exec -T postgres psql -U chat_user chat_db < backup.sql
```

## 故障排除

### 查看服务状态

```bash
docker-compose ps
```

### 查看服务日志

```bash
# 查看所有服务日志
docker-compose logs -f

# 查看特定服务日志
docker-compose logs -f backend
docker-compose logs -f frontend
```

### 重启服务

```bash
docker-compose restart
```

### 重新构建并启动

```bash
docker-compose up -d --build
```
