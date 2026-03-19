# 群聊平台 Web UI

这是为 Rust 群聊平台后端创建的 Web 用户界面。

## 功能特性

- 👤 **用户注册和登录** - 使用实名认证（身份证号）
- 💬 **群聊管理** - 创建、加入、管理群聊
- 💌 **实时消息** - 基于 WebSocket 的实时消息推送
- 👥 **成员管理** - 查看群成员列表
- 🎨 **现代化 UI** - 响应式设计，支持移动设备

## 项目结构

```
ui/
├── index.html       # HTML 页面
├── style.css        # 样式表
├── app.js          # 主应用逻辑
├── README.md       # 此文件
└── server.js       # 简单 HTTP 服务器（可选）
```

## 快速开始

### 方法 1: 使用 Python 内置服务器

```bash
cd ui
python3 -m http.server 3000
```

然后在浏览器中打开 `http://localhost:3000`

### 方法 2: 使用 Node.js HTTP 服务器

```bash
cd ui
node server.js
```

然后在浏览器中打开 `http://localhost:3000`

### 方法 3: 使用 Live Server (VS Code 扩展)

如果你使用 VS Code，可以安装 Live Server 扩展，然后右击 `index.html` 选择 "Open with Live Server"。

## 配置

在 `app.js` 中修改 API 基础 URL：

```javascript
const API_BASE_URL = 'http://localhost:8080/api/v1';
```

确保与你的 Rust 后端地址一致。

## 使用流程

### 1. 注册
- 点击"注册"标签页
- 填写真实姓名、18位身份证号和手机号
- 点击"注册"按钮
- 系统自动为新用户创建默认 Bot

### 2. 登录
- 点击"登录"标签页
- 目前暂不支持直接登录（需要后端实现登录接口）
- 建议注册新账户或使用已有账户的注册信息

### 3. 创建群聊
- 在侧边栏点击 "+" 按钮
- 填写群名称和描述
- 点击"创建"按钮

### 4. 加入群聊
- 点击右上角"加入群聊"按钮
- 选择想要加入的群聊
- 点击"加入"按钮

### 5. 发送消息
- 在侧边栏选择一个群聊
- 在下方输入框输入消息
- 按 Enter 或点击"发送"按钮

### 6. 查看成员
- 点击右上角"成员"按钮
- 查看群聊的所有成员列表

## API 端点

应用使用以下后端 API 端点：

| 方法 | 端点 | 描述 |
|------|------|------|
| POST | `/api/v1/auth/register` | 用户注册 |
| POST | `/api/v1/groups` | 创建群聊 |
| GET | `/api/v1/groups` | 获取用户群聊列表 |
| POST | `/api/v1/groups/{group_id}/join` | 加入群聊 |
| GET | `/api/v1/groups/{group_id}/members` | 获取群成员列表 |
| POST | `/api/v1/message/send` | 发送消息 |
| WS | `/ws?token={token}` | WebSocket 消息推送 |

## 浏览器兼容性

- Chrome 60+
- Firefox 55+
- Safari 11+
- Edge 79+

## 已知限制

1. **登录功能** - 当前版本建议使用注册功能，后续可添加登录接口
2. **消息历史** - 首次选择群聊时消息列表为空，新消息通过 WebSocket 接收
3. **文件上传** - 当前版本仅支持文本消息，不支持文件上传
4. **用户头像** - 用户无法自定义头像

## 开发

### 添加新功能

1. 修改 HTML 结构（`index.html`）
2. 更新样式（`style.css`）
3. 添加事件处理和 API 调用（`app.js`）

### 调试

在浏览器开发者工具中查看：
- Console 标签：查看 JavaScript 错误和日志
- Network 标签：查看 API 请求和响应
- Application 标签：查看本地存储的 token 和用户信息

## 常见问题

### Q: CORS 错误
A: 确保 Rust 后端启用了 CORS 支持。可以在 Actix-web 中添加 CORS 中间件。

### Q: WebSocket 连接失败
A: 检查：
1. 后端是否启动了 WebSocket 处理程序
2. WebSocket URL 是否正确
3. Token 是否有效

### Q: 消息无法发送
A: 检查：
1. 是否已登录
2. 是否已加入群聊
3. 网络连接是否正常
4. 后端是否返回错误信息（查看浏览器控制台）

## 许可证

MIT

## 联系方式

如有任何问题或建议，欢迎提出 Issue 或 Pull Request。
