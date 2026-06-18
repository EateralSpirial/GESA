# L2 Object & Permission Layer 对象与权限层

> Version: 0.1  
> Status: Layer Design  
> Position: Runtime Plane / Governance Boundary

---

## 1. 定位

对象与权限层是 GESA 的统一治理层，负责管理所有实体对象、权限策略、AI Agent 接入、操作注册和权限判断。

这一层回答的问题是：

```text
系统中有哪些对象？
谁拥有这些对象？
谁能对这些对象做什么？
AI Agent 能调用哪些工具？
哪些操作可以进入工作流？
哪些操作必须审批？
```

它是数据库层和知识库层之上的强制治理边界，也是所有上层动作的访问控制入口。

---

## 2. 核心职责

```text
1. 统一注册和管理所有实体对象。
2. 维护实体之间的所有权、从属、协作和引用关系。
3. 管理真人、AI Agent、部门、项目、客户、合同等对象的权限。
4. 管理 AI Agent 的能力、工具、上下文和执行边界。
5. 注册所有基础操作，并声明其权限、风险和审计要求。
6. 对上层所有操作请求执行权限判断。
7. 记录权限判断结果和权限变更审计。
```

---

## 3. 模块拆分

```text
Entity Management Module
- 实体注册
- 实体画像
- 实体关系
- 实体生命周期

Permission Management Module
- 权限模型
- 权限策略
- 权限判断
- 权限审计

AI Agent Management Module
- Agent 注册
- 能力范围
- 工具绑定
- 上下文范围
- 成本限制
- 健康状态

Operation Registry Module
- 操作定义
- 操作分类
- 权限绑定
- 风险等级
- 审批要求
- 是否可被 Agent / Workflow / UI 调用

Policy Decision Module
- 请求解析
- 策略评估
- 条件判断
- 审批触发
- 决策记录
```

---

## 4. Entity Management Module

### 4.1 实体类型

```text
human
ai_agent
department
organization
project
customer
contract
order
inventory_item
device
service_node
file
knowledge_document
workflow
task
feedback
module
operation
```

### 4.2 实体基础字段

```text
entity_id
entity_type
name
status
owner_entity_id
parent_entity_id
metadata_json
created_at
updated_at
deleted_at
```

### 4.3 实体关系

```text
owns
belongs_to
reports_to
manages
depends_on
blocks
references
executes
reviews
approves
contains
```

实体关系必须可被权限模块使用。例如：销售经理可以查看其团队客户；项目负责人可以修改其项目任务。

---

## 5. Permission Management Module

### 5.1 权限判断结构

```text
Subject      谁发起操作
Object       操作作用于谁
Action       做什么
Scope        在哪个范围内做
Condition    什么情况下允许
Approval     是否需要审批
Audit        是否必须记录
```

### 5.2 权限模型

GESA 权限不应只有 RBAC。建议采用组合模型：

```text
RBAC：角色权限，例如财务、销售、管理员。
ABAC：属性权限，例如部门、项目、金额、风险等级。
ReBAC：关系权限，例如负责人、上级、项目成员。
PBAC：策略权限，例如高风险操作必须审批。
```

### 5.3 权限结果

```text
allow
deny
require_approval
require_mfa
require_human_review
require_more_context
```

### 5.4 高风险操作

```text
付款
删除数据
修改权限
导出客户数据
修改生产配置
部署服务
提交代码
修改知识库正式规则
让 Agent 获得新工具权限
```

---

## 6. AI Agent Management Module

AI Agent 必须作为 Entity 管理。

```text
agent_id
entity_id
agent_type
capability_scope
available_tools
permission_policy_id
context_scope
task_queue_id
cost_limit
failure_policy
model_provider
model_name
version
health_status
created_at
updated_at
```

### 6.1 Agent 权限原则

```text
默认无权限。
每个工具调用都必须映射到 Operation。
每次 Operation 调用都必须经过权限判断。
Agent 上下文检索也必须经过知识权限过滤。
Agent 失败必须记录。
Agent 权限变更必须审计。
```

### 6.2 Agent 生命周期

```text
注册
  ↓
绑定能力
  ↓
绑定工具
  ↓
绑定权限
  ↓
进入工作空间
  ↓
执行任务
  ↓
记录成本和失败
  ↓
复盘与调优
  ↓
停用或升级
```

---

## 7. Operation Registry Module

所有原子操作进入系统前必须注册。

```text
operation_id
operation_name
operation_category
operation_description
required_permission
execution_engine
parameter_schema
result_schema
approval_required
audit_required
agent_callable
workflow_callable
ui_callable
risk_level
created_at
updated_at
```

### 操作类别

```text
Read
Write
Execute
Communicate
Schedule
Approve
Delegate
Configure
Evolve
```

---

## 8. 权限判断链路

```text
上层请求 Operation
  ↓
解析 requester_entity_id
  ↓
解析 target_entity_id
  ↓
读取 operation definition
  ↓
读取 subject/object/context
  ↓
执行 RBAC/ABAC/ReBAC/PBAC 判断
  ↓
返回 allow / deny / require_approval
  ↓
写入 permission_decision log
  ↓
允许后进入 Atomic Operation Layer
```

---

## 9. 核心 Schema

```text
entities
entity_profiles
entity_relations
roles
role_bindings
permissions
policy_definitions
policy_decisions
agent_profiles
agent_tool_bindings
agent_context_scopes
operation_definitions
operation_permission_bindings
approval_rules
```

---

## 10. 开源方案

| 功能 | 轻量级 | MVP | 企业级 |
|---|---|---|---|
| 实体后台 | PocketBase / Directus | Directus / NocoDB | Directus + SSO + 审计 |
| 身份认证 | PocketBase Auth / Authelia | Keycloak / Zitadel | Keycloak / Zitadel HA |
| 策略引擎 | Casbin | Casbin + OpenFGA | OpenFGA / SpiceDB + OPA |
| Agent 管理 | LangGraph config | LangGraph + Dify | LangGraph + Langfuse + 权限网关 |
| 操作注册 | FastAPI schema / YAML | OpenAPI + MCP | API Registry + MCP + 审计 |
| 审计 | SQLite logs | OpenTelemetry + DB logs | OpenTelemetry + SIEM / Loki |

---

## 11. MVP 实现顺序

```text
1. entities 表
2. entity_relations 表
3. roles / role_bindings
4. permissions / policy_definitions
5. Casbin 或 OpenFGA 初版
6. operation_definitions
7. agent_profiles
8. agent_tool_bindings
9. approval_rules
10. policy_decisions 日志
```

---

## 12. 风险

| 风险 | 表现 | 对策 |
|---|---|---|
| 权限绕过 | Agent 直接调用工具 | 所有工具都必须包装为 Operation |
| 权限过粗 | 只有管理员/普通用户 | 引入对象关系和上下文条件 |
| Agent 权限膨胀 | Agent 获得过多工具 | 默认最小权限、按任务授权、到期回收 |
| 知识检索越权 | RAG 绕开文档权限 | 检索前权限过滤，检索日志审计 |
| 权限逻辑分散 | UI、API、Agent 各判断一套 | 统一 Policy Decision Module |

---

## 13. 完成定义

```text
所有实体可注册。
所有实体关系可查询。
所有操作可注册并绑定权限。
所有 Agent 工具调用经过权限判断。
所有高风险操作可触发审批。
所有权限判断和权限变更有审计记录。
知识检索和工作流执行都受权限约束。
```
