# L1 Database Layer 数据库层

> Version: 0.1  
> Status: Layer Design  
> Position: Runtime Plane / Foundation 1

---

## 1. 定位

数据库层是 GESA 的第一基础，负责保存企业运行系统中的事实状态、事件、审计、指标、索引引用和基础元数据。

数据库层回答的问题是：

```text
现在是什么状态？
过去发生过什么？
谁在什么时候做了什么？
哪些对象、任务、权限、流程、反馈存在？
这些事实如何被稳定查询、回滚、迁移和审计？
```

数据库层不负责解释业务含义，也不负责自主决策。解释和复用属于知识库层，调度属于工作空间层，执行属于原子操作层。

---

## 2. 核心职责

```text
1. 保存所有 Entity 的结构化状态。
2. 保存所有任务、工作流、日程、反馈、权限、Agent、模块记录。
3. 保存所有基础操作日志和审计日志。
4. 保存事件流或事件表，支撑回溯与自进化分析。
5. 保存知识库、文件、向量库、对象存储的元数据引用。
6. 支持 schema 版本迁移、备份、恢复、归档和数据质量检查。
7. 为上层模块提供一致的数据访问边界。
```

---

## 3. 边界

### 数据库层负责

```text
状态存储
事件存储
事务一致性
基础索引
审计记录
数据备份
迁移记录
结构化查询
```

### 数据库层不负责

```text
业务解释
知识沉淀
权限策略推理
Agent 编排
工作流调度
自然语言交互
代码自进化
```

---

## 4. 核心数据域

| 数据域 | 说明 | 示例表 |
|---|---|---|
| Entity State | 所有对象状态 | entities, entity_profiles, entity_relations |
| Access State | 权限和身份关联 | permissions, roles, policies, access_grants |
| Operation State | 操作定义和执行记录 | operations, operation_logs, operation_results |
| Workflow State | 工作流定义和运行记录 | workflows, workflow_runs, workflow_steps |
| Workspace State | 工作空间、看板、日程 | workspaces, calendar_items, task_views |
| Feedback State | 反馈和进化候选 | feedbacks, evolution_candidates |
| Agent State | Agent 档案、会话、成本 | agent_profiles, agent_sessions, agent_costs |
| Knowledge Metadata | 知识库元数据引用 | knowledge_documents, knowledge_chunks, embeddings |
| Artifact Metadata | 文件和产物引用 | artifacts, artifact_versions |
| Observability | 指标、日志、审计 | audit_logs, metrics, traces |
| Control Plane | 项目模块和版本 | module_registry, migrations, deployment_records |

---

## 5. 推荐核心表

```text
entities
entity_profiles
entity_relations
permissions
roles
policies
operations
operation_logs
tasks
workflows
workflow_runs
calendar_items
workspaces
feedbacks
evolution_candidates
agent_profiles
agent_sessions
knowledge_documents
knowledge_chunks
artifacts
metrics
audit_logs
module_registry
schema_migrations
deployment_records
```

---

## 6. 关键表设计

### 6.1 entities

```text
id
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

### 6.2 operation_logs

```text
id
operation_id
requester_entity_id
target_entity_id
permission_decision_id
input_json
output_json
status
error_code
error_message
started_at
finished_at
trace_id
```

### 6.3 audit_logs

```text
id
actor_entity_id
action
object_type
object_id
before_json
after_json
reason
ip_address
user_agent
created_at
```

### 6.4 schema_migrations

```text
id
version
name
checksum
applied_by
applied_at
rollback_script_uri
status
```

---

## 7. 事件模型

数据库层需要记录事实事件。

```text
event_id
entity_id
event_type
source_layer
payload_json
causation_id
correlation_id
created_by
created_at
```

事件类型示例：

```text
entity.created
task.created
task.completed
permission.granted
operation.executed
workflow.failed
feedback.submitted
knowledge.updated
module.deployed
```

早期可以用数据库事件表；中后期可以引入 NATS、Kafka、Pulsar 或 Debezium。

---

## 8. 数据访问原则

```text
1. 上层模块不直接随意读写表，应通过 Repository / Service / API 边界访问。
2. 高风险写入必须带 requester_entity_id、operation_id、trace_id。
3. 所有状态变更应产生 audit log 或 event log。
4. 所有删除优先软删除，硬删除需要单独权限。
5. 每个表应考虑 owner、tenant、scope、created_at、updated_at。
```

---

## 9. 备份与恢复

轻量阶段：

```text
SQLite 文件备份
每日压缩快照
Git 记录 schema
本地对象文件目录备份
```

MVP 阶段：

```text
PostgreSQL dump
对象存储备份
迁移脚本版本化
恢复演练
```

企业级阶段：

```text
PITR
跨节点复制
冷热分层归档
审计日志不可变存储
灾备环境
```

---

## 10. 开源方案

| 功能 | 轻量级 | MVP | 企业级 |
|---|---|---|---|
| 状态数据库 | SQLite | PostgreSQL | PostgreSQL HA / Citus |
| 数据后台 | PocketBase | Directus | Directus + SSO |
| 向量索引 | sqlite-vec / Chroma | pgvector / Qdrant | Qdrant / Milvus / Weaviate |
| 对象存储 | 本地文件目录 | MinIO | MinIO / Ceph |
| 事件流 | SQLite event table | NATS / RabbitMQ | Kafka / Pulsar |
| 分析 | DuckDB | Metabase | ClickHouse / OpenSearch |
| 观测 | operation_logs | OpenTelemetry + Prometheus | OpenTelemetry + Prometheus + Grafana + Loki |

---

## 11. MVP 实现顺序

```text
1. entities
2. operations
3. permissions
4. tasks
5. workflows
6. feedbacks
7. audit_logs
8. knowledge_documents metadata
9. module_registry
10. schema_migrations
```

---

## 12. 风险

| 风险 | 表现 | 对策 |
|---|---|---|
| 数据库变成业务大泥球 | 所有逻辑写进 SQL 和触发器 | 业务逻辑上移到服务层，数据库保存事实 |
| 审计缺失 | 无法追责和复盘 | 所有高价值操作必须写 operation_logs/audit_logs |
| schema 过早复杂 | MVP 无法启动 | 先定六个核心 schema，再增量扩展 |
| SQLite 超出边界 | 多人高并发写入阻塞 | 明确迁移到 PostgreSQL 的阈值 |
| 数据与知识混杂 | 文档、规则、状态无法区分 | 引入 L1.5 Knowledge Base Layer |

---

## 13. 完成定义

数据库层完成的最低标准：

```text
Entity 可创建、查询、更新。
Operation 可注册、执行后可记录。
Permission 可保存策略和判断结果。
Workflow/Task 可保存状态。
Feedback 可保存并转候选。
Audit log 覆盖关键操作。
Schema migration 可回放。
数据可备份和恢复。
```
