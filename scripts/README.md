# 聊天平台 API 测试脚本

这个目录包含用于测试聊天平台 API 的 Python 脚本。

## 文件说明

### `test_chat_api.py`
完整的 API 测试脚本，演示如何：
- 注册新用户
- 用户登录
- 用户之间发送消息

## 前置需求

### 1. Python 3.7+
检查 Python 版本：
```bash
python3 --version
```

### 2. 安装依赖
```bash
# 安装 requests 库
pip install requests
```

或使用 pip3：
```bash
pip3 install requests
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

## 使用方法

### 运行完整的 API 测试
```bash
python3 scripts/test_chat_api.py
```

或直接运行（需要添加可执行权限）：
```bash
chmod +x scripts/test_chat_api.py
./scripts/test_chat_api.py
```

## 脚本做了什么

脚本会执行以下步骤：

### 1️⃣ 服务器健康检查
检查 API 服务器是否运行正常

### 2️⃣ 注册两个用户
- 用户 1: alice (alice@example.com)
- 用户 2: bob (bob@example.com)

### 3️⃣ 相互发送消息
- Alice 发送 2 条消息给 Bob
- Bob 回复 2 条消息给 Alice
- 继续相互交流更多消息

### 4️⃣ 输出测试结果
显示用户信息和消息统计

## 预期输出

脚本会输出类似以下内容：

```
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
  "user_id": "u_xxx",
  "username": "alice",
  ...
}
✨ 用户 alice 注册成功!
   用户ID: u_xxx
   机器人ID: b_xxx
   Token: xxx...

...

✨ 测试完成！
============================================================

📊 测试统计:
   注册用户数: 2
   发送消息数: 6
   总消息往返: 6

✅ 所有测试都通过了！
```

## 常见问题

### 1. "连接错误: 无法连接到 127.0.0.1:8080"

**原因**: API 服务器未运行

**解决方案**:
```bash
# 在新的终端中启动服务器
cd /home/harkerhand/codes/rust-bochat
cargo run
```

### 2. "ModuleNotFoundError: No module named 'requests'"

**原因**: 未安装 requests 库

**解决方案**:
```bash
pip3 install requests
```

### 3. "HTTP 错误: 500"

**原因**: 服务器内部错误，可能是数据库问题

**解决方案**:
```bash
# 删除数据库文件
rm chat_platform.db

# 重新启动服务器（会自动创建新数据库）
cargo run
```

## 自定义脚本

如果想修改用户名或消息内容，编辑 `test_chat_api.py` 中的 `main()` 函数：

```python
# 修改这些行
alice_info = tester.register_user(
    username="your_username",
    email="your_email@example.com",
    password="your_password"
)

# 修改消息内容
tester.send_message(
    from_user="alice",
    to_user="bob",
    message="你的自定义消息"
)
```

## 测试流程图

```
┌─────────────────────────────────┐
│     启动测试脚本                │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   检查服务器健康状态            │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   注册两个用户                   │
│   - alice                        │
│   - bob                          │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   用户相互发送消息               │
│   - Alice → Bob (2条)            │
│   - Bob → Alice (2条)            │
│   - 继续交流 (4条)               │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   显示测试结果                   │
│   - 用户信息                     │
│   - 消息统计                     │
└─────────────────────────────────┘
```

## API 端点参考

脚本使用以下 API 端点：

- `GET /health` - 健康检查
- `POST /api/v1/auth/register` - 用户注册
- `POST /api/v1/auth/login` - 用户登录
- `POST /api/v1/message/send` - 发送消息

详细信息见项目的 `README_CN.md`

## 扩展脚本

可以基于这个脚本创建更多测试用例：

```python
# 例如：压力测试
for i in range(100):
    tester.send_message("alice", "bob", f"消息 #{i}")

# 或者：多用户测试
users = ["alice", "bob", "charlie", "david"]
for user in users:
    tester.register_user(user, f"{user}@example.com", f"{user}123")
```

## 许可

MIT License

---

**提示**: 如有问题，请查看服务器日志输出或检查 `README_CN.md` 获取更多帮助。
