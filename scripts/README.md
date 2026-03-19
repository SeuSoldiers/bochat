# 聊天平台 API 测试脚本

这个目录包含用于测试聊天平台 API 的 Python 脚本。

## 📁 文件说明

### `test_chat.py`
完整的 API 测试脚本，演示如何：
- ✅ 使用身份证号注册新用户（实名认证）
- ✅ 创建和管理多个Bot
- ✅ 通过Bot发送消息
- ✅ 测试完整的API流程

## 🔧 前置需求

### 1. Python 3.7+
检查 Python 版本：
```bash
python3 --version
```

### 2. 使用uv管理虚拟环境

安装uv（如果还未安装）：
```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

在scripts目录中创建虚拟环境：
```bash
cd scripts
uv venv
source .venv/bin/activate  # Linux/Mac
# 或
.venv\Scripts\activate     # Windows
```

### 3. 安装依赖
```bash
uv pip install requests
```

### 4. 启动聊天平台服务器
在另一个终端运行：
```bash
cd /home/harkerhand/codes/rust-bochat
cargo run --release
```

等待看到类似的输出：
```
Server starting on 127.0.0.1:8080
Database migrations completed successfully
```

## 🚀 使用方法

### 运行API测试

**方式 1：使用 python3**
```bash
python3 test_chat.py
```

**方式 2：直接运行（需要添加可执行权限）**
```bash
chmod +x test_chat.py
./test_chat.py
```

**方式 3：使用 uv 运行**
```bash
uv run test_chat.py
```

## 📝 脚本做了什么

脚本会自动执行以下步骤：

### 🏥 第 1 步：服务器健康检查
- 检查 API 服务器是否运行正常
- 如果服务器未运行，会提示需要启动

### 📝 第 2 步：注册两个用户
```
用户 1: Alice
  - 身份证号: 110101199003071234
  - 手机号: 13800138001

用户 2: Bob
  - 身份证号: 110101199003071235
  - 手机号: 13800138002
```

### 🤖 第 3 步：创建多个Bot
- Alice 创建一个客服Bot
- Bob 创建一个支持Bot

### 💬 第 4 步：相互发送消息
```
Alice的默认Bot → Bob的默认Bot: "你好Bob！这是来自Alice的默认Bot的消息。"
Bob的默认Bot → Alice的默认Bot: "你好Alice！这是来自Bob的默认Bot的消息。"
Alice的客服Bot → Bob的默认Bot: "这是来自Alice的客服Bot的消息"
Bob的支持Bot → Alice的默认Bot: "这是来自Bob的支持Bot的消息"
```

### 📊 第 5 步：输出测试结果
显示：
- 已注册的用户信息（ID、身份证号、手机号）
- Bot列表（ID、名称、所有者、状态）
- 测试是否全部通过

## 💻 实际运行示例

```bash
$ python3 test_chat.py

============================================================
🚀 聊天平台测试 - 新架构
============================================================

🔍 检查服务器状态...
✅ 服务器运行中！

============================================================
场景1: 使用身份证号进行用户注册认证
============================================================

📝 正在注册用户: Alice
   身份证号: 110101199003071234
✅ 用户注册成功！
   用户ID: u_550e8400-e29b-41d4-a716-446655440000
   默认Bot ID: b_550e8400-e29b-41d4-a716-446655440001

...

============================================================
📊 测试总结
============================================================

✅ 已注册的用户:
  • Alice
    - 用户ID: u_550e8400-e29b-41d4-a716-446655440000
    - 身份证号: 110101199003071234
  • Bob
    - 用户ID: u_550e8400-e29b-41d4-a716-446655440002
    - 身份证号: 110101199003071235

✅ Bot列表:
  • Alice的默认Bot (b_550e8400-e29b-41d4-a716-446655440001)
    - 所有者: Alice
    - 状态: active
  • Bob的默认Bot (b_550e8400-e29b-41d4-a716-446655440003)
    - 所有者: Bob
    - 状态: active

============================================================
✨ 所有测试完成成功！
============================================================
```

## 🔍 常见问题

### Q: 运行脚本时出现 "ConnectionError" 错误
**A:** 这表示无法连接到服务器。请确保：
1. 服务器已启动：`cargo run --release`
2. 服务器运行在 `127.0.0.1:8080`
3. 没有其他应用占用 8080 端口

### Q: 运行脚本时出现 "409 Conflict" 错误
**A:** 这表示身份证号已被注册。原因是：
1. 之前的测试还没有清理数据库
2. 解决办法：删除 `chat.db` 文件并重启服务器

```bash
cd /home/harkerhand/codes/rust-bochat
rm -f chat.db
cargo run --release
```

### Q: 如何修改测试中使用的用户数据？
**A:** 编辑 `test_chat.py` 中的 `main()` 函数，找到以下代码并修改：

```python
user_a_success = tester.register_user(
    name="Alice",  # 修改用户名
    id_number="110101199003071234",  # 修改身份证号（18位）
    phone="13800138001"  # 修改手机号
)
```

### Q: 如何只运行部分测试？
**A:** 可以注释掉 `main()` 函数中不需要的部分。例如，只测试注册功能：

```python
def main():
    tester = ChatPlatformTester()

    if not tester.health_check():
        sys.exit(1)

    # 只运行注册测试
    tester.register_user("Alice", "110101199003071234", "13800138001")
    tester.register_user("Bob", "110101199003071235", "13800138002")

    # 注释掉其他测试
    # tester.list_user_bots("Alice")
    # ...
```

## 📚 更多信息

- 查看 [API_GUIDE.md](../API_GUIDE.md) 了解完整的API文档
- 查看 [ARCHITECTURE_REFORM.md](../ARCHITECTURE_REFORM.md) 了解系统架构改革说明
- 查看 [README.md](../README.md) 了解项目概述

## 🐛 报告问题

如果遇到问题，请提供：
1. Python版本：`python3 --version`
2. 错误信息的完整输出
3. 运行的命令
4. 服务器日志（如果有）

祝您测试愉快！
