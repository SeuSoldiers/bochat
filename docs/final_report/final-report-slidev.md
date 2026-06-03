---
theme: default
title: BoChat 项目总结汇报
info: 项目背景、需求分析、总体设计与功能演示占位
class: text-left
highlighter: shiki
lineNumbers: false
drawings:
  persist: false
transition: fade
mdc: true
css: unocss
---

<style>
:root {
  --bochat-blue: #163b65;
  --bochat-slate: #44556c;
  --bochat-border: #cfd8e3;
  --bochat-soft: #eef3f8;
}

.slidev-layout {
  font-family: "Noto Serif SC", "Source Han Serif SC", serif;
}

h1, h2, h3 {
  color: var(--bochat-blue);
}

.section-tag {
  display: inline-block;
  padding: 0.2rem 0.6rem;
  border: 1px solid var(--bochat-border);
  color: var(--bochat-blue);
  font-size: 0.85rem;
  letter-spacing: 0.04em;
}

.summary-box {
  border: 1px solid var(--bochat-border);
  background: linear-gradient(180deg, #f8fbfe 0%, #eef3f8 100%);
  padding: 1rem 1.1rem;
  border-radius: 8px;
}

.video-box {
  min-height: 320px;
  border: 2px dashed #8ea3bb;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: repeating-linear-gradient(
    -45deg,
    #f7f9fc,
    #f7f9fc 16px,
    #eef3f8 16px,
    #eef3f8 32px
  );
  color: var(--bochat-slate);
  font-size: 1.3rem;
  font-weight: 600;
}

.tiny {
  font-size: 0.9rem;
  color: var(--bochat-slate);
}

table {
  font-size: 0.92rem;
}
</style>

# BoChat 项目总结汇报

<div class="section-tag">FINAL REPORT</div>

<div class="mt-8 summary-box">

- 项目名称：`BoChat`
- 汇报人：`第三组`
- 汇报内容：`项目背景及需求分析`、`总体设计及架构概览`、`功能演示及介绍`
- 项目定位：`面向 Bot 协作场景的群聊平台`

</div>


---

# 目录

1. 项目背景及需求分析
2. 项目总体设计及架构概览
3. 项目功能演示及介绍

---

# 一、项目背景及需求分析

<div class="section-tag">PART ONE</div>

---

# 1. 项目背景

<div class="section-tag">BACKGROUND</div>

<div class="mt-6 grid grid-cols-2 gap-6">
<div class="summary-box">

### 项目定位

BoChat 是一个面向 `Bot 协作场景` 的群聊平台。  
其核心模型是：

- 一个用户可管理多个 Bot
- Bot 作为群成员参与消息收发
- Bot 可承载插件与自动化协作

</div>
<div class="summary-box">

### 场景背景

- 传统聊天工具以“人”为中心，不适合多 Bot 协作
- 传统聊天工具中心化设计，难以满足隐私与安全需求
- 开发者需要稳定接口快速构建业务 Bot

</div>
</div>

---

# 2. 项目目标

<div class="section-tag">OBJECTIVES</div>

<div class="mt-6 summary-box">

### 总体目标

构建一套围绕 Bot 协作的群聊平台，统一 `聊天能力`、`平台能力` 与 `开发能力`。

</div>

<div class="grid grid-cols-2 gap-6 mt-6">
<div class="summary-box">

### 业务目标

- 支持用户管理多个 Bot
- 支持 Bot 参与群聊协作
- 支持消息、文件与插件的统一流转

</div>
<div class="summary-box">

### 工程目标

- 提供稳定的 HTTP API 与 WebSocket 实时能力
- 建立清晰的分层架构与模块边界
- 提供 Rust SDK 与 Python SDK

</div>
</div>

---

# 3. 用户需求分析

<div class="section-tag">USER NEEDS</div>

| 用户角色 | 主要需求 |
| --- | --- |
| 账号管理员 | 管理 Bot、插件、群聊和成员 |
| Bot 使用者 | 完成实时聊天并使用插件 |
| Bot 开发者 | 基于 SDK 快速开发 Bot |
| 平台维护者 | 保证安全、稳定、可维护 |

<div class="mt-5 summary-box">

### 需求归纳

- 管理员关注 `多 Bot 管理与协作组织`
- 使用者关注 `实时交互体验与扩展能力`
- 开发者关注 `接入效率与接口一致性`

</div>

---

# 4. 功能需求分析

<div class="section-tag">FUNCTIONAL</div>

| 需求类别 | 具体需求 |
| --- | --- |
| 账号与 Bot | 注册登录、Bot 创建与管理 |
| 群聊与消息 | 建群、成员管理、消息发送与历史查询 |
| 实时与文件 | WebSocket 推送、文件上传下载与去重 |
| 平台扩展 | 社区插件、Rust SDK、Python SDK |

<div class="mt-4 tiny">
功能需求对应项目真实链路：注册、建 Bot、建群、发消息、拉历史、传文件、接收事件、触发插件。
</div>

---

# 5. 非功能需求分析

<div class="section-tag">NON-FUNCTIONAL</div>

<div class="grid grid-cols-2 gap-6 mt-6">
<div class="summary-box">

### 可靠性

- 接口参数校验与权限校验
- 消息发送幂等控制
- 核心链路具备测试覆盖

</div>
<div class="summary-box">

### 性能与实时性

- 支持 WebSocket 实时推送
- 使用 Redis 支撑热消息访问
- 支持并发消息处理

</div>
</div>

<div class="grid grid-cols-2 gap-6 mt-4">
<div class="summary-box">

### 安全性

- User Token / Bot Token 双认证体系
- 文件上传后异步安全扫描
- 审计日志记录关键操作

</div>
<div class="summary-box">

### 可维护性与可扩展性

- 后端、Rust SDK、Python SDK 语义对齐
- 明确的分层架构与模块边界
- 支持社区插件扩展

</div>
</div>

---

# 二、项目总体设计及架构概览

<div class="section-tag">PART TWO</div>

---

# 1. 总体设计思路

<div class="section-tag">DESIGN IDEA</div>

<div class="mt-6 summary-box">

### 设计思路

1. 以 `Bot 协作` 为系统核心，而不是传统的人类单点聊天
2. 以前端上 `管理员端 + Bot 用户端` 双入口承载两类典型使用场景
3. 以 `WebSocket 实时通信` 为消息链路基础
4. 采用 `分层架构` 保证系统可维护
5. 通过 `SDK + 插件` 降低接入与扩展成本

</div>

---

# 2. 系统架构设计

<div class="section-tag">ARCHITECTURE</div>

<div class="mt-6 summary-box">

### 架构分层

`接入层`
管理员前端 / Bot 用户前端 / SDK / 社区插件

`业务层`
Middleware / Handlers / Services

`数据层`
PostgreSQL / Redis / File Storage

</div>

<div class="tiny mt-4">
前端设计上，`/login` 对应管理员账号入口，`/bot/login` 和 `/bot/chat` 对应 Bot Token 用户入口，两类界面共享同一套后端能力，但面向不同角色分工。
</div>

---

# 3. 技术选型

<div class="section-tag">TECH STACK</div>

| 技术层 | 选型 | 选型原因 |
| --- | --- | --- |
| 后端语言 | `Rust` | 强类型、内存安全、适合高并发服务 |
| 前端框架 | `Vue 3 + Vite + TypeScript` | 适合快速构建双前端入口与组件化界面 |
| 前端状态与路由 | `Pinia + Vue Router` | 便于区分管理员端与 Bot 端状态流转 |
| Web 框架 | `Axum` | 适合异步 HTTP 与 WebSocket |
| 数据层 | `PostgreSQL + Redis` | 持久化与热数据访问分离 |
| 开发接口 | `Rust SDK + Python SDK` | 兼顾工程化与快速开发 |

---

# 4. 功能模块划分

<div class="section-tag">MODULES</div>

| 模块 | 主要职责 |
| --- | --- |
| `Admin Web` | 管理员登录、Bot 管理、群聊管理、通知与审计入口 |
| `Bot Web` | Bot Token 登录、群聊工作台、实时消息与文件交互 |
| `Auth / Middleware` | 用户与 Bot 双 Token 认证、权限隔离 |
| `Bot / Group` | Bot 生命周期管理、群聊与成员管理 |
| `Message Handler` | 消息发送、历史查询、幂等控制、缓存与索引 |
| `File / Notification / Audit` | 文件流转、通知处理、审计记录 |
| `Rust SDK / Python SDK` | 对外统一开发接口 |

---

# 5. 系统流程设计

<div class="section-tag">FLOW</div>

<div class="mt-6 summary-box">

### 核心流程

1. 用户或开发者通过前端或 SDK 发起请求  
2. 服务端完成认证、鉴权和业务处理  
3. 数据写入 PostgreSQL，并同步更新 Redis 缓存  
4. 结果通过 WebSocket 推送给在线 Bot 或客户端

</div>

<div class="grid grid-cols-2 gap-6 mt-6">
<div class="summary-box">

### 主链路说明

- 管理员端与 Bot 端分别从各自入口发起请求
- 中间件负责身份校验和权限隔离
- 服务层完成业务处理与数据持久化
- 结果通过 WebSocket 实时分发

</div>
<div class="summary-box">

### 流程设计价值

- 保证消息链路的实时性
- 让管理员端与 Bot 用户端职责分离、界面清晰
- 将持久化与热数据访问职责分离
- 支撑插件、SDK 与群聊场景统一接入

</div>
</div>

---

# 三、项目功能演示及介绍

<div class="section-tag">DEMO VIDEO</div>

<div class="mt-8 video-box">
  演示视频占位符
</div>

<div class="mt-6 tiny">

- 此处后续插入项目功能演示视频成品
- 视频内容建议对应三个视角：`账号管理员`、`Bot 使用者`、`Bot 开发者`

</div>
