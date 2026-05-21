#!/bin/bash

# Docker 开发环境启动脚本

set -e

echo "启动 Docker 开发环境..."

# 检查 Docker 是否运行
if ! docker info > /dev/null 2>&1; then
    echo "错误: Docker 未运行，请先启动 Docker"
    exit 1
fi

# 检查 docker-compose 是否安装
if ! command -v docker-compose &> /dev/null; then
    echo "错误: docker-compose 未安装"
    exit 1
fi

# 停止现有容器
echo "停止现有容器..."
docker-compose down

# 构建并启动服务
echo "构建并启动服务..."
docker-compose -f docker-compose.yml -f docker-compose.override.yml up -d --build

# 等待服务启动
echo "等待服务启动..."
sleep 10

# 检查服务状态
echo "检查服务状态..."
docker-compose ps

echo ""
echo "开发环境已启动!"
echo "前端界面: http://localhost:48080"
echo "后端 API: http://localhost:48888"
echo "数据库: localhost:5432"
echo "Redis: localhost:6379"
echo ""
echo "查看日志: docker-compose logs -f"
echo "停止服务: docker-compose down"
