# 群聊平台 UI 启动指南

本文档说明如何启动 UI 与后端 API 进行通信。

## 前置要求

### 后端服务
- Rust 群聊平台后端已编译并运行在 `http://localhost:8080`
- 数据库已初始化

### UI 服务
以下三种方式任选其一：

## 方法 1: Python 内置服务器（推荐用于开发）

这是最简单的方法，不需要额外安装。

```bash
cd ui
python3 -m http.server 3000
```

输出应该如下：
```
Serving HTTP on 0.0.0.0 port 3000 (http://0.0.0.0:3000/) ...
```

然后在浏览器中打开：`http://localhost:3000`

## 方法 2: Node.js 服务器

如果已安装 Node.js，可以使用内置的 server.js：

```bash
cd ui
node server.js
```

输出应该如下：
```
✨ 群聊平台 UI 已启动
📱 访问地址: http://localhost:3000
🔌 后端 API: http://localhost:8080

按 Ctrl+C 停止服务器
```

然后在浏览器中打开：`http://localhost:3000`

## 方法 3: VS Code Live Server

如果使用 VS Code：

1. 安装 "Live Server" 扩展（by Ritwick Dey）
2. 右键点击 `ui/index.html`
3. 选择 "Open with Live Server"
4. 浏览器会自动打开，通常在 `http://127.0.0.1:5500`

## 完整启动步骤

### 第一步：启动后端服务

```bash
cd /path/to/rust-bochat
cargo run --release
```

等待输出：
```
Server starting on 127.0.0.1:8080
```

### 第二步：启动 UI 服务

在新的终端窗口中：

```bash
cd /path/to/rust-bochat/ui
python3 -m http.server 3000
```

或使用其他方法之一。

### 第三步：打开浏览器

访问 `http://localhost:3000`

## 验证连接

1. **打开浏览器开发者工具** (F12 或 Cmd+Option+I)
2. **切换到 Console 标签**
3. **执行以下 JavaScript 代码**验证与后端的连接：

```javascript
fetch('http://localhost:8080/health')
    .then(r => r.text())
    .then(d => console.log('后端连接成功:', d))
    .catch(e => console.error('后端连接失败:', e))
```

如果输出显示 "后端连接成功: OK"，则说明连接正常。

## 解决 CORS 问题

如果在浏览器中看到 CORS 错误：

```
Access to XMLHttpRequest at 'http://localhost:8080/...'
from origin 'http://localhost:3000' has been blocked by CORS policy
```

这说明后端没有启用 CORS。需要在后端代码中添加 CORS 中间件。

在 `src/main.rs` 中添加：

```rust
use actix_cors::Cors;

// 在 HttpServer::new 中添加：
Cors::default()
    .allow_any_origin()
    .allow_any_method()
    .allow_any_header()
```

然后在 `Cargo.toml` 中添加依赖：

```toml
actix-cors = "0.7"
```

## 调试技巧

### 查看 API 请求

在浏览器开发者工具中：
1. 切换到 **Network** 标签
2. 执行操作（如注册、发送消息）
3. 查看请求和响应

### 查看 WebSocket 消息

在浏览器开发者工具中：
1. 切换到 **Network** 标签
2. 筛选 **WS** 协议
3. 点击 WebSocket 连接
4. 在 **Messages** 标签中查看发送和接收的消息

### 查看本地存储

在浏览器开发者工具中：
1. 切换到 **Application** 标签
2. 点击 **Local Storage**
3. 可以看到保存的 token 和用户信息

## 开发工作流

### 修改 UI

只需编辑以下文件并刷新浏览器：
- `ui/index.html` - 页面结构
- `ui/style.css` - 样式
- `ui/app.js` - 应用逻辑

大多数文件保存后刷新浏览器即可看到效果。

### 修改后端

修改后端代码后：
1. 停止后端服务 (Ctrl+C)
2. 运行 `cargo run --release`
3. 刷新浏览器

### 使用浏览器代理

如果需要拦截和修改请求，可以使用：
- **Chrome DevTools** (内置)
- **Charles** (跨平台)
- **Fiddler** (Windows)
- **Burp Suite** (安全测试)

## 性能优化建议

### 生产环境

1. **使用 gzip 压缩**：大多数 HTTP 服务器都支持
2. **缓存静态资源**：设置合理的 Cache-Control headers
3. **CDN**：将静态文件放到 CDN
4. **最小化 CSS/JS**：使用压缩工具
5. **lazy loading**：按需加载图片和模块

### 后端 API

1. **数据库索引**：确保查询字段有索引
2. **连接池**：使用数据库连接池
3. **缓存**：缓存频繁访问的数据
4. **分页**：限制一次返回的数据量

## 故障排除

### 问题：连接拒绝

```
ERR_CONNECTION_REFUSED at http://localhost:8080/...
```

**解决方案**：
1. 检查后端是否启动: `curl http://localhost:8080/health`
2. 检查端口是否正确
3. 检查防火墙设置

### 问题：无法加载页面

如果页面无法加载：
1. 检查 UI 服务器是否启动
2. 检查端口是否正确
3. 查看浏览器控制台错误

### 问题：登录后无法显示群聊

1. 检查 token 是否正确保存
2. 查看浏览器控制台是否有错误
3. 检查 Network 标签中的 API 响应

### 问题：消息无法发送

1. 确保已选择一个群聊
2. 确保已加入该群聊
3. 检查 token 是否过期
4. 查看 Network 标签中的错误响应

## 测试场景

### 场景 1：基本注册和登录

1. 打开 UI 首页
2. 切换到"注册"标签
3. 填写信息：
   - 姓名：张三
   - 身份证号：12345678901234567X (18位)
   - 手机号：13800138000
4. 点击注册
5. 应该自动进入聊天页面

### 场景 2：创建和加入群聊

1. 点击侧边栏的 "+" 按钮
2. 填写群名称："测试群"
3. 点击创建
4. 群聊应该出现在列表中

### 场景 3：发送消息

1. 选择一个群聊
2. 在输入框输入消息
3. 按 Enter 或点击发送
4. 消息应该立即显示

## 常见配置

### 修改 API 基地址

如果后端运行在不同的 URL，编辑 `ui/app.js`：

```javascript
const API_BASE_URL = 'http://your-api-host:8080/api/v1';
```

### 修改 UI 端口

**Python 方式**：
```bash
python3 -m http.server 8000  # 改为 8000
```

**Node.js 方式**：
编辑 `ui/server.js`，将 `const PORT = 3000;` 改为需要的端口。

## 联系和支持

如有问题，检查以下内容：

1. 查看浏览器控制台错误
2. 查看后端日志
3. 检查网络连接
4. 验证 token 和认证信息

更多信息请参考项目根目录的 README.md
