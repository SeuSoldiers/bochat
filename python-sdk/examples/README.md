# Python SDK Example 运行指南

本指南适用于以下示例：

- `basic_flow.py`
- `ws_session.py`
- `ws_observe.py`
- `ws_dispatcher.py`
- `message_rate_ramp.py`

## 1. 前置条件

- Python `3.10+`
- BoChat 服务已启动，地址默认 `http://127.0.0.1:8080`
- 你有可登录的账号（用于示例登录）

## 2. 安装 SDK（开发模式）

在仓库根目录执行：

```bash
cd python-sdk
pip install -e .
```

如果需要运行 WebSocket 示例：

```bash
pip install -e ".[ws]"
```

## 3. 运行 `basic_flow.py`

```bash
cd /home/harkerhand/codes/bochat/python-sdk
python examples/basic_flow.py
```

该示例会执行：注册/登录、选 Bot、建群、发消息、上传文件，并在末尾清理创建的资源。

## 4. 运行 `ws_session.py`

先把示例里的账号密码改成你本地可用账号（文件：`examples/ws_session.py`），然后执行：

```bash
cd /home/harkerhand/codes/bochat/python-sdk
python examples/ws_session.py
```

该示例会执行：登录、选择 Bot token、连接 `/ws`、等待 `connection` 事件并打印群列表。

## 5. 运行 `ws_observe.py`（对齐 Rust ws_observe）

```bash
cd /home/harkerhand/codes/bochat/python-sdk
python examples/ws_observe.py <bot_token>
```

该示例会持续监听消息并打印，按 `Ctrl+C` 退出。

## 6. 运行 `ws_dispatcher.py`（对齐 Rust ws_session）

```bash
cd /home/harkerhand/codes/bochat/python-sdk
python examples/ws_dispatcher.py
```

该示例会创建 3 个群并注册 `default/group` 处理器，发送测试消息验证分发，再清理资源。
处理器注册使用装饰器风格：`@dispatcher.on_message()` 和 `@dispatcher.on_group_message(group_id)`。

## 7. 运行 WS 单元测试

```bash
cd /home/harkerhand/codes/bochat/python-sdk
python -m unittest discover -s tests -p "test_*.py"
```

## 8. 运行消息递增压测（10 msg/s 指数递增）

```bash
cd /home/harkerhand/codes/bochat/python-sdk
PYTHONPATH=. python examples/message_rate_ramp.py \
  --base-url http://127.0.0.1:50080 \
  --start-rps 10 \
  --max-rps 1000 \
  --growth 2 \
  --step-seconds 15
```

停止条件：
- 达到 `--max-rps`；
- 或失败率超过 `--fail-rate-threshold`（默认 5%）；
- 或 P95 超过 `--p95-ms-threshold`（默认 800ms）。

## 9. 常见问题

- `MissingUserToken`：登录未成功，检查账号密码是否正确。
- `MissingBotToken`：账号下没有可用 Bot，先在平台创建 Bot 或确认 Bot 状态为 `active`。
- 连接失败：确认服务地址是否为 `http://127.0.0.1:8080`，以及后端服务已启动。
