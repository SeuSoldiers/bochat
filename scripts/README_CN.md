# 聊天平台 API 测试脚本 - 中文说明

这个目录包含用于测试聊天平台 API 的 Python 脚本。

## 📁 文件说明

### `test_chat_api.py`
完整的 API 测试脚本，演示如何：
- ✅ 注册新用户
- ✅ 用户登录
- ✅ 用户之间发送消息
- ✅ 测试 API 的完整流程

## 🔧 前置需求

### 1. Python 3.7+
检查 Python 版本：
```bash
python3 --version
```

### 2. 安装依赖
```bash
# 安装 requests 库
pip3 install requests
```

或使用 pip：
```bash
pip install requests
```

### 3. 启动聊天平台服务器
在另一个终端运行：
```bash
cd /home/harkerhand/codes/rust-bochat
cargo run
```

等待看到类似的输出：
```
Server starting on 127.0.0.1:8080
Database migrations completed
```

## 🚀 使用方法

### 运行完整的 API 测试

**方式 1：使用 python3**
```bash
python3 scripts/test_chat_api.py
```

**方式 2：直接运行（需要添加可执行权限）**
```bash
chmod +x scripts/test_chat_api.py
./scripts/test_chat_api.py
```

## 📝 脚本做了什么

脚本会自动执行以下步骤：

### 🏥 第 1 步：服务器健康检查
- 检查 API 服务器是否运行正常
- 如果服务器未运行，会提示需要启动

### 📝 第 2 步：注册两个用户
```
用户 1: alice
  - 邮箱: alice@example.com
  - 密码: alice123

用户 2: bob
  - 邮箱: bob@example.com
  - 密码: bob123
```

### 💬 第 3 步：相互发送消息
```
Alice → Bob: "你好，Bob！这是我的第一条消息。"
Alice → Bob: "你在吗？"
Bob → Alice: "你好，Alice！我在这里。"
Bob → Alice: "很高兴认识你！"
... 更多消息交流
```

### 📊 第 4 步：输出测试结果
显示：
- 用户信息（ID、邮箱、Bot ID）
- 消息统计
- 测试是否通过

## 💻 实际运行示例

```bash
$ python3 scripts/test_chat_api.py

🚀 聊天平台 API 测试脚本
============================================================

🏥 检查服务器状态...
✅ 服务器正常运行 (响应: OK)

############################################################
# 阶段 1: 注册两个用户
############################################################

============================================================
📝 注册用户: alice
============================================================

📤 POST /auth/register
   请求数据: {
  "username": "alice",
  "email": "alice@example.com",
  "password": "alice123"
}
✅ 响应 (201): {
  "user_id": "u_550e8400-e29b-41d4-a716-446655440000",
  "username": "alice",
  "email": "alice@example.com",
  "bot_id": "b_660e8400-e29b-41d4-a716-446655440001",
  "token": "b_660e8400...:1711000000:abc123def456...",
  "created_at": "2026-03-19T08:00:00Z"
}
✨ 用户 alice 注册成功!
   用户ID: u_550e8400-e29b-41d4-a716-446655440000
   机器人ID: b_660e8400-e29b-41d4-a716-446655440001
   Token: b_660e8400...:1711000000:abc123...

... (类似输出继续)

============================================================
💬 alice → bob: 你好，Bob！这是我的第一条消息。
============================================================

📤 POST /message/send
   请求数据: {
  "to_id": "b_...",
  "content": {
    "text": "你好，Bob！这是我的第一条消息。"
  },
  "msg_type": "text"
}
✅ 响应 (201): {
  "msg_id": 1,
  "sender_id": "b_...",
  "to_id": "b_...",
  "content": {
    "text": "你好，Bob！这是我的第一条消息。"
  },
  "msg_type": "text",
  "created_at": "2026-03-19T08:00:05Z"
}
✨ 消息发送成功!
   消息ID: 1
   发送时间: 2026-03-19T08:00:05Z

... (更多消息发送)

✨ 测试完成！
============================================================

📊 测试统计:
   注册用户数: 2
   发送消息数: 6
   总消息往返: 6

用户信息:

   用户: alice
   - 用户ID: u_550e8400-e29b-41d4-a716-446655440000
   - 机器人ID: b_660e8400-e29b-41d4-a716-446655440001
   - 邮箱: alice@example.com

   用户: bob
   - 用户ID: u_770e8400-e29b-41d4-a716-446655440002
   - 机器人ID: b_880e8400-e29b-41d4-a716-446655440003
   - 邮箱: bob@example.com

✅ 所有测试都通过了！
```

## ⚠️ 常见问题

### 1️⃣ 错误：连接错误: 无法连接到 127.0.0.1:8080

**原因**: API 服务器未运行

**解决方案**:
```bash
# 在新的终端中启动服务器
cd /home/harkerhand/codes/rust-bochat
cargo run
```

### 2️⃣ 错误：ModuleNotFoundError: No module named 'requests'

**原因**: 未安装 requests 库

**解决方案**:
```bash
# 使用 pip3 安装
pip3 install requests

# 或使用 pip
pip install requests
```

### 3️⃣ 错误：HTTP 错误: 500

**原因**: 服务器内部错误，可能是数据库问题

**解决方案**:
```bash
# 删除旧的数据库文件
rm chat_platform.db

# 重新启动服务器（会自动创建新数据库）
cargo run
```

### 4️⃣ 错误：Permission denied

**原因**: Python 脚本没有可执行权限

**解决方案**:
```bash
# 添加可执行权限
chmod +x scripts/test_chat_api.py

# 然后运行
./scripts/test_chat_api.py
```

## 🎨 自定义脚本

### 修改用户信息

编辑 `test_chat_api.py` 中的 `main()` 函数：

```python
# 修改用户名和邮箱
alice_info = tester.register_user(
    username="your_username",
    email="your_email@example.com",
    password="your_password"
)

bob_info = tester.register_user(
    username="another_name",
    email="another_email@example.com",
    password="another_password"
)
```

### 修改消息内容

```python
# 修改发送的消息
tester.send_message(
    from_user="alice",
    to_user="bob",
    message="你的自定义消息"
)
```

### 添加更多用户

```python
# 注册更多用户
charlie_info = tester.register_user(
    username="charlie",
    email="charlie@example.com",
    password="charlie123"
)

# 发送消息
tester.send_message("alice", "charlie", "你好，Charlie！")
```

## 📊 测试流程图

```
┌──────────────────────────┐
│   启动测试脚本            │
└────────┬─────────────────┘
         │
         ▼
┌──────────────────────────┐
│  检查服务器是否运行        │
└────────┬─────────────────┘
         │
         ├─ ✅ 运行中 ──┐
         │              │
         └─ ❌ 未运行   └──▶ 提示启动
         │
         ▼
┌──────────────────────────┐
│   阶段 1: 注册两个用户    │
│   - 注册 Alice            │
│   - 注册 Bob              │
└────────┬─────────────────┘
         │
         ▼
┌──────────────────────────┐
│  阶段 2: 相互发送消息     │
│  - Alice → Bob (2条)      │
│  - Bob → Alice (2条)      │
│  - 继续交流 (4条)         │
└────────┬─────────────────┘
         │
         ▼
┌──────────────────────────┐
│   显示测试结果            │
│  - 用户信息               │
│  - 消息统计               │
│  - 测试状态               │
└──────────────────────────┘
```

## 🔗 API 端点参考

脚本使用以下 API 端点：

| 方法 | 端点 | 说明 |
|------|------|------|
| GET | `/health` | 健康检查 |
| POST | `/api/v1/auth/register` | 用户注册 |
| POST | `/api/v1/auth/login` | 用户登录 |
| POST | `/api/v1/message/send` | 发送消息 |

详细信息见项目的 `README_CN.md`

## 🚀 扩展脚本

### 压力测试

```python
# 发送大量消息
for i in range(100):
    tester.send_message("alice", "bob", f"测试消息 #{i}")
```

### 多用户测试

```python
# 注册多个用户
users = ["alice", "bob", "charlie", "david", "eve"]
for user in users:
    tester.register_user(user, f"{user}@example.com", f"{user}123")

# 相互发送消息
for i in range(len(users)):
    for j in range(len(users)):
        if i != j:
            tester.send_message(users[i], users[j], f"来自 {users[i]} 的消息")
```

### 登录测试

```python
# 测试登录功能
tester.login_user("alice", "alice123")
tester.login_user("bob", "bob123")
```

## 📖 相关文档

- `README.md` - 英文说明
- `README_CN.md` - 中文项目说明
- `QUICKSTART_CN.md` - 快速开始指南
- `API 文档` - 见项目主目录

## 📝 许可

MIT License

---

**需要帮助？** 查看项目的 `README_CN.md` 获取完整的 API 文档和使用说明。
