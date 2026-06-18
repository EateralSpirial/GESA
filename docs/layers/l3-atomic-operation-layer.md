# L3 Atomic Operation Layer 原子操作层

> Version: 0.1  
> Status: Layer Design  
> Position: Runtime Plane / Action Capability

---

## 1. 定位

原子操作层是 GESA 的动作能力层，负责把所有不可分割的动作注册、校验、执行、审计和结果回写。

它回答的问题是：

```text
系统到底能做哪些动作？
每个动作由哪个引擎执行？
动作需要哪些参数？
动作执行前需要什么权限？
动作执行后如何记录结果？
动作失败如何重试、回滚和反馈？
```

原子操作层不负责业务目标，也不负责任务拆解。复杂业务由工作空间层和工作流模块编排。

---

## 2. 核心职责

```text
1. 定义系统所有不可分割操作。
2. 将外部工具、浏览器、文件、邮件、GitHub、日历、数据库查询等能力封装成 Operation。
3. 执行前请求 L2 权限判断。
4. 执行中进行参数校验、风险检查、沙箱隔离和超时控制。
5. 执行后写入 operation_logs、events、audit_logs。
6. 将失败结果提交给反馈层或工作空间层。
7. 为 Agent 和 Workflow 提供统一工具接口。
```

---

## 3. 三分法

```text
Operation Definition   操作定义：系统允许做什么。
Operation Execution    操作执行：具体引擎如何完成动作。
Operation Audit        操作审计：谁对谁做了什么，结果如何。
```

这三部分必须分离，避免工具调用与权限、审计、工作流耦合。

---

## 4. 操作类别

| Category | 中文 | 示例 |
|---|---|---|
| Read | 读取 | 查询客户、读取文件、检索知识 |
| Write | 写入 | 修改状态、创建任务、更新文档 |
| Execute | 执行 | 运行脚本、调用工具、浏览器动作 |
| Communicate | 通信 | 发邮件、发消息、客服回复 |
| Schedule | 调度 | 创建日程、排班、设置提醒 |
| Approve | 审批 | 批准付款、确认合同、发布知识 |
| Delegate | 委托 | 转交任务、分配责任人 |
| Configure | 配置 | 修改规则、调整权限、配置模块 |
| Evolve | 进化 | 创建改进候选、生成补丁、提交 PR |

---

## 5. Operation Definition Schema

```text
operation_id
operation_name
operation_category
description
input_schema
output_schema
required_permission
risk_level
approval_required
audit_required
idempotent
retry_policy
timeout_seconds
execution_engine
agent_callable
workflow_callable
ui_callable
created_at
updated_at
```

---

## 6. 执行链路

```text
调用方请求操作
  ↓
读取 Operation Definition
  ↓
参数 schema 校验
  ↓
请求 L2 权限判断
  ↓
判断是否需要审批
  ↓
选择执行引擎
  ↓
执行前记录 operation_started
  ↓
执行动作
  ↓
写入 operation_result
  ↓
写入 event / audit log
  ↓
通知 Workspace / Feedback
```

---

## 7. 操作引擎

```text
Browser Use Engine
- Playwright
- Selenium
- browser-use

Database Query Engine
- 受限 SQL 查询
- 预定义查询模板
- 只读分析查询

Knowledge Engine
- search_knowledge
- create_knowledge_candidate
- approve_knowledge_update

File Operation Engine
- 读取文件
- 写入文件
- 转换文件
- 生成报告

Document Engine
- Tika
- Unstructured
- Pandoc
- Docling

Email / Message Engine
- 邮件
- Mattermost
- Rocket.Chat
- Chatwoot

Calendar Engine
- CalDAV
- Cal.com
- Google Calendar connector

GitHub / Repo Engine
- Issue
- PR
- Commit
- Release

Deployment Engine
- Docker Compose
- Ansible
- Argo CD
```

---

## 8. 幂等、重试与回滚

每个操作需要声明：

```text
是否幂等
是否可重试
重试次数
重试间隔
是否可回滚
回滚操作
失败是否进入反馈层
失败是否阻塞工作流
```

示例：

| 操作 | 幂等 | 可重试 | 可回滚 | 说明 |
|---|---|---|---|---|
| 读取客户 | 是 | 是 | 不需要 | 安全读操作 |
| 创建任务 | 可设计为幂等 | 是 | 可软删除 | 需要 idempotency_key |
| 发送邮件 | 否 | 谨慎 | 不可回滚 | 需要人工确认或去重 |
| 修改权限 | 否 | 否 | 可恢复旧策略 | 高风险审计 |
| 部署服务 | 部分 | 是 | 需要 rollback plan | 由 M1 执行 |

---

## 9. 沙箱与风险控制

```text
低风险：直接执行，记录日志。
中风险：权限检查 + 执行日志。
高风险：权限检查 + 审批 + 沙箱 + 审计。
极高风险：禁止 Agent 自动执行，只能人工确认。
```

高风险操作包括：

```text
付款
权限变更
生产部署
删除数据
导出客户数据
提交代码
发布正式知识
修改自进化规则
```

---

## 10. Agent 工具接口

Agent 不应该直接调用底层工具。Agent 只调用注册后的 Operation。

```text
Agent Tool Call
  ↓
Operation Gateway
  ↓
Permission Decision
  ↓
Execution Engine
  ↓
Audit / Event / Feedback
```

MCP、OpenAPI、FastAPI schema 都可以作为工具描述格式。

---

## 11. 核心 Schema

```text
operation_definitions
operation_engines
operation_permissions
operation_requests
operation_logs
operation_results
operation_retries
operation_approvals
operation_events
operation_risk_rules
```

---

## 12. 开源方案

| 功能 | 轻量级 | MVP | 企业级 |
|---|---|---|---|
| 浏览器操作 | Playwright | Playwright + browser-use | Playwright Grid / RPA 平台 |
| API 封装 | FastAPI | FastAPI + OpenAPI + MCP | API Gateway + Registry |
| 任务队列 | RQ / APScheduler | Celery / Dramatiq | Temporal / Kafka |
| 文档操作 | Pandoc / MarkItDown | Tika / Unstructured | Tika + Unstructured + Docling |
| 沙箱 | Docker | Docker + gVisor | Firecracker / gVisor |
| 审计 | SQLite logs | OpenTelemetry + DB logs | OpenTelemetry + Loki + SIEM |
| 低代码操作 | Node-RED | Windmill / n8n | Temporal + Windmill + BPM |

---

## 13. MVP 实现顺序

```text
1. operation_definitions
2. operation_logs
3. Operation Gateway
4. FastAPI 操作接口
5. Casbin/OpenFGA 权限检查
6. Playwright Browser Engine
7. File Operation Engine
8. Knowledge Search Operation
9. Task/Workflow Operation
10. GitHub Issue Operation
```

---

## 14. 风险

| 风险 | 表现 | 对策 |
|---|---|---|
| Agent 绕过 Operation | 直接调浏览器或 shell | 所有工具只暴露 Operation Gateway |
| 操作粒度过大 | 一个操作包含复杂业务逻辑 | 复杂逻辑放工作流，操作保持原子 |
| 审计缺失 | 不知道谁做了什么 | operation_logs 强制记录 |
| 重试导致重复副作用 | 重复发送邮件、重复付款 | 幂等键、风险分类、人工确认 |
| 工具失控 | Browser use 操作错误页面 | URL 白名单、动作限制、截图记录 |

---

## 15. 完成定义

```text
所有工具能力已注册为 Operation。
每个 Operation 有参数 schema、风险等级、权限要求。
所有 Operation 调用经过权限判断。
所有执行结果进入 operation_logs。
失败可反馈到 Feedback Layer。
Agent 和 Workflow 只能通过 Operation Gateway 执行动作。
```
