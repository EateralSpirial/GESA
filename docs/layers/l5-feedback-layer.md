# L5 Feedback Layer 使用反馈层

> Version: 0.1  
> Status: Layer Design  
> Position: Runtime Plane / Evolution Input

---

## 1. 定位

反馈层是 GESA 自进化系统的输入口。它负责收集真人、AI Agent、客户、工作流、系统服务、部署节点和操作引擎产生的反馈，并将其分类、归因、分流为任务、知识更新候选或项目自进化候选。

它回答的问题是：

```text
哪里不好用？
哪里失败了？
哪里阻塞了？
哪里权限不合理？
哪里流程低效？
哪些反馈应变成任务？
哪些反馈应变成知识？
哪些反馈应推动项目自进化？
```

---

## 2. 核心职责

```text
1. 接收所有实体对象提交的反馈。
2. 接收系统自动生成的失败、异常、阻塞和风险反馈。
3. 对反馈进行分类、打标签、聚类和严重程度判断。
4. 将普通反馈转为任务或工作空间事项。
5. 将知识类反馈转为知识更新候选。
6. 将系统性反馈转为 evolution candidate。
7. 将安全反馈升级并触发阻断或审批。
8. 为自进化引擎提供结构化输入。
```

---

## 3. 反馈来源

```text
真人反馈
AI Agent 失败反馈
客户反馈
任务超时反馈
工作流阻塞反馈
权限冲突反馈
UI 使用困难反馈
部署异常反馈
数据缺失反馈
安全风险反馈
操作失败反馈
知识缺失反馈
自进化失败反馈
```

---

## 4. 反馈类型

| 类型 | 说明 | 处理方向 |
|---|---|---|
| Bug | 功能错误 | Issue / 修复任务 |
| Process Inefficiency | 流程低效 | 工作流改进候选 |
| Permission Missing | 权限不足 | 权限调整候选 |
| Permission Too Broad | 权限过宽 | 安全审计候选 |
| Data Missing | 数据结构缺失 | schema 改进候选 |
| UI Confusion | UI 不清晰 | 可视化改进候选 |
| Agent Failure | Agent 能力不足或执行失败 | Agent 调优 / 知识补充 |
| Tool Failure | 工具调用失败 | Operation 修复候选 |
| Business Rule Conflict | 业务规则冲突 | 知识或流程更新 |
| Deployment Issue | 部署问题 | M1 / M2 候选 |
| Security Risk | 安全风险 | 阻断、升级、审计 |
| Feature Request | 新功能需求 | 产品候选 / 自进化候选 |
| Knowledge Gap | 知识缺失 | 知识更新候选 |

---

## 5. Feedback Schema

```text
feedback_id
source_entity_id
target_entity_id
related_task_id
related_workflow_id
related_operation_id
related_knowledge_id
feedback_type
severity
description
evidence_uri
evidence_json
suggested_fix
status
created_at
updated_at
```

---

## 6. 分流规则

```text
普通反馈 → 工作空间任务
知识缺失 → knowledge_update_candidate
流程低效 → workflow_improvement_candidate
操作失败 → operation_fix_candidate
权限问题 → permission_review_candidate
系统性问题 → evolution_candidate
安全风险 → security_escalation + operation block
```

---

## 7. 反馈处理链路

```text
反馈提交
  ↓
基础校验
  ↓
权限与来源确认
  ↓
分类和严重程度判断
  ↓
证据绑定
  ↓
去重和聚类
  ↓
分流为任务 / 知识候选 / 进化候选 / 安全升级
  ↓
进入工作空间或项目控制平面
  ↓
处理结果回写反馈状态
```

---

## 8. 自动反馈

系统应自动产生反馈：

```text
operation.failed
workflow.timeout
agent.tool_error
agent.low_confidence
permission.denied_repeatedly
ui.error
service.unhealthy
deployment.failed
knowledge.not_found
retrieval.low_quality
```

自动反馈必须带 evidence，例如日志、trace、截图、任务 ID、操作 ID、Agent 输出。

---

## 9. 与知识库的关系

反馈层不只驱动代码变化，也驱动知识更新。

```text
用户问了系统无法回答的问题 → FAQ 候选
Agent 因缺少 SOP 失败 → SOP 更新候选
工作流阻塞原因重复出现 → 流程文档更新候选
部署失败后人工解决 → 运维手册更新候选
权限冲突频繁出现 → 权限规则说明更新候选
```

---

## 10. 与自进化引擎的关系

Feedback Layer 输出 `evolution_candidates`，供 M2 Self-Evolution Engine 读取。

```text
evolution_candidate_id
source_feedback_ids
candidate_type
problem_summary
root_cause_hypothesis
suggested_change
risk_level
affected_modules
required_tests
status
created_at
```

候选不能直接修改项目。候选必须经过 M1 项目控制平面和 M2 自进化引擎流程。

---

## 11. 开源方案

| 功能 | 轻量级 | MVP | 企业级 |
|---|---|---|---|
| 功能反馈 | Fider | Fider / Formbricks | Formbricks + 产品分析 |
| Issue | GitHub Issues / Gitea | GitHub / Gitea / Plane | GitLab / Jira 替代栈 / Plane |
| 客服 | 无 / Chatwoot Lite | Chatwoot / Zammad | Chatwoot + 工单系统 |
| 错误追踪 | 日志文件 | Sentry / OpenTelemetry | Sentry + OpenTelemetry + Loki |
| 产品分析 | Umami | PostHog / Matomo | PostHog + OpenReplay |
| Agent 反馈 | JSON logs | Langfuse / Phoenix | Langfuse + Phoenix + Eval |
| 知识分流 | GitHub Issues | Wiki.js / Outline | 知识门户 + OpenSearch |

---

## 12. MVP 实现顺序

```text
1. feedbacks 表
2. feedback_type 分类
3. severity 分类
4. feedback submission API
5. feedback → task
6. feedback → knowledge_update_candidate
7. feedback → evolution_candidate
8. GitHub/Gitea Issue 同步
9. Agent failure 自动反馈
10. 反馈聚类和周报
```

---

## 13. 风险

| 风险 | 表现 | 对策 |
|---|---|---|
| 反馈噪音过多 | 大量低价值反馈淹没系统 | 分类、去重、聚类、严重度 |
| 反馈直接改系统 | 未审查变更进入生产 | 反馈只能变候选，不能直接执行 |
| 证据不足 | 无法复现问题 | evidence_uri/evidence_json 必填 |
| Agent 失败不可见 | 只看到最终失败 | Agent 调用必须生成 trace 和 feedback |
| 安全反馈处理慢 | 高风险问题未阻断 | security feedback 触发升级和阻断 |

---

## 14. 完成定义

```text
所有实体可提交反馈。
系统可自动生成失败反馈。
反馈有类型、严重度、证据和状态。
反馈可转任务、知识候选和自进化候选。
安全反馈可升级处理。
反馈处理结果可回写。
自进化引擎可读取结构化候选。
```
