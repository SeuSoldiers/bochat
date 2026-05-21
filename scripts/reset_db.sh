#!/usr/bin/env bash
set -e

PROJECT_NAME="chat-server"

POSTGRES_VOLUME="${PROJECT_NAME}_chat_postgres_data"
REDIS_VOLUME="${PROJECT_NAME}_chat_redis_data"

echo "WARNING: This will delete all PostgreSQL and Redis data."
echo

read -p "Are you sure? Type 'yes' to continue: " CONFIRM

if [ "${CONFIRM}" != "yes" ]; then
  echo "Canceled."
  exit 0
fi

echo "Stopping services and removing containers..."
docker compose -p "${PROJECT_NAME}" down

echo "Removing data volumes..."
docker volume rm "${POSTGRES_VOLUME}" >/dev/null 2>&1 || true
docker volume rm "${REDIS_VOLUME}" >/dev/null 2>&1 || true

echo
echo "Database data has been deleted."
