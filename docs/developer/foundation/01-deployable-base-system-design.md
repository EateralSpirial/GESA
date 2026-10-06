# GESA 可部署基座系统设计

> Version: 0.1  
> Date: 2026-10-07  
> Status: Foundation System Design  
> Scope: Rust 可部署基座、中央控制平面、企业运行节点、终端、模块系统与信息流内核  
> Parent: [GESA 开发者架构层](../README.md)

---

## 0. 文档结论

GESA 的第一份可运行实体应定义为：

```text
GESA Foundation Runtime
= 信息流内核
+ 通用对象模型
+ 组织拓扑
+ 确定性权限引擎
+ 原子操作入口
+ 事件与审计
+ 模块与企业剖面
+ 节点与更新机制
```

第一版采用：

```text
Rust 模块化单体内核
+ 企业本地运行节点
+ Windows / Linux 终端
+ 独立的 GESA 中央控制平面
```

核心架构判断：

1. **信息流是第一性对象。** 任何读取、写入、执行、审批、同步、更新和隐私数据交换，都必须成为有类型、有来源、有目的、有权限、有版本、有审计的信息流。
2. **母结构由稳定契约组成。** 企业子结构由模块、规则、工作流和界面剖面裁剪生成，保持同一代码谱系。
3. **中央端承担控制平面。** 企业业务数据与业务运行默认留在企业节点；中央端提供发布、规则、API、节点身份、兼容性和支持服务。
4. **企业本地运行保持自治。** 中央端暂时不可用时，企业内部授权、业务操作、任务与审计仍可继续。
5. **权限结论由确定性引擎产生。** 组织几何、部门投影、角色、直接授权和委托会话都属于权限输入。
6. **首版选择模块化单体。** 内部模块从第一天拥有清晰端口与契约；达到独立扩缩容、故障隔离或合规边界后再拆分进程。
7. **Rust 负责信任边界。** 内核、API、权限、更新、模块治理、审计和节点通信使用 Rust；未来模型推理工作器可以采用独立进程，通过受控协议接入。

---

## 1. 设计依据与当前文档融合

现有 GESA 文档已经形成两组互补视角：

```text
企业视角：
组织权限图、共享空间、个人工作区、Agent Team、对象聊天、日程与执行器。

开发视角：
数据库、知识库、权限、原子操作、核心引擎、Issue、部署和自进化。
```

本设计把两组视角映射到同一份可执行基座：

| 企业侧概念 | 基座实现 |
|---|---|
| 企业、部门、层级、方向、格子 | Object Kernel + Organization Module |
| 组织权限图 | Organization Scope Resolver + Policy Engine |
| 共享空间、个人工作区 | Space / Resource Module + Workspace Projection |
| 人类、Agent、服务节点 | Actor / Node / Delegation Model |
| 对象聊天、日程、任务 | 可选业务模块，统一注册 Operation 与 Event |
| 执行器 | Operation Gateway + Worker Scheduler |
| 使用反馈 | Feedback / Issue Adapter |
| 自动更新与补丁 | Central Release Service + Local Update Agent |
| 通用规则 | Signed Rule Pack Registry |
| 隐私 API | Enterprise-local Privacy Gateway |
| 自进化 | 外部 Agent 通过 PR、测试、签名发布进入控制平面 |

本文件保留现有分层与核心引擎设计，同时补充以下缺口：

```text
1. 信息流契约
2. 母结构与企业子结构的生成方式
3. 中央控制平面与企业数据平面的边界
4. Rust 进程、crate、协议与部署形态
5. Windows / Linux 的实际发布路径
6. 更新、回滚、密钥和隐私 API 的安全边界
7. 第一条可验收的端到端垂直切片
```

---

## 2. 第一性原理：信息流通规则

### 2.1 信息流定义

GESA 中的一次信息流可以抽象为：

```text
Flow =
Source
× Destination
× PayloadType
× Purpose
× Action
× Scope
× Policy
× Transformation
× Delivery
× Audit
```

每一次流通都必须回答：

```text
谁发出？
发给谁？
传递什么类型的信息？
用于什么目的？
请求什么动作？
作用于哪个对象和范围？
依据什么策略允许？
经过什么脱敏、聚合或格式转换？
采用何种交付与重试语义？
留下什么可追溯记录？
```

### 2.2 五类基础消息

| 类型 | 含义 | 状态语义 |
|---|---|---|
| Query | 查询现有状态或投影 | 不产生预期业务写入 |
| Command | 请求执行一次状态变更 | 成功后产生事实 |
| Domain Event | 已经发生且提交成功的域内事实 | 追加记录 |
| Integration Event | 允许跨模块或节点传播的事实投影 | 可脱敏、可裁剪 |
| Control Message | 更新、心跳、配置、证书、兼容性等控制信息 | 受控制平面策略约束 |

大型文档、附件、模型和构建产物通过 `ArtifactRef` 传递：

```text
ArtifactRef
- artifact_id
- content_hash
- media_type
- size
- storage_location
- encryption_metadata
- retention_policy
```

消息本体只携带引用，降低重复传输与敏感数据扩散。

### 2.3 标准消息信封

```text
MessageEnvelope
- message_id
- tenant_id
- flow_type
- schema_id
- schema_version
- source_object_id
- destination_object_id
- actor_principal_id
- effective_principal_id
- delegation_chain
- action
- resource_ref
- scope
- purpose
- data_classification
- correlation_id
- causation_id
- trace_id
- idempotency_key
- issued_at
- expires_at
- policy_context
- payload_ref / payload
- signature
```

### 2.4 信息流不变量

所有实现必须维护以下不变量：

```text
Typed          每一条流都有 schema 与版本。
Attributed     每一条流都能定位发起者、有效主体和委托链。
Authorized     每一次读取与写入都经过确定性权限判断。
Purpose-bound  跨边界流通必须声明用途。
Minimized      只传递完成用途所需的最少字段。
Versioned      API、事件、规则和模块都声明兼容版本。
Idempotent     可重试写入使用幂等键。
Observable     每条关键流拥有 trace 与状态。
Auditable      决策和结果形成审计记录。
Recoverable    高价值变更拥有补偿、回滚或人工恢复路径。
```

### 2.5 一致性模型

首版采用清晰而有限的保证：

```text
单个企业数据库事务内：
- PostgreSQL ACID
- 状态写入、域事件、审计和 outbox 同一事务提交

跨进程或跨节点：
- 至少一次投递
- 幂等消费
- 最终一致
- 无全局分布式事务承诺
```

GESA 首版使用“当前状态 + 追加事件 + 审计记录”的组合。完整事件溯源可以在特定模块按需启用，基座无需依靠重放全部历史来恢复所有业务状态。

---

## 3. 母结构与企业子结构

### 3.1 形式化定义

```text
MotherStructure =
KernelContracts
+ CoreModules
+ ObjectTypeCatalog
+ OperationCatalog
+ EventCatalog
+ RuleCatalog
+ SchemaCatalog
+ ModuleCatalog
+ MigrationGraph
```

```text
EnterpriseDistributionProfile =
AllowedModules
+ RequiredCoreModules
+ ReleaseChannel
+ APIEntitlements
+ CompatibilityRange
+ DataResidencyConstraints
```

```text
EnterpriseLocalConfiguration =
OrganizationTopology
+ Memberships
+ LocalPolicies
+ LocalWorkflows
+ UIConfiguration
+ ConnectorConfiguration
+ RetentionRules
+ LocalRuleOverrides
```

```text
EnterpriseInstance =
FoundationRuntime
+ SignedDistributionProfile
+ SignedLocalConfiguration
+ SelectedModules
+ LocalData
```

### 3.2 子集生成原则

企业子结构通过配置与模块装载产生，保持同一母结构：

```text
1. Required Core 始终存在。
2. 可选业务模块按 EnterpriseDistributionProfile 启用。
3. 本地配置可以收紧权限、数据外发和执行范围。
4. 本地配置只能在已授权能力集合内扩展业务对象。
5. 模块卸载前必须完成依赖检查、数据保留策略和迁移计划。
6. 企业实例保存自己的有效配置快照与来源证明。
```

基座强制保留的核心模块：

```text
Object Identity
Tenant / Enterprise
Actor & Session
Organization Topology
Policy / Authorization
Operation Registry & Gateway
Event / Outbox
Audit
Module Registry
Node Identity
Update Agent
Configuration & Secret References
Health / Observability
```

可裁剪模块：

```text
Workflow
Task / Schedule
Workspace
Semantic Engine
Agent Engine
Feedback / Issue
Finance
HR
CRM
Inventory
Procurement
Document
Connector Packs
```

### 3.3 避免客户代码分叉

客户差异首先表达为：

```text
Profile
Rule Pack
Workflow Pack
UI Surface Pack
Connector Pack
Business Module
```

只有长期稳定、具备通用价值的变化才进入母结构。单一客户的特殊逻辑保留为独立模块或规则包。该机制使补丁、迁移和通用规则可以沿同一版本链传播。

---

## 4. 统一对象模型

### 4.1 根对象

现有文档中“所有业务对象均为 Entity”与“Entity 仅含 Human / Agent”存在口径差异。基座统一为：

```text
GesaObject
├── EntityObject
│   ├── ActorEntity
│   │   ├── HumanActor
│   │   ├── AgentActor
│   │   └── ServiceActor
│   ├── OrganizationEntity
│   │   ├── Enterprise
│   │   ├── OrgUnit
│   │   ├── Department
│   │   ├── Team
│   │   └── Position
│   └── BusinessEntity
│       ├── Project
│       ├── Customer
│       ├── Contract
│       ├── Order
│       └── Module-defined Entity
├── OrganizationTopologyObject
│   ├── Level
│   ├── Direction
│   ├── Cell
│   ├── Membership
│   └── ReportingRelation
├── SpaceObject
├── ResourceObject
├── OperationDefinition
├── PolicyObject
├── WorkflowObject
├── TaskObject
├── MessageObject
├── ArtifactObject
├── ModuleObject
├── NodeObject
└── ReleaseObject
```

所有对象共享：

```text
object_id
tenant_id
object_kind
schema_version
name
lifecycle_state
owner_ref
created_at
updated_at
metadata
```

`ObjectRef` 是所有授权、关系、消息、事件和审计的统一引用。推荐使用可排序的 UUIDv7，并在所有外部接口中以字符串编码。

### 4.2 主体与资源

权限模型中区分：

```text
PrincipalRef = Human / Agent / Service / Role / OrgUnit / AgentTeam
ResourceRef  = 任意可授权 GesaObject
```

同一个对象可以在某次决策中成为主体，在另一次决策中成为资源。例如部门可以获得权限，也可以被管理。

### 4.3 对象关系

统一关系表或关系端口表达：

```text
owns
member_of
reports_to
occupies
contains
references
assigned_to
delegates_to
derived_from
supersedes
depends_on
```

高频、强约束关系可以拥有专用表；所有关系仍映射为统一 `ObjectRelation` 供审计、检索和 API 使用。

---

## 5. 组织拓扑与权限

### 5.1 组织拓扑

保留现有组织权限图：

```text
Cell = Level × Direction
Department = Set<Cell>
```

基座数据对象：

```text
Level
Direction
Cell
OrgUnit
OrgUnitCellAssignment
Membership
Position
ReportingRelation
```

硬约束：

```text
1. 同一企业内，Cell(level, direction) 唯一。
2. 同一个 Cell 在同一时刻最多由一个部门占用。
3. 组织变更拥有生效时间和历史记录。
4. 成员身份、岗位、部门首脑与格子占用彼此独立建模。
```

### 5.2 组织范围推导

部门向下投影生成 `OrgScope`：

```text
OrgScope(department) =
union of downward projections for all occupied cells
```

`OrgScope` 表示候选管理范围。它参与权限计算，同时保留资源敏感级别、直接授权、显式拒绝、委托边界和风险规则。

### 5.3 权限决策

统一调用：

```text
authorize(
  principal,
  action,
  resource,
  context
) -> Decision
```

上下文至少包含：

```text
tenant
session
device
network_zone
purpose
delegation
org_scope
data_classification
risk_level
time
approval_state
```

组合优先级：

```text
Explicit Deny
  >
Risk Restriction
  >
Local Mandatory Policy
  >
Direct Grant / Role / Org Scope / Delegation / Global Policy
  >
Default Deny
```

输出：

```text
Decision
- effect: allow | deny | require_approval
- reason_codes
- matched_policy_ids
- obligations
- redaction_plan
- audit_level
- expires_at
```

### 5.4 CEO 与最高权限管理

治理权和技术恢复权分开表达：

```text
CEO：
- 必须是 HumanActor。
- 拥有 CEO 任命动作的唯一治理权限。
- 可以授予最高权限管理角色。
- 所有行为仍经过策略、风险门和审计。

Highest Permission Manager：
- 管理层级、方向、格子、部门和授权系统。
- 权限范围来自显式策略。
- 角色可以被撤销、限时和审计。

Break-glass Recovery：
- 只用于系统恢复。
- 采用多方密钥、限时会话、限定范围和强制告警。
- 不产生 CEO 任命能力。
```

该设计保留企业治理规则，同时消除隐藏的系统级绕过路径。

### 5.5 Agent 权限

```text
1. Agent 作为 ActorEntity 管理。
2. Agent 每次执行都使用 DelegatedPermissionSession 或服务身份。
3. Agent 只调用已注册 Operation。
4. Agent 不直接访问业务数据库、密钥或任意系统工具。
5. 语义检索结果先应用资源权限和字段过滤。
6. 高风险动作进入审批工作流。
7. 委托范围、成本、时限、工具和数据分类均可约束。
```

---

## 6. 物理部署拓扑

### 6.1 三层拓扑

```text
┌─────────────────────────────────────────────────────────┐
│ GESA Central Control Plane                              │
│ Fleet / Release / Rule / API / Privacy Broker / PKI    │
└───────────────────────▲─────────────────────────────────┘
                        │ outbound mTLS, pull-first
                        │
┌───────────────────────┴─────────────────────────────────┐
│ Enterprise Runtime                                      │
│ API / Object / Org / Policy / Operation / Event / Audit │
│ PostgreSQL / Artifact Store / Module Host / Update Agent│
└──────────────▲──────────────────────▲───────────────────┘
               │ HTTPS / SSE / WS     │ mTLS task channel
               │                      │
┌──────────────┴─────────────┐  ┌─────┴──────────────────┐
│ Employee / Manager Terminal │  │ Compute Worker Nodes   │
│ Tauri UI + secure cache      │  │ CPU / GPU / connectors │
└──────────────────────────────┘  └────────────────────────┘
```

### 6.2 中央控制平面

中央端提供：

```text
Fleet Registry
- 企业实例、节点、版本、能力和健康状态

Release Service
- 构建产物、发布通道、兼容矩阵、撤回和安全公告

Rule Pack Registry
- 通用权限规则、校验规则、工作流规则、隐私规则

Profile Service
- 企业分发剖面、必选模块、可选模块和能力范围

API Catalog & Gateway
- 通用 API、版本、配额、租户凭据和服务发现

Privacy API Broker
- 目的约束请求、字段白名单、聚合规则、回执与审计

Certificate / Device Identity
- 企业节点和工作节点身份、轮换与吊销

Support & Telemetry
- 经企业授权的健康指标、诊断包和支持会话
```

中央控制平面不进入企业内部每一次业务授权的同步关键路径。

### 6.3 企业运行节点

企业节点承载：

```text
Enterprise API Gateway
Identity Adapter / Session Validator
Object Kernel
Organization Module
Policy Engine
Operation Gateway
Workflow / Approval optional
Event + Outbox + Inbox
Audit Ledger
Module Host
Node Scheduler
Update Agent
PostgreSQL
Artifact Store
Observability
```

生产基线使用 PostgreSQL。终端缓存使用 SQLite。文件与大型产物先采用本地内容寻址目录，接口保持 S3 兼容存储的迁移空间。

### 6.4 终端

终端承担：

```text
UI rendering
local encrypted cache
session handling
low-risk offline command queue
notifications
user-approved file selection
device-local capabilities explicitly granted by user
```

终端不直接连接企业数据库。所有业务读写通过企业 API 进入信息流内核。

### 6.5 工作节点与异构设备

`NodeRegistry` 记录：

```text
node_id
node_role
os
architecture
cpu
memory
gpu
storage
network_zone
capabilities
labels
health
certificate
runtime_version
```

`OperationDefinition` 声明执行要求：

```text
execution_class
required_capabilities
resource_limits
data_locality
sandbox_profile
supported_os
timeout
retry_policy
```

调度器选择：

```text
enterprise_server
dedicated_worker
terminal_assisted
external_connector
```

高风险与高计算任务优先放在服务器或隔离工作节点。终端辅助执行需要用户可见确认和最小能力授权。

---

## 7. 进程与模块架构

### 7.1 首版进程

```text
gesa-enterprise-server
- 企业 API
- 核心模块
- 内置后台任务
- 本地策略与审计
- update-agent controller

gesa-control-plane
- 中央 Fleet / Release / Rule / Profile / API 服务

gesa-worker
- 可选独立计算与连接器工作器

gesa-desktop
- Windows / Linux Tauri 终端

gesa-admin-cli
- 安装、迁移、诊断、密钥和恢复操作
```

单机开发环境允许把企业服务器和工作器合并运行。中央控制平面保持独立部署。

### 7.2 模块化单体

首版内部模块在一个企业服务器进程中运行，具备：

```text
独立 crate
独立 schema namespace
显式依赖
注册式 API
注册式 Operation
注册式 Event
独立 migration
独立 health check
独立 feature flag
```

模块拆分为独立服务的触发条件：

```text
1. 需要独立扩缩容。
2. 故障需要隔离。
3. 发布节奏显著独立。
4. 合规或数据驻留要求独立边界。
5. 资源模型与主进程差异显著。
```

### 7.3 模块类型

| 类型 | 运行方式 | 用途 |
|---|---|---|
| Core Module | 编译进基座 | 对象、组织、权限、操作、事件、审计、更新 |
| Business Module | 编译或受控加载 | 财务、人事、客户、库存等 |
| Connector Module | 工作器或隔离进程 | 外部 ERP、邮件、GitHub、设备 |
| Rule Pack | 声明式数据 | 权限、校验、隐私、兼容性 |
| Workflow Pack | 声明式状态机 | 审批、任务与业务流程 |
| UI Extension | 已签名前端资源 | 页面、菜单、对象视图 |
| WASM Component | Wasmtime 沙箱 | 低信任、跨平台、可动态装载扩展 |
| Migration Pack | 一次性受控执行 | schema、数据与配置迁移 |

Rust 原生动态库不作为通用插件边界。内置高信任模块采用静态 crate；运行期扩展优先使用 WebAssembly Component Model 或独立进程协议。

### 7.4 模块清单

```text
ModuleManifest
- module_id
- name
- version
- module_api_version
- required_kernel_range
- schema_epoch
- dependencies
- conflicts
- capabilities
- object_types
- operations
- emitted_events
- consumed_events
- routes
- ui_extensions
- permission_actions
- data_classifications
- migrations
- health_checks
- artifact_hash
- publisher
- signature
```

模块在激活前依次通过：

```text
签名验证
兼容性检查
依赖解析
权限能力检查
migration 预检
健康检查
注册表提交
```

---

## 8. Rust 技术基线

### 8.1 首选组件

| 能力 | 首选方案 | 说明 |
|---|---|---|
| 异步运行时 | Tokio | 企业服务器与控制平面 |
| HTTP API | Axum + Tower | REST、SSE、WebSocket、中间件 |
| 内部 RPC | tonic / Protobuf | 控制平面、工作节点和流式任务 |
| 序列化 | serde | JSON、TOML 与内部数据 |
| API Schema | OpenAPI + JSON Schema | 外部接口与规则校验 |
| 数据库 | PostgreSQL + SQLx | 主状态源、事务、迁移 |
| 终端缓存 | SQLite | 加密后的有限缓存与离线队列 |
| 桌面终端 | Tauri 2 | Windows / Linux 统一桌面壳 |
| 授权策略 | Cedar-compatible Policy Adapter | Principal / Action / Resource / Context |
| WASM 扩展 | Wasmtime Component Model | 能力约束和跨平台扩展 |
| TLS | rustls | TLS 1.2 / 1.3 与 mTLS |
| 日志与追踪 | tracing | 稳定的 Rust 内部观测 API |
| 遥测出口 | OpenTelemetry 可选 | OTLP traces / metrics / logs |
| 配置 | config layer + TOML / env references | 配置值与秘密引用分离 |

组件选择通过 ADR 固化。业务内核只依赖抽象端口，避免策略引擎、存储和消息实现渗入领域对象。

### 8.2 Rust 信任边界

必须使用 Rust 实现：

```text
对象与租户边界
权限判断入口
操作注册与执行网关
事件、outbox 和审计
模块验证与能力控制
更新验证与激活
节点身份和通信
配置与秘密引用
```

可独立实现的计算工作器：

```text
Embedding / BGE-M3 inference
LLM inference
OCR / document parsing
GPU analytics
第三方专有 SDK
```

这些工作器只能通过受权 Operation 或内部 RPC 接入，无法成为权限事实源。

### 8.3 Cargo Workspace 建议

```text
GESA/
├── Cargo.toml
├── crates/
│   ├── gesa-types/
│   ├── gesa-contracts/
│   ├── gesa-kernel/
│   ├── gesa-object/
│   ├── gesa-org/
│   ├── gesa-policy/
│   ├── gesa-operation/
│   ├── gesa-flow/
│   ├── gesa-event/
│   ├── gesa-audit/
│   ├── gesa-module-sdk/
│   ├── gesa-node/
│   ├── gesa-update/
│   ├── gesa-storage/
│   └── gesa-observability/
├── services/
│   ├── enterprise-server/
│   ├── control-plane/
│   └── worker/
├── apps/
│   └── desktop/
├── modules/
│   ├── workspace/
│   ├── task/
│   └── feedback/
├── plugins/
├── api/
│   ├── openapi/
│   ├── protobuf/
│   └── schemas/
├── migrations/
│   └── postgres/
├── deploy/
│   ├── linux/
│   ├── windows/
│   └── container/
├── config/
├── tests/
└── docs/
```

依赖方向：

```text
types / contracts
        ↓
kernel ports
        ↓
domain modules
        ↓
application services
        ↓
API / storage / transport adapters
```

领域 crate 不直接依赖 Axum、SQLx、Tauri 或中央服务实现。

---

## 9. 核心运行链路

### 9.1 写入链路

```text
Terminal / API / Agent
  ↓
Authenticate session or workload identity
  ↓
Normalize MessageEnvelope
  ↓
Validate schema and idempotency
  ↓
Resolve Object / Organization context
  ↓
Policy decision
  ↓
Risk gate / approval
  ↓
Invoke registered Operation
  ↓
PostgreSQL transaction:
  - update current state
  - append domain event
  - append audit entry
  - write outbox
  ↓
Commit
  ↓
Publish integration event
  ↓
Update projections / notify terminal
```

状态变化只能通过注册 Operation 进入。数据库管理工具保留为受控运维通道，并产生独立高风险审计。

### 9.2 查询链路

```text
Query
  ↓
Authenticate
  ↓
Resolve requested resource set
  ↓
Policy pre-filter
  ↓
Read state / projection
  ↓
Field-level redaction and minimization
  ↓
Policy post-filter
  ↓
Return response + query audit metadata
```

批量查询必须把权限过滤下推到数据层，避免先读取完整敏感集合再在应用层删除。

### 9.3 命令状态机

```text
RECEIVED
  → VALIDATED
  → AUTHORIZED
  → APPROVAL_PENDING optional
  → SCHEDULED
  → EXECUTING
  → COMMITTED
  → PUBLISHED
  → COMPLETED
```

失败状态：

```text
REJECTED
DENIED
EXPIRED
CANCELLED
EXECUTION_FAILED
COMPENSATION_REQUIRED
ROLLED_BACK
```

### 9.4 事件发布

采用 Transactional Outbox：

```text
业务事务写入 state + event + audit + outbox
  ↓
publisher 领取 outbox lease
  ↓
发布 Integration Event
  ↓
consumer 写入 inbox / dedup record
  ↓
幂等处理
  ↓
ack
```

早期 outbox 由 PostgreSQL 驱动。消息量、拓扑和延迟需求达到明确阈值后，可接入 NATS 或 Kafka，领域事件契约保持稳定。

---

## 10. 中央更新、补丁与规则分发

### 10.1 Pull-first 更新

中央端提供发布和通知，企业节点主动拉取：

```text
Central Release Service
  ↓ update notification / poll
Enterprise Update Agent
  ↓ fetch signed metadata
verify trust root / channel / version / hash / signature
  ↓
download artifact
  ↓
stage
  ↓
backup / migration dry-run
  ↓
activate in maintenance slot
  ↓
health check
  ↓
commit or rollback
  ↓
report result
```

中央控制平面不获得企业节点的通用远程 Shell 能力。

### 10.2 发布安全

采用 TUF 风格的元数据与密钥层级：

```text
Offline Root Keys
Delegated Release Keys
Platform / Channel Targets
Timestamp / Snapshot Metadata
Artifact Hashes
Version and Anti-rollback Metadata
```

桌面端同时使用 Tauri 更新包签名。服务端发布包、模块、WASM 组件、Rule Pack 和 UI Extension 都进入统一制品注册表。

### 10.3 发布通道

```text
dev
canary
stable
lts
security
```

企业实例可以固定通道和维护窗口。安全发布可带 `mandatory_after`，超过期限后限制中央互联能力；企业本地核心业务保持可恢复和可导出。

### 10.4 规则包与可执行代码分离

Rule Pack 可以包含：

```text
policy rules
validation rules
privacy rules
compatibility rules
workflow definitions
UI configuration
data classification mappings
```

可执行逻辑需要进入：

```text
signed binary release
signed WASM component
signed connector package
```

Rule Pack 无权申请超出模块清单和本地企业策略的能力。

### 10.5 回滚边界

每次发布必须声明：

```text
rollback_supported
database_migration_reversible
minimum_compatible_data_version
backup_required
health_check_set
rollback_deadline
```

破坏性迁移优先采用 expand → migrate → contract 三阶段，并在旧版本兼容窗口内保留读取能力。

---

## 11. 通用 API 与隐私 API

### 11.1 三类 API

```text
Enterprise API
- 企业内部对象、操作、查询、工作流和审计接口

Central General API
- 中央提供的通用能力、目录、更新、模型、公共数据和连接器服务

Privacy API
- 涉及企业数据的目的约束、最小化、可审计跨边界调用
```

终端优先调用 Enterprise API。企业服务器根据本地策略决定是否调用中央 API。

### 11.2 Privacy API 请求

中央端只能提交已注册的隐私操作：

```text
PrivacyRequest
- request_id
- tenant_id
- registered_operation_id
- purpose
- recipient
- requested_fields
- filter_constraints
- aggregation_requirement
- minimum_group_size
- retention_period
- expires_at
- requester_signature
```

企业本地 `Privacy Gateway` 执行：

```text
验证中央身份
验证 operation 是否注册
验证 purpose
本地权限决策
应用字段白名单
应用脱敏 / 聚合 / 匿名化
检查输出大小与数据分类
生成 PrivacyReceipt
返回最小结果
```

中央端无法通过 Privacy API 发送任意 SQL、脚本或数据库路径。

### 11.3 数据外发默认值

```text
默认外发：
- 版本
- 节点健康
- 模块兼容状态
- 更新结果
- 经授权的聚合遥测

显式授权后外发：
- 诊断包
- 选定日志片段
- 业务聚合结果
- 支持会话数据

企业本地保留：
- 原始业务表
- 原始文档
- 员工与客户明细
- 密钥
- 完整审计载荷
```

所有外发拥有目的、范围、过期时间、接收方和本地回执。

---

## 12. 数据与存储设计

### 12.1 首批核心表

```text
tenants
objects
object_relations
actors
actor_sessions
org_levels
org_directions
org_cells
org_units
org_unit_cells
memberships
positions
policies
policy_versions
grants
delegation_sessions
operation_definitions
operation_requests
operation_runs
domain_events
integration_outbox
integration_inbox
audit_entries
approval_requests
modules
module_instances
module_dependencies
rule_packs
nodes
node_capabilities
releases
release_targets
update_runs
artifacts
configuration_versions
secret_references
```

### 12.2 数据边界

```text
tenant_id：
所有业务对象、关系、权限、事件和审计的第一隔离键。

module namespace：
每个模块拥有自己的表、迁移和事件命名空间。

object_id：
跨模块引用使用统一 ObjectRef，模块内部可以保留专用主键。

schema_version：
对象、事件、消息和配置都保存版本。
```

### 12.3 审计账本

`audit_entries` 至少记录：

```text
actor_principal
effective_principal
delegation_chain
operation
resource
decision
reason_codes
policy_snapshot
risk_level
input_hash
output_hash
state_diff_ref
trace_id
timestamp
previous_entry_hash
```

哈希链与周期性签名检查点用于提高篡改可检测性。原始敏感载荷通过受控引用保存，审计表默认记录摘要、哈希和字段级差异。

### 12.4 秘密管理

```text
终端：
- OS credential store
- Windows DPAPI / Credential Manager
- Linux Secret Service compatible store

服务器：
- SecretProvider trait
- production 可接 OpenBao / Vault / cloud KMS
- 数据库只保存 secret reference

配置：
- TOML / database configuration 保存非敏感值
- 环境变量只用于启动期引用
- 日志系统执行 secret redaction
```

---

## 13. 安全模型

### 13.1 威胁对象

```text
被攻陷的员工终端
恶意或越权的内部人员
越权 Agent
有漏洞的业务模块
供应链与更新服务器被攻陷
网络中间人
旧版本回滚攻击
中央控制平面泄露
工作节点逃逸
日志和诊断包泄露
```

### 13.2 核心控制

```text
Deny by default
Tenant isolation
mTLS workload identity
short-lived user sessions
explicit delegation
capability-scoped modules
signed releases and rule packs
anti-rollback metadata
transactional audit
field-level redaction
rate limits and quotas
sandboxed untrusted execution
dependency and license policy
security test gates
```

### 13.3 中央端被攻陷时的隔离目标

架构目标：

```text
1. 中央端保存的企业原始业务数据接近于零。
2. 中央端无法直接打开企业数据库。
3. 中央端无法分发未签名制品。
4. 单个在线发布密钥泄露可由离线根密钥撤销。
5. 企业本地策略可以阻止新规则扩大数据外发范围。
6. Privacy API 只执行本地已注册操作。
7. 支持数据通过限时、范围化会话提供。
```

### 13.4 供应链门禁

CI 至少运行：

```text
cargo fmt
cargo clippy
cargo test / cargo nextest
cargo audit
cargo deny
dependency license checks
SBOM generation
secret scan
container / package scan
migration tests
policy golden tests
update rollback tests
```

对协议解析器、权限上下文和模块清单增加 property test 与 fuzz test。

### 13.5 失效策略

```text
Policy Engine 无法作出决定：
- 新的受保护操作默认拒绝。

Audit 与业务事务无法共同提交：
- 状态写入失败，避免无审计变更。

Central Control Plane 离线：
- 本地业务继续。
- 更新、中央 API 和证书轮换延后。

Enterprise Server 离线：
- 终端显示加密缓存。
- 仅允许已批准的低风险离线命令排队。

Worker 失联：
- lease 到期后重试或转移。
- 外部副作用通过幂等键和补偿操作控制。
```

---

## 14. Windows 与 Linux 发布

### 14.1 服务端

Linux：

```text
gesa-enterprise-server binary
systemd unit
/etc/gesa/
/var/lib/gesa/
/var/log/gesa/
optional OCI container
```

Windows：

```text
gesa-enterprise-server.exe
Windows Service
%ProgramData%\GESA\config
%ProgramData%\GESA\data
%ProgramData%\GESA\logs
MSI / signed ZIP
```

核心逻辑不调用平台专属 shell。服务管理、路径、密钥存储和更新激活通过平台适配器实现。

### 14.2 桌面终端

Tauri 2 目标产物：

```text
Windows:
- MSI
- NSIS installer

Linux:
- AppImage
- deb
- rpm when needed
```

Tauri 权限能力文件按窗口和插件声明最小权限。文件系统、Shell、通知和更新能力分开授权。

### 14.3 CI 构建矩阵

首批目标：

```text
x86_64-pc-windows-msvc
x86_64-unknown-linux-gnu
```

后续目标：

```text
aarch64-unknown-linux-gnu
aarch64 Windows when demand exists
```

发布产物包含：

```text
binary / installer
checksums
signature
SBOM
build provenance
module manifest
migration manifest
release notes
rollback notes
```

---

## 15. 可观测性与运维

### 15.1 统一关联标识

所有关键链路携带：

```text
trace_id
correlation_id
causation_id
operation_run_id
workflow_run_id
node_id
tenant_id
```

### 15.2 指标

首批指标：

```text
API latency / error rate
authorization allow / deny / approval rate
operation duration / failure / retry
outbox lag
worker lease and queue depth
database pool and migration state
audit commit failures
update success / rollback
node heartbeat
module health
privacy API volume and denied requests
```

### 15.3 诊断包

企业管理员显式生成诊断包：

```text
selected time range
selected services
redaction profile
size limit
expiry
recipient
support case id
```

生成前展示数据类别和样例，上传后生成本地审计回执。

---

## 16. API 与契约版本

### 16.1 契约类型

```text
OpenAPI:
- 企业和中央的外部 HTTP API

Protobuf:
- 节点、工作器、控制平面内部 RPC

JSON Schema:
- MessageEnvelope payload
- ModuleManifest
- EnterpriseProfile
- Rule Pack
- Configuration
```

### 16.2 兼容规则

```text
1. 字段新增默认可选。
2. 事件字段不复用旧语义。
3. 删除字段经过 deprecation window。
4. module_api_version 使用语义化版本。
5. schema_epoch 标识需要协调迁移的结构代际。
6. Rule Pack 声明 required_kernel_range。
7. Release Service 保存完整兼容矩阵。
```

### 16.3 数据迁移

每个迁移记录：

```text
migration_id
module_id
from_version
to_version
preconditions
up
verification
rollback / compensation
estimated_duration
required_free_space
data_classification
```

---

## 17. 第一条端到端垂直切片

首版先证明基座闭环，暂缓财务、人事、CRM 等复杂业务。

### 17.1 场景

```text
1. 安装一个企业运行节点。
2. 创建 Enterprise 与一名 Human CEO。
3. 创建 Level、Direction、Cell。
4. 创建 Department，并占用多个 Cell。
5. 创建员工、岗位和 Membership。
6. 创建 Department Space 与 Document Resource。
7. 注册 resource.read 与 resource.write Operation。
8. CEO 将部门管理权限授予部门首脑。
9. 普通员工可读、不可写；部门首脑可写。
10. Windows / Linux 终端展示对象、组织图、资源与审计结果。
11. 每次操作原子写入 state、event、audit 和 outbox。
12. 中央端发布已签名 Rule Pack。
13. 企业 Update Agent 验签、预览差异、应用并回报结果。
14. 中央端离线后，企业本地操作继续。
15. EnterpriseProfile 关闭示例 Task Module 后，相关 API 与 UI 同步消失。
```

### 17.2 验收标准

```text
Build：
- Windows 与 Linux CI 通过。
- enterprise-server 与 desktop 可安装启动。

Object：
- 所有对象拥有 tenant、kind、version 和统一引用。

Organization：
- Cell 唯一占用约束有效。
- Department scope 计算可重复验证。

Authorization：
- allow、deny、require_approval 均有 reason code。
- CEO、最高权限管理器与 break-glass 权限边界可测试。

Operation：
- 未注册操作无法执行。
- 重复 idempotency_key 不产生第二次写入。

Transaction：
- state、event、audit、outbox 同步成功或同步回滚。

Module：
- profile 裁剪无需创建客户代码分支。
- 依赖缺失时模块无法激活。

Update：
- 未签名、哈希不符、版本回滚的制品被拒绝。
- 失败更新能够恢复上一可用版本。

Privacy：
- 中央端无法提交任意 SQL。
- 外发字段与目的均进入回执。

Resilience：
- 中央端不可用时本地核心操作继续。
- Policy 或 Audit 失效时受保护写入关闭。
```

---

## 18. 实施阶段

### Phase 0：契约冻结

```text
1. ObjectRef / PrincipalRef
2. MessageEnvelope
3. OperationDefinition
4. Decision
5. ModuleManifest
6. EnterpriseProfile
7. DomainEvent / AuditEntry
8. Node / Release contracts
```

### Phase 1：Rust 工作区与数据库骨架

```text
1. Cargo workspace
2. enterprise-server
3. PostgreSQL migrations
4. types / kernel / storage
5. health / config / tracing
6. Windows + Linux CI
```

### Phase 2：对象、组织、权限

```text
1. Tenant / Object registry
2. Actor / Session adapter
3. Level / Direction / Cell / Department
4. OrgScope resolver
5. Policy adapter
6. authorization audit
```

### Phase 3：Operation、事件与审计

```text
1. Operation Registry
2. Operation Gateway
3. command state machine
4. transaction + domain event + audit + outbox
5. inbox and idempotency
6. worker lease
```

### Phase 4：桌面终端

```text
1. Tauri shell
2. login / session
3. organization view
4. object and resource view
5. operation result
6. audit view
7. encrypted cache
```

### Phase 5：中央控制平面与更新

```text
1. Fleet Registry
2. Release Registry
3. Node certificates
4. signed metadata
5. update agent
6. staging / health / rollback
7. release channels
```

### Phase 6：企业剖面与模块 SDK

```text
1. ModuleManifest
2. profile resolver
3. dependency graph
4. Rule Pack
5. UI extension registry
6. WASM proof of concept
```

### Phase 7：上层能力

```text
Workflow
Workspace
Feedback / Issue
Semantic Engine
Agent Engine
first business module
```

---

## 19. 已锁定与暂缓决策

### 19.1 已锁定

```text
Rust 作为基座实现语言。
模块化单体作为首版企业运行形态。
PostgreSQL 作为生产主状态源。
Tauri 2 作为 Windows / Linux 桌面终端。
中央端定位为控制平面。
企业业务数据默认本地保存。
企业子结构通过 Profile + Module + Rule Pack 生成。
组织范围属于权限输入，最终决策由确定性引擎完成。
所有状态变化通过 Operation Gateway。
状态、事件、审计和 outbox 原子提交。
更新采用签名、拉取、分阶段激活和回滚。
低信任运行期扩展采用 WASM 或独立进程。
```

### 19.2 需要 ADR 与原型验证

```text
Cedar 作为嵌入式策略实现的最终适配方式。
内置身份认证与外部 OIDC 的默认组合。
TUF Rust 客户端的具体实现。
桌面本地缓存加密方案。
对象关系的专用表与通用关系表比例。
WASM Component Model 的宿主能力接口。
NATS / Kafka 的进入阈值。
```

### 19.3 暂缓

```text
完整微服务拆分
Kubernetes 生产基线
多区域主动-主动
全量事件溯源
大规模图数据库
自动执行高风险自进化
任意第三方插件市场
完整 BGE-M3 / Agent 运行栈
财务、人事、CRM 全业务模型
```

---

## 20. 后续文档拆分

本文件负责总体基座。实现前继续拆分：

```text
docs/developer/foundation/
├── 01-deployable-base-system-design.md
├── 02-information-flow-contract.md
├── 03-core-object-model.md
├── 04-organization-and-permission-engine.md
├── 05-module-sdk-and-enterprise-profile.md
├── 06-central-control-and-update-plane.md
├── 07-database-schema-and-migrations.md
├── 08-api-and-protocol-contracts.md
├── 09-security-threat-model.md
└── 10-deployment-and-operations.md
```

同时建立：

```text
docs/adr/
- ADR-0001 modular-monolith-first
- ADR-0002 rust-runtime
- ADR-0003 postgres-state-source
- ADR-0004 deterministic-authorization
- ADR-0005 pull-first-signed-update
- ADR-0006 profile-over-fork
- ADR-0007 wasm-runtime-extension
```

---

## 21. 一句话定义

> GESA 可部署基座是一套以信息流契约为第一性规则、以统一对象和组织拓扑描述企业、以确定性权限与原子操作约束所有行动、以企业本地数据平面承载业务、以中央控制平面分发签名模块与通用规则，并能通过企业剖面持续生长出不同企业服务子结构的 Rust 企业运行内核。
