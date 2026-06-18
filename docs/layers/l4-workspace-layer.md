# L4 Workspace Layer 工作空间层

> Version: 0.1  
> Status: Layer Design  
> Position: Runtime Plane / Coordination Experience

---

## 1. 定位

工作空间层是 GESA 的协调与体验层。它负责把数据库状态、知识上下文、权限边界和原子操作能力组合成真人和 AI Agent 都能使用的工作环境。

这一层回答的问题是：

```text
每个实体本周要做什么？
任务如何安排到日程？
工作流如何创建、修改、合并和执行？
真人如何用可视化界面兜底操作？
秘书 Agent 如何协调对象、任务、日程、流程和反馈？
```

工作空间层是秘书 Agent 的主场。

---

## 2. 核心职责

```text
1. 为每个实体创建独立工作空间。
2. 聚合任务、日程、工作流、知识、反馈、风险和审批。
3. 以周为基本周期生成计划、执行、复盘和改进候选。
4. 提供工作流创建、管理、修改、版本化和合并能力。
5. 提供日程、排班、提醒、截止时间和冲突检测。
6. 为真人提供低负担可视化界面。
7. 为 AI Agent 提供任务队列、工具调用入口和执行状态。
8. 在 Agent 不可用时提供人工兜底操作。
```

---

## 3. Workspace 类型

```text
Human Workspace
- 我的任务
- 我的日程
- 我的审批
- 我的风险
- 我的秘书

AI Agent Workspace
- 任务队列
- 工具权限
- 执行记录
- 失败复盘
- 成本统计

Project Workspace
- 目标
- 里程碑
- 任务流
- 依赖关系
- 项目知识

Department Workspace
- 团队任务
- 排班
- 指标
- 风险
- 协作关系

Customer Workspace
- 客户画像
- 联系记录
- 合同
- 问题
- 跟进计划

Contract / Order Workspace
- 生命周期状态
- 责任人
- 审批节点
- 相关文档
- 风险提示
```

---

## 4. 模块拆分

```text
Workspace Core
- 工作空间创建
- 工作空间绑定实体
- 工作空间状态聚合
- 工作空间权限边界

Workflow Module
- 工作流创建
- 工作流修改
- 工作流合并
- 工作流版本管理
- 工作流运行记录

Schedule Module
- 周计划
- 日程
- 截止时间
- 会议
- 排班
- 时间冲突检测

Visualization Module
- 看板
- 日历
- 时间线
- 审批入口
- 风险提示
- 操作按钮

Secretary Agent Module
- 周计划生成
- 任务协调
- 自然语言交互
- 复盘总结
- 反馈提交

Manual Fallback Module
- 表单
- 按钮
- 手动审批
- 手动状态修改
- Agent 故障兜底
```

---

## 5. 周期管理

GESA 以周为工作空间的基本管理周期。

```text
年度目标
  ↓
季度战役
  ↓
月度项目
  ↓
周工作流
  ↓
日任务 / 实时事件
```

每周循环：

```text
周一：读取目标、知识、任务和日程，生成本周计划。
周二至周五：执行、提醒、协调、处理异常。
周六：生成周复盘，识别阻塞和风险。
周日：沉淀经验，形成知识更新候选和工作流改进候选。
```

---

## 6. Secretary Agent 职责

```text
读取实体状态
读取相关知识
生成周计划
整理任务池
安排日程
创建工作流
调用原子操作
请求审批
协调多个实体
汇总反馈
生成复盘
提出改进建议
```

### 行动链路

```text
Secretary Agent
  ↓
Workspace Module
  ↓
Knowledge Retrieval
  ↓
Operation Gateway
  ↓
Permission Decision
  ↓
Atomic Operation Execution
  ↓
Database / Feedback / Audit
```

秘书 Agent 不允许绕过权限和原子操作层。

---

## 7. 可视化设计原则

真人只需要看到：

```text
我要做什么
什么时候做
为什么重要
谁在等我
我在等谁
我需要审批什么
哪里出问题了
下一步建议是什么
```

推荐视图：

| 视图 | 用途 |
|---|---|
| Week View | 本周目标、任务、日程、风险 |
| Task Board | To Do / Doing / Blocked / Done |
| Calendar | 会议、截止时间、排班 |
| Workflow View | 流程节点、状态、审批、异常 |
| Risk Panel | 超时、阻塞、权限冲突、失败操作 |
| Knowledge Sidebar | 相关 SOP、案例、规则、FAQ |
| Secretary Chat | 自然语言交互和建议确认 |

---

## 8. 工作流模型

```text
workflow_id
name
owner_entity_id
applicable_entity_types
version
status
trigger_type
steps_json
permission_policy_id
exception_policy
created_at
updated_at
```

工作流步骤：

```text
step_id
step_type
operation_id
assignee_entity_id
required_permission
approval_required
timeout_policy
retry_policy
fallback_action
next_steps
```

---

## 9. 无 Agent 兜底机制

```text
有 Agent：Agent 生成建议，用户确认或修改。
无 Agent：系统展示按钮、表单、看板、日程。
Agent 故障：回退到人工可视化操作。
权限不足：显示申请权限或请求审批入口。
操作失败：显示错误原因、重试、转人工、提交反馈。
```

兜底机制是工作空间层的硬要求。

---

## 10. 核心 Schema

```text
workspaces
workspace_members
workspace_views
workspace_widgets
weekly_plans
weekly_reviews
tasks
task_dependencies
calendar_items
workflow_definitions
workflow_runs
workflow_steps
approval_items
risk_items
workspace_notifications
```

---

## 11. 开源方案

| 功能 | 轻量级 | MVP | 企业级 |
|---|---|---|---|
| 工作流 | APScheduler / Node-RED | Temporal / Windmill | Temporal + BPMN 引擎 |
| BPM | 无 / 简单状态机 | Flowable / Operaton | Flowable / Operaton + DMN |
| 任务看板 | Kanboard / Vikunja | Plane / OpenProject | OpenProject / 自研 Workspace |
| 日历 | Radicale / CalDAV | Cal.com | Cal.com + 企业日历集成 |
| 可视化 | NiceGUI / Directus | Appsmith / ToolJet | Appsmith / 自研前端 |
| Agent 工作台 | LangGraph CLI/UI | Dify / OpenWebUI / LangGraph | LangGraph + Langfuse + 权限网关 |
| BI/指标 | DuckDB | Metabase | Grafana / Superset / ClickHouse |

---

## 12. MVP 实现顺序

```text
1. workspace 实体绑定
2. task board
3. weekly_plan
4. calendar_items
5. workflow_definitions 简化版
6. workflow_runs
7. approval_items
8. risk_items
9. knowledge sidebar
10. secretary chat
11. manual fallback buttons
```

---

## 13. 风险

| 风险 | 表现 | 对策 |
|---|---|---|
| 工作空间变成大杂烩 | 所有模块都塞进 UI | Workspace 只做聚合和编排 |
| Agent 黑箱决策 | 用户不知道为什么被安排任务 | 所有建议附来源、知识引用和原因 |
| 没有兜底 | Agent 故障后系统瘫痪 | 表单、按钮、看板必须可手动操作 |
| 工作流过早复杂 | MVP 变成 BPM 巨系统 | 先状态机，后 BPMN |
| 周期管理缺失 | 只有任务列表，没有复盘 | weekly_plan + weekly_review 必须保留 |

---

## 14. 完成定义

```text
每个实体可拥有工作空间。
工作空间能展示任务、日程、工作流、知识和反馈。
真人可通过 UI 操作关键流程。
Secretary Agent 可生成周计划和复盘。
工作流可调用原子操作并受权限约束。
Agent 故障时存在人工兜底路径。
```
