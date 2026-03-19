#!/bin/bash

# 群聊平台启动脚本
# 同时启动后端服务和 UI

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "════════════════════════════════════════════════════════════════"
echo "🚀 群聊平台 启动脚本"
echo "════════════════════════════════════════════════════════════════"
echo ""

# 检查是否已在运行
check_port() {
    if command -v lsof &> /dev/null; then
        lsof -Pi :$1 -sTCP:LISTEN -t >/dev/null 2>&1 && return 0 || return 1
    elif command -v netstat &> /dev/null; then
        netstat -tlnp 2>/dev/null | grep -q ":$1 " && return 0 || return 1
    fi
    return 1
}

# 启动后端服务
echo "1️⃣  启动后端服务 (端口 8080)..."
if check_port 8080; then
    echo "⚠️  端口 8080 已被占用，跳过后端启动"
else
    cd "$PROJECT_ROOT"
    cargo run --release &
    BACKEND_PID=$!
    echo "✅ 后端进程 ID: $BACKEND_PID"
    sleep 2
fi

echo ""

# 启动 UI 服务
echo "2️⃣  启动 UI 服务 (端口 3000)..."
if check_port 3000; then
    echo "⚠️  端口 3000 已被占用，跳过 UI 启动"
else
    cd "$PROJECT_ROOT/ui"
    if command -v node &> /dev/null; then
        echo "使用 Node.js 启动 UI..."
        node server.js &
        UI_PID=$!
    elif command -v python3 &> /dev/null; then
        echo "使用 Python 启动 UI..."
        python3 -m http.server 3000 &
        UI_PID=$!
    else
        echo "❌ 需要 Node.js 或 Python3"
        exit 1
    fi
    echo "✅ UI 进程 ID: $UI_PID"
    sleep 1
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✨ 启动完成！"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "📱 Web UI 访问地址: http://localhost:3000"
echo "🔌 后端 API 地址: http://localhost:8080"
echo "📡 WebSocket 地址: ws://localhost:8080/ws"
echo ""
echo "💡 使用说明:"
echo "  1. 打开浏览器访问 http://localhost:3000"
echo "  2. 注册新账户（身份证号需为18位）"
echo "  3. 登录获取 Token"
echo "  4. 创建群聊并邀请其他用户加入"
echo ""
echo "⌚ 运行测试脚本:"
echo "  python3 scripts/test_chat.py"
echo ""
echo "❌ 停止服务:"
echo "  按 Ctrl+C 停止所有服务"
echo ""

# 等待中断信号
trap "echo '👋 关闭服务...'; kill $BACKEND_PID $UI_PID 2>/dev/null; exit 0" SIGINT SIGTERM

wait
