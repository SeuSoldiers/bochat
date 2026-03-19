@echo off
REM 群聊平台启动脚本 (Windows)

setlocal enabledelayedexpansion

echo ================================================================
echo 🚀 群聊平台 启动脚本 (Windows)
echo ================================================================
echo.

REM 启动后端服务
echo 1️⃣  启动后端服务 (端口 8080)...
echo 请在新的命令行窗口中执行以下命令:
echo   cargo run --release
echo.
echo 按任意键继续...
pause

REM 启动 UI 服务
echo.
echo 2️⃣  启动 UI 服务 (端口 3000)...
echo 请在新的命令行窗口中执行以下命令:

if exist "%SystemRoot%\System32\where.exe" (
    for /f %%i in ('where node 2^>nul') do (
        echo   cd ui && node server.js
        goto :start_ui
    )
)

echo   cd ui && python -m http.server 3000

:start_ui
echo.
echo ================================================================
echo ✨ 启动完成！
echo ================================================================
echo.
echo 📱 Web UI 访问地址: http://localhost:3000
echo 🔌 后端 API 地址: http://localhost:8080
echo 📡 WebSocket 地址: ws://localhost:8080/ws
echo.
echo 💡 使用说明:
echo   1. 打开浏览器访问 http://localhost:3000
echo   2. 注册新账户（身份证号需为18位）
echo   3. 登录获取 Token
echo   4. 创建群聊并邀请其他用户加入
echo.
echo ⚙️  运行测试脚本:
echo   python scripts\test_chat.py
echo.

pause
