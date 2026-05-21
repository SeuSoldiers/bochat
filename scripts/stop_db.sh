#!/usr/bin/env bash
set -e

PROJECT_NAME="chat-server"

echo "Stopping services..."

docker compose -p "${PROJECT_NAME}" down

echo "Services stopped."
