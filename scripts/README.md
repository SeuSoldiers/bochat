
如果要临时监听 WebSocket 推送，可以使用：

- [ws_monitor.py](/home/harkerhand/codes/rust-bochat/scripts/ws_monitor.py)

说明：

- 传入的是单个 `bot_token`
- 握手时会自动加 `Authorization: Bearer <bot_token>`
- 脚本监听的是“这个 Bot 当前所在群”的消息推送
- 不是监听这个用户名下所有 Bot

运行前安装依赖：

```bash
pip install websockets
```

示例：

```bash
python3 scripts/ws_monitor.py '<bot_token>'
python3 scripts/ws_monitor.py '<bot_token>' --url http://127.0.0.1:8080/ws
```
