# BoChat 测试与质量保障（PPT 讲稿版）

## 封面页
- 主题：BoChat 测试体系与可靠性实践
- 版本：2026-05-27
- 口径：仅使用可复现的项目证据，不虚构覆盖率或 CI 记录

## Q1｜到目前为止，你们的测试覆盖率是多少？
  - `chat_platform`（Rust 后端）：`75.66%`
  - `rust_sdk` ：`82.46%`
  - `python-sdk`：`79.56%`
- 使用的测试工具：
  - Rust 覆盖率：`llvm-cov`
  - Python 覆盖率：`coverage`

## Q2｜能举个你们单元测试的例子吗？
- 详细示例（Rust，`chat_platform/tests/token_tests.rs`）：
  - `test_token_generation_and_verification`：验证 token 生成后可正确解码 `bot_id`。
  - `test_token_verification_fails_with_wrong_secret`：错误密钥场景必须拒绝，避免伪造 token。
  - `test_token_verification_fails_with_invalid_format`：字段缺失/格式破坏必须拒绝。
  - `test_token_verification_fails_with_invalid_timestamp`：非法时间戳必须拒绝。
  - `test_token_verification_fails_when_expired`：过期 token 必须拒绝（包含边界时间处理）。
  - `test_user_token_generation_and_verification`：用户 token 主链路可验证。
  - `test_user_token_verification_fails_with_invalid_prefix`：用户 token 前缀约束可验证。
  - `test_user_token_verification_fails_with_wrong_secret`：用户 token 签名校验可验证。
- 这组用例证明的能力：
  - 正常路径可用。
  - 异常输入可控失败。
  - 认证链路具备可回归、可审计的安全边界。

## Q3｜你们如何进行集成测试？
- 策略：
  - 用 `chat_platform` 的真实路由和数据库/Redis 依赖进行端到端行为验证。
  - 当前已落地核心用例：
    - `chat_flow_integration`：注册、登录、建群、入群、发消息、查历史。
    - `file_reference_integration`：文件引用计数、群删除后的引用回收、头像引用重置。
- 汇报目标（扩展规模）：
  - 平台侧集成场景扩展到 **10+**（鉴权、消息、群管理、文件生命周期全链路）。
  - 社区协同场景补充 **4+**（社区插件消息推送/订阅回归）。
- 环境保障：
  - 端口统一 `50000+`（PG `50032`、Redis `50079`），规避本机常见端口冲突。
  - 测试数据唯一化（账号/群号/file_id 带时间戳），支持反复运行不脏库。
- 测试工具：
  - 执行器：`cargo test --test <integration_test_name>`
  - 依赖环境：`docker compose`（PostgreSQL + Redis）
  - 覆盖率汇总：`cargo llvm-cov`
- 当前状态：核心集成测试集已全绿（含 `chat_flow_integration`、`file_reference_integration`、`message_audit_integration`、`notification_review_integration`、`message_record_manager_integration`、`bot_handler_integration`、`repository_coverage_integration`）。

## Q4｜项目有没有可靠性指标？如何保证可靠性？
- 可靠性指标（汇报口径）：
  - 单元测试通过率：目标 **100%**（关键模块）。
  - 集成测试通过率：目标 **100%**（核心链路）。
  - 性能稳定性：目标 **99%**（连续压测成功率）。
- 当前落地基线：
  - 核心单测和关键集测已跑通并可复现。
- 可靠性保障手段：
  - 重试策略边界测试（429/5xx、非幂等方法不自动重试）。
  - token/user-token 认证边界测试。
  - 文件引用生命周期集成测试（防资源泄漏/脏引用）。
  - 环境隔离与端口规范化，降低“非代码因素”导致的误报失败。

## Q5｜需要与遗留系统交互吗？兼容性计划是什么？
- 结论：
  - 当前不需要与遗留系统交互。
- 兼容性计划：
  - HTTP 接口：保持向后兼容与版本演进策略。
  - WS 协议：保持事件结构稳定并兼容旧客户端处理逻辑。
  - SDK（Rust/Python）：保持一致语义与升级迁移说明。

## Q6｜你们是怎么做代码审查的？
- 流程（当前实践）：
  - 变更先补测试，再跑相关测试，再更新文档证据。
  - 审查重点：
    - 是否引入行为回归（尤其鉴权、重试、引用回收）。
    - 测试是否覆盖新增分支和边界。
    - 报告结论是否与测试输出一致。
- 审查标准：
  - “无证据不下结论”；每个“已验证”主张要能映射到具体命令和测试文件。

## Q7｜项目中有容错设计吗？
- 有，且已测试覆盖关键路径：
  - SDK 重试策略：网络异常、429、5xx 触发重试；4xx 与非幂等方法不盲目重试。
  - WS 处理：非法事件/非法 JSON 输入可安全忽略，不中断主流程。
  - 消息发送：重试分支显式可测，失败可追踪。

## Q8｜你们如何进行性能测试？
- 汇报目标（性能实验口径）：
  - 稳定性目标：**99%**
  - 吞吐目标：消息发送达到千级 msg/s
  - 延迟目标：核心 API 保持低延迟区间
- 试验环境（计划）：
  - 参考本机硬件环境进行压测（多核 CPU、充足内存、本地 Docker 依赖）。
  - 重点观察吞吐、P95 延迟、错误率、连接稳定性。
- 当前状态：
  - 已完成测试框架与指标定义，后续会沉淀为独立性能测试报告。

## Q9｜你们如何进行验收测试？
- 采用“场景化验收”：
  - 用户链路：注册/登录/建群/加群/发消息/查历史。
  - 文件链路：文件发送、引用建立、删除后回收。
- 验收标准：
  - 关键 API 状态码符合预期。
  - 关键副作用可验证（DB 引用计数、资源是否清理）。
  - 测试可重复执行且结果一致。

## Q10｜你们怎么准备测试文档？
- 这里的“测试文档”指独立文档体系，规划如下：
  - 单元测试文档：
    - 模块清单、核心用例、边界用例、失败样例与修复记录。
  - 集成测试文档：
    - 场景矩阵、依赖环境、执行脚本、通过标准、回归记录。
  - 覆盖率文档：
    - 总覆盖率、模块覆盖率、趋势变化、低覆盖风险点。
- 文档输出形式：
  - 一份详细版（工程使用）+ 一份答辩摘要版（汇报使用）。

## 收尾页｜一句话总结
- 我们不是“写了一份好看的报告”，而是把测试从“能讲”推进到“能跑、能复现、能证明”。
