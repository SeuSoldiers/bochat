#!/bin/bash
# UI-Vue 项目启动脚本

echo "================================"
echo "BoChat UI-Vue 项目启动"
echo "================================"
echo ""

# 检查依赖
if [ ! -d "node_modules" ]; then
  echo "📦 安装项目依赖..."
  npm install
fi

# 启动开发服务器
echo "🚀 启动开发服务器..."
npm run dev
