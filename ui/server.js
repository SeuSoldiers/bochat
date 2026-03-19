#!/usr/bin/env node

const http = require('http');
const fs = require('fs');
const path = require('path');
const url = require('url');

const PORT = 3000;
const MIME_TYPES = {
    '.html': 'text/html; charset=utf-8',
    '.js': 'text/javascript; charset=utf-8',
    '.css': 'text/css; charset=utf-8',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.jpeg': 'image/jpeg',
    '.gif': 'image/gif',
    '.svg': 'image/svg+xml',
    '.webp': 'image/webp',
};

const server = http.createServer((req, res) => {
    // 设置 CORS 头
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');

    if (req.method === 'OPTIONS') {
        res.writeHead(200);
        res.end();
        return;
    }

    // 解析 URL
    const parsedUrl = url.parse(req.url, true);
    let pathname = parsedUrl.pathname;

    // 移除多余的路径前缀
    if (pathname === '/' || pathname === '') {
        pathname = '/index.html';
    }

    // 获取文件路径
    let filePath = path.join(__dirname, pathname);

    // 防止目录遍历攻击
    try {
        const realPath = fs.realpathSync(__dirname);
        const resolvedPath = fs.realpathSync(filePath);
        if (!resolvedPath.startsWith(realPath)) {
            res.writeHead(403, { 'Content-Type': 'text/plain' });
            res.end('403 Forbidden');
            return;
        }
    } catch (err) {
        // 路径不存在，继续处理
    }

    // 检查文件是否存在
    fs.stat(filePath, (err, stats) => {
        if (err) {
            // 文件不存在，返回 404，但对于 SPA 应返回 index.html
            if (pathname !== '/index.html' && !pathname.includes('.')) {
                const indexPath = path.join(__dirname, 'index.html');
                fs.readFile(indexPath, (err, content) => {
                    if (err) {
                        res.writeHead(404, { 'Content-Type': 'text/plain' });
                        res.end('404 Not Found');
                        return;
                    }
                    res.writeHead(200, { 'Content-Type': MIME_TYPES['.html'] });
                    res.end(content);
                });
            } else {
                res.writeHead(404, { 'Content-Type': 'text/plain' });
                res.end('404 Not Found');
            }
            return;
        }

        if (stats.isDirectory()) {
            // 如果是目录，尝试读取 index.html
            const indexPath = path.join(filePath, 'index.html');
            fs.readFile(indexPath, (err, content) => {
                if (err) {
                    res.writeHead(404, { 'Content-Type': 'text/plain' });
                    res.end('404 Not Found');
                    return;
                }
                res.writeHead(200, { 'Content-Type': MIME_TYPES['.html'] });
                res.end(content);
            });
        } else if (stats.isFile()) {
            // 读取文件
            fs.readFile(filePath, (err, content) => {
                if (err) {
                    console.error(`读取文件失败 ${filePath}:`, err.message);
                    res.writeHead(500, { 'Content-Type': 'text/plain' });
                    res.end('500 Internal Server Error');
                    return;
                }

                const ext = path.extname(filePath);
                const contentType = MIME_TYPES[ext] || 'application/octet-stream';

                res.writeHead(200, { 'Content-Type': contentType });
                res.end(content);
            });
        } else {
            res.writeHead(404, { 'Content-Type': 'text/plain' });
            res.end('404 Not Found');
        }
    });
});

server.listen(PORT, '127.0.0.1', () => {
    console.log(`✨ 群聊平台 UI 已启动`);
    console.log(`📱 访问地址: http://localhost:${PORT}`);
    console.log(`🔌 后端 API: http://localhost:8080`);
    console.log(`\n按 Ctrl+C 停止服务器\n`);
});

server.on('error', (err) => {
    if (err.code === 'EADDRINUSE') {
        console.error(`❌ 端口 ${PORT} 已被占用，请使用其他端口或关闭占用该端口的应用`);
    } else {
        console.error('服务器错误:', err);
    }
    process.exit(1);
});
