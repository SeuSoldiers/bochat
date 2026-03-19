# 🚀 快速开始指南

## 系统要求

- Rust 1.70+ (运行后端)
- Python 3.7+ 或 Node.js 14+ (运行 UI)
- SQLite (已包含在项目中)

## 一键启动

### Linux / macOS

```bash
./start.sh
```

### Windows

```bash
start.bat
```

或在 PowerShell 中：

```powershell
.\start.bat
```

## 手动启动

### 1️⃣ 启动后端服务

```bash
cargo run --release
```

输出应该显示：
```
🚀 群聊平台后端服务 启动中...
✅ 数据库连接池创建成功
✅ 数据库迁移完成
🌐 服务器将启动在 http://127.0.0.1:8080
```

### 2️⃣ 启动 UI 服务（在新终端中）

**使用 Node.js:**
```bash
cd ui
node server.js
```

**或使用 Python:**
```bash
cd ui
python3 -m http.server 3000
```

### 3️⃣ 打开浏览器

访问 http://localhost:3000

## 使用流程

### 1. 注册账户

- 切换到"注册"标签页
- 填写信息：
  - 姓名：任意
  - 身份证号：18位数字（例如：110101199003071234）
  - 手机号：任意
- 点击"注册"

### 2. 登录

- 在"登录"标签页输入身份证号和手机号
- 点击"登录"
- 获取 Token（自动保存到本地）

### 3. 创建群聊

- 点击侧边栏的 "+" 按钮
- 填写群名称
- 点击"创建"

### 4. 发送消息

- 在侧边栏选择一个群聊
- 在下方输入框输入消息
- 按 Enter 或点击"发送"

## 运行测试脚本

```bash
python3 scripts/test_chat.py
```

这会自动演示：
- ✅ 用户注册
- ✅ 用户登录
- ✅ 创建群聊
- ✅ Bot 加入群聊
- ✅ 发送消息
- ✅ 查看群成员

## 常见问题

### 问题：端口 8080 已被占用

**解决方案：**
```bash
# 查找占用 8080 的进程
lsof -i :8080

# 杀死进程
kill -9 <PID>
```

### 问题：端口 3000 已被占用

**解决方案：**
```bash
# 使用其他端口启动 UI
python3 -m http.server 8000
# 然后访问 http://localhost:8000
```

### 问题：无法连接到服务器

检查清单：
1. ✅ 后端服务是否在 8080 端口运行
2. ✅ UI 是否在 3000 端口运行
3. ✅ 防火墙是否阻止访问
4. ✅ 浏览器控制台是否有错误信息

### 问题：身份证号验证失败

- 必须是 18 位
- 可以是数字或最后一位为 X（大写）
- 例如：110101199003071234 ✅
- 例如：11010119900307123X ✅

## 架构

```
┌─────────────────────────────────────────────┐
│            Web 浏览器 (UI)                   │
│     http://localhost:3000                    │
└────────────────┬────────────────────────────┘
                 │ HTTP/WebSocket
                 ↓
┌─────────────────────────────────────────────┐
│      Rust 后端 (Actix-web)                   │
│     http://localhost:8080                    │
│  ┌─────────────────────────────┐            │
│  │      SQLite 数据库           │            │
│  │  chat_platform.db            │            │
│  └─────────────────────────────┘            │
└─────────────────────────────────────────────┘
```

## 项目结构

```
rust-bochat/
├── src/
│   ├── main.rs              # 应用入口
│   ├── handlers/            # 请求处理器
│   ├── models/              # 数据模型
│   ├── db/                  # 数据库操作
│   ├── utils/               # 工具函数
│   └── ws/                  # WebSocket 管理
├── ui/                      # Web UI
│   ├── index.html
│   ├── style.css
│   ├── app.js
│   └── server.js
├── scripts/
│   └── test_chat.py         # 测试脚本
├── Cargo.toml               # Rust 依赖
└── README.md                # 项目文档
```

## 有用的命令

### 编译

```bash
# 调试模式（快速编译，慢速运行）
cargo build

# 发布模式（慢速编译，快速运行）
cargo build --release
```

### 检查代码

```bash
# 检查代码正确性（不生成二进制文件）
cargo check

# 运行 clippy 检查代码风格
cargo clippy

# 格式化代码
cargo fmt
```

### 查看日志

后端日志级别可以通过环境变量控制：

```bash
# 显示 debug 日志
RUST_LOG=debug cargo run --release

# 只显示特定模块的日志
RUST_LOG=chat_platform::handlers=debug cargo run --release
```

## API 文档

详见 [API_GUIDE.md](API_GUIDE.md)

## UI 文档

详见 [ui/README.md](ui/README.md)

## 启动指南

详见 [UI_STARTUP_GUIDE.md](UI_STARTUP_GUIDE.md)

## 许可证

MIT

## 联系方式

有任何问题或建议，欢迎提出 Issue 或 Pull Request。
