
如果要临时监听 WebSocket 推送，可以使用：

- [ws_monitor.py](/home/harkerhand/codes/rust-bochat/scripts/ws_monitor.py)

运行前安装依赖：

```bash
pip install websockets
```

示例：

```bash
python3 scripts/ws_monitor.py '<bot_token>'
python3 scripts/ws_monitor.py '<bot_token>' --url http://127.0.0.1:8080/ws
```
