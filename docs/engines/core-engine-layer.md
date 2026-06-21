# GESA Core Engine Layer

> Version: 0.1  
> Date: 2026-06-19  
> Status: Architecture Supplement  
> Position: Runtime / Meta shared core engine abstraction

---

## 1. 结论

GESA 不应只按垂直层级理解。随着自进化系统、Issue 调度、Agent 执行和语义检索能力进入项目，GESA 还需要一个横向的 **Core Engine Layer**。

Core Engine Layer 的职责是把系统中的关键能力拆成可独立治理、可独立演进、可相互调用的基础引擎。

```text
GESA Core Engine Layer
├── Database Engine
├── Semantic Engine
├── Agent Engine
├── Workflow Engine
├── Issue / Feedback Engine
├── Policy / Permission Engine
├── Operation / Tool Engine
└── Deployment Engine
```

其中，**Semantic Engine 应与 Agent Engine 同级**。

```text
Agent Engine    = 行动智能
Semantic Engine = 记忆智能
Database Engine = 事实状态
Workflow Engine = 流程秩序
Issue Engine    = 演化账本
Policy Engine   = 安全边界
Deployment Engine = 发布执行
```

---

## 2. 为什么需要 Core Engine Layer

原有分层结构可以说明系统从数据库、对象权限、原子操作、工作空间、反馈层到自进化层的纵向关系。

但自进化企业服务系统还有一组横向能力：

```text
检索
记忆
推理
调度
执行
审计
部署
回滚
```

这些能力会被多个层级共同调用。如果仍把它们塞进某一个垂直层级，后续会出现边界混乱：

```text
- Agent Engine 会被误解为拥有知识库。
- Knowledge Base 会被误解为只服务 Agent。
- Feedback Layer 会被误解为既做账本又做语义去重。
- Database Layer 会被误解为既保存事实又负责解释事实。
```

因此，GESA 应同时拥有两套视角：

```text
Vertical Layer View  = 业务系统结构
Core Engine View     = 基础能力结构
```

---

## 3. Core Engine Layer 总览

```text
┌────────────────────────────────────────────┐
│               Business Apps                 │
│ Finance / HR / Business / Admin / Client UI │
└────────────────────────────────────────────┘
                      ↓
┌────────────────────────────────────────────┐
│              API / Gateway Layer            │
└────────────────────────────────────────────┘
                      ↓
┌────────────────────────────────────────────┐
│              Core Engine Layer              │
│                                            │
│  ┌──────────────┐   ┌──────────────────┐   │
│  │ Agent Engine │←→ │ Semantic Engine  │   │
│  └──────────────┘   └──────────────────┘   │
│         ↓                    ↑             │
│  ┌──────────────┐   ┌──────────────────┐   │
│  │Workflow Engine│  │ Issue Engine     │   │
│  └──────────────┘   └──────────────────┘   │
│         ↓                    ↑             │
│  ┌──────────────┐   ┌──────────────────┐   │
│  │Policy Engine │   │ Deployment Engine│   │
│  └──────────────┘   └──────────────────┘   │
│         ↓                                  │
│  ┌──────────────────────────────────────┐  │
│  │ Operation / Tool Engine              │  │
│  └──────────────────────────────────────┘  │
└────────────────────────────────────────────┘
                      ↓
┌────────────────────────────────────────────┐
│              Data / Storage Layer           │
│ PostgreSQL / Vector DB / Object Store / Git │
└────────────────────────────────────────────┘
```

---

## 4. 各核心引擎职责

| 引擎 | 核心职责 | 不负责 |
|---|---|---|
| Database Engine | 保存事实、状态、事件、审计、指标 | 复杂解释、语义推理、自动决策 |
| Semantic Engine | 语义索引、相似检索、上下文召回、经验关联 | 最终行动、权限裁决、业务写入 |
| Agent Engine | 任务理解、计划生成、工具调用、代码执行、报告生成 | 长期索引、事实状态源、权限规则源 |
| Workflow Engine | 流程状态机、审批流、任务流转、节点编排 | 语义召回、代码修改 |
| Issue / Feedback Engine | 反馈入口、Issue 账本、状态协调、审计记录 | 直接执行代码修改 |
| Policy / Permission Engine | RBAC / ABAC、资源权限、上下文权限、审批策略 | embedding 相似度判断 |
| Operation / Tool Engine | 原子操作注册、工具适配、执行审计 | 高层任务规划 |
| Deployment Engine | 构建、发布、健康检查、回滚、版本记录 | 需求理解、修复决策 |

---

## 5. Semantic Engine 与 Agent Engine 的同级关系

### 5.1 Agent Engine

Agent Engine 负责行动：

```text
分析任务
拆解步骤
选择工具
调用工具
生成代码
运行测试
创建 PR
输出报告
```

### 5.2 Semantic Engine

Semantic Engine 负责记忆与关联：

```text
检索相关文档
检索相关代码
检索相似 Issue
检索历史修复
检索相关配置
检索数据库 schema 解释
构造 Context Pack
沉淀新经验
```

### 5.3 边界原则

```text
Agent Engine 决定做什么、怎么做。
Semantic Engine 决定参考什么、记起什么、关联什么。
```

二者互相调用，但任何一方都不应吞并另一方。

---

## 6. Knowledge Base Layer 与 Semantic Engine 的关系

Knowledge Base Layer 仍然存在，但定位需要更精确。

```text
Knowledge Base Layer = 知识内容、知识治理、知识对象结构
Semantic Engine      = 语义索引、检索召回、相似关联、Context Pack
```

也就是：

```text
Knowledge Base 保存知识。
Semantic Engine 调用、索引、连接和召回知识。
```

Knowledge Base Layer 可以包含：

```text
文档
规则
经验
SOP
FAQ
案例
知识图谱
知识版本
知识审批
```

Semantic Engine 可以包含：

```text
embedding worker
vector index
sparse index
hybrid retriever
reranker
context pack builder
retrieval logs
retrieval eval
```

---

## 7. Issue Engine、Semantic Engine、Agent Engine 的三分法

自进化闭环中必须区分三类职责：

```text
Issue Engine    = 问题进入系统的账本
Semantic Engine = 问题进入记忆网络
Agent Engine    = 问题进入执行流程
```

示例流程：

```text
客户反馈 / 系统告警 / GitHub Issue
        ↓
Issue / Feedback Engine
  - 记录原始反馈
  - 生成 Issue
  - 维护状态、标签、优先级
        ↓
Semantic Engine
  - 检索相似反馈
  - 检索相似 Issue
  - 检索相关文档
  - 检索相关代码
  - 检索历史修复
  - 构造 Context Pack
        ↓
Agent Engine
  - 读取 Context Pack
  - 拆解任务
  - 调用工具
  - 修改代码
  - 运行测试
  - 创建 PR
        ↓
Workflow / CI / Review / Deployment
        ↓
结果回写 Issue 与 Knowledge Base
        ↓
Semantic Engine 重新索引新经验
```

---

## 8. 核心调用契约

Core Engine 之间应通过明确接口调用。

```text
semantic.search(query, filters, top_k)
semantic.build_context_pack(task, scope, risk_level)
semantic.write_memory(source, summary, artifacts)

agent.plan(task, context_pack)
agent.execute(plan, tool_scope)
agent.report(run_id)

workflow.start(template_id, payload)
workflow.transition(run_id, event)
workflow.require_approval(run_id, policy)

issue.create(payload)
issue.classify(issue_id)
issue.link_pr(issue_id, pr_id)
issue.close(issue_id, resolution)

policy.check(subject, action, object, context)
policy.require_approval(operation, risk_level)

operation.invoke(operation_id, parameters, actor)
operation.audit(operation_id, result)

deployment.release(version, environment)
deployment.health_check(deployment_id)
deployment.rollback(deployment_id)
```

---

## 9. 权限与安全原则

Core Engine Layer 的所有调用都必须经过权限边界。

### 9.1 Semantic Engine 不得绕过权限

```text
Agent 能检索到什么，必须由 Policy / Permission Engine 控制。
```

Embedding 相似度不能决定：

```text
用户能不能读取某条知识
用户能不能访问某条客户数据
Agent 能不能调用某个工具
Agent 能不能修改生产配置
Agent 能不能执行部署
```

### 9.2 Agent Engine 不直接拥有系统权限

Agent Engine 每次调用工具前都应经过：

```text
Operation Registry
Policy Check
Audit Log
Risk Gate
```

### 9.3 高风险操作必须进入 Workflow Engine

高风险操作包括：

```text
权限升级
财务计算逻辑修改
数据库迁移
生产部署
客户数据删除
资金相关动作
合规相关动作
```

这些操作可以由 Agent 生成方案和 PR，但不应自动合并或直接执行。

---

## 10. 数据落点

Core Engine Layer 不是单一数据库表，而是一组服务与数据结构。

```text
Database Engine
- PostgreSQL / SQLite
- event tables
- audit logs
- entity state

Semantic Engine
- knowledge_documents
- knowledge_chunks
- semantic_embeddings
- sparse_terms
- vector_store
- retrieval_logs
- context_packs

Agent Engine
- agent_profiles
- agent_runs
- tool_calls
- plans
- execution_reports

Issue Engine
- feedback_events
- issue_mirror
- issue_labels
- triage_records

Workflow Engine
- workflow_templates
- workflow_runs
- workflow_nodes
- approvals

Deployment Engine
- releases
- deployment_runs
- health_checks
- rollback_records
```

---

## 11. 与原有层级文档的融合方式

原有层级文档不需要废弃，而应做定位修正：

```text
L1 Database Layer
- 仍是事实状态源。
- 不再承载“复杂语义召回”的完整职责。

L1.5 Knowledge Base Layer
- 仍是知识内容与治理层。
- Semantic Engine 从其中抽象出来，成为同级基础引擎。

L2 Object & Permission Layer
- 继续管理实体、权限、Agent、操作注册。
- 为 Core Engine 调用提供权限约束。

L3 Atomic Operation Layer
- 对应 Operation / Tool Engine 的操作执行面。

L4 Workspace Layer
- 调用 Agent Engine 与 Semantic Engine，为真人和 Agent 提供工作界面。

L5 Feedback Layer
- 对应 Issue / Feedback Engine 的反馈入口与任务账本。

M1 Project Control Plane
- 调度项目级规则、部署器、模块注册和增量更新。

M2 Self-Evolution Engine
- 主要由 Agent Engine + Semantic Engine + Issue Engine + Deployment Engine 协同实现。
```

---

## 12. 推荐落地顺序

### Phase 1：文档与接口先行

```text
1. 固化 Core Engine Layer 文档。
2. 固化 Semantic Engine 文档。
3. 在文档索引中加入 Core Engine 视角。
4. 明确 Knowledge Base Layer 与 Semantic Engine 边界。
```

### Phase 2：Semantic Engine MVP

```text
1. 文档切片。
2. BGE-M3 dense embedding。
3. BM25 / BGE-M3 sparse。
4. metadata filter。
5. context pack builder。
6. retrieval logs。
```

### Phase 3：Agent Engine 对接

```text
1. Issue 进入后先调用 Semantic Engine。
2. Semantic Engine 返回 Context Pack。
3. Agent Engine 只基于授权 Context Pack 执行。
4. PR 与修复报告回写 Issue。
```

### Phase 4：自进化闭环

```text
1. 已解决 Issue 写回经验知识。
2. 新 PR / commit / test report 进入索引。
3. 失败复盘进入知识候选。
4. 召回质量进入 eval。
```

---

## 13. 架构判断

GESA 的核心抽象应从单纯分层系统升级为：

```text
分层系统 + 核心引擎系统
```

一句话定义可修正为：

> GESA 是一套以数据库保存企业事实状态、以知识库保存企业认知结构、以 Semantic Engine 提供语义记忆与上下文召回、以 Agent Engine 提供规划和行动、以 Issue Engine 形成演化账本、以 Workflow / Policy / Deployment Engine 约束执行闭环的企业自进化服务架构。
