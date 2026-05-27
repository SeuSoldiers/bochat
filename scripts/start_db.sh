#!/usr/bin/env bash
set -e

PROJECT_NAME="chat-server"

POSTGRES_CONTAINER="chat-postgres"
REDIS_CONTAINER="chat-redis"

POSTGRES_USER="chat_user"
POSTGRES_DB="chat_db"
REDIS_PASSWORD="redis_password"

echo "Starting services..."

docker compose -p "${PROJECT_NAME}" up -d postgres redis

echo "Waiting for PostgreSQL..."
until docker exec "${POSTGRES_CONTAINER}" pg_isready \
  -U "${POSTGRES_USER}" \
  -d "${POSTGRES_DB}" \
  -h 0.0.0.0 >/dev/null 2>&1; do
  sleep 1
done

echo "Waiting for Redis..."
until docker exec "${REDIS_CONTAINER}" redis-cli \
  -a "${REDIS_PASSWORD}" \
  -h 0.0.0.0 \
  ping >/dev/null 2>&1; do
  sleep 1
done

echo
echo "Services are ready."
echo
echo "PostgreSQL: 127.0.0.1:50032"
echo "Redis:      127.0.0.1:50079"
echo
echo "DATABASE_URL=postgresql://chat_user:chat_password@127.0.0.1:50032/chat_db"
echo "REDIS_URL=redis://:redis_password@127.0.0.1:50079/0"
