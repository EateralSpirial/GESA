# GESA 知识库层设计

> Version: 0.1  
> Date: 2026-06-19  
> Status: Architecture Supplement  
> Position: L1.5 Knowledge Base Layer

---

## 1. 结论

GESA 当前开源方案组合中已经包含知识库相关组件，但这些组件被分散放置在不同层级中：

- L1 Database Layer 中包含向量记忆、RAG、文档索引、对象存储。
- L4 Workspace Layer 中包含文档型工作空间、项目协作和人工操作界面。
- L5 Feedback Layer 中包含知识库、FAQ、问题分流和反馈沉淀。
- M2 Self-Evolution Engine 中包含代码上下文、项目文档、Issue、日志和测试报告。

这个设计还不够明确。GESA 的第二大基础应当是一个独立的 **Knowledge Base Layer**。

数据库层负责保存企业事实状态；知识库层负责保存企业可理解、可复用、可检索、可治理的知识结构。

---

## 2. 层级定位

建议将知识库层放在数据库层之上、对象与权限层之下。

```text
┌──────────────────────────────────────────────┐
│  L5. Feedback Layer                           │
└──────────────────────▲───────────────────────┘
                       │
┌──────────────────────┴───────────────────────┐
│  L4. Workspace Layer                          │
└──────────────────────▲───────────────────────┘
                       │
┌──────────────────────┴───────────────────────┐
│  L3. Atomic Operation Layer                   │
└──────────────────────▲───────────────────────┘
                       │
┌──────────────────────┴───────────────────────┐
│  L2. Object & Permission Layer                │
└──────────────────────▲───────────────────────┘
                       │
┌──────────────────────┴───────────────────────┐
│  L1.5 Knowledge Base Layer                    │
│  文档 / 规则 / 经验 / 语义索引 / 知识图谱       │
└──────────────────────▲───────────────────────┘
                       │
┌──────────────────────┴───────────────────────┐
│  L1. Database Layer                           │
│  状态 / 事件 / 审计 / 文件 / 指标              │
└──────────────────────────────────────────────┘
```

知识库层依赖数据库层保存元数据、索引和版本记录，但它承担独立职责：组织知识、解释知识、沉淀知识、供人和 Agent 共同调用。

---

## 3. 数据库与知识库的分工

| 维度 | 数据库层 | 知识库层 |
|---|---|---|
| 核心对象 | 状态、事件、任务、权限、日志 | 文档、规则、流程说明、经验、案例、FAQ、知识图谱 |
| 主要问题 | 发生了什么，现在是什么状态 | 这件事意味着什么，应该怎么理解和处理 |
| 结构 | 表、字段、关系、事件流 | 文档树、标签、语义块、引用、知识节点、版本 |
| 查询方式 | SQL / API / event query | 全文检索 / 语义检索 / 图谱检索 / RAG |
| 更新方式 | 业务操作写入 | 人工编辑、Agent 总结、反馈沉淀、文档导入 |
| 使用者 | 系统模块、工作流、Agent | 真人、秘书 Agent、自进化引擎、培训系统 |
| 典型内容 | task.status = done | 为什么这个任务这样做、下次怎么做更好 |

数据库层是企业现实的状态源。  
知识库层是企业认知的结构源。

---

## 4. 知识库层模块

Knowledge Base Layer 建议拆成以下模块：

```text
Knowledge Document Module
- 项目文档
- 操作手册
- 流程说明
- 业务规则
- 培训材料
- 架构文档

Knowledge Chunk Module
- 文档切片
- 语义块
- 引用范围
- 章节层级
- 原文来源

Knowledge Index Module
- 全文索引
- 向量索引
- 标签索引
- 实体索引
- 时间索引

Knowledge Graph Module
- 实体关系
- 概念关系
- 规则依赖
- 工作流依赖
- 模块依赖

Knowledge Governance Module
- 知识版本
- 所有者
- 审批状态
- 可信度
- 过期时间
- 适用范围

Knowledge Retrieval Module
- 关键词检索
- 语义检索
- 图谱检索
- 权限过滤
- 上下文组装

Knowledge Evolution Module
- 反馈沉淀
- 周复盘沉淀
- Agent 失败经验沉淀
- 流程改进沉淀
- 文档更新候选
```

---

## 5. 知识类型

GESA 的知识库不应只保存普通文档，还应保存多种企业知识对象。

| 知识类型 | 示例 | 主要用途 |
|---|---|---|
| 架构知识 | GESA 分层、模块边界、schema 设计 | 供自进化引擎和开发者理解项目 |
| 业务知识 | 报价规则、客户分类、售后流程 | 供工作流和秘书 Agent 执行业务 |
| 操作知识 | 如何创建任务、如何审批、如何部署 | 供真人和 Agent 执行基础操作 |
| 权限知识 | 什么角色能做什么、高风险操作清单 | 辅助权限配置与审计 |
| 工作流知识 | 报销流程、采购流程、客户跟进流程 | 供工作空间生成和修改工作流 |
| 经验知识 | 常见失败、复盘结论、处理建议 | 支撑自进化和避免重复错误 |
| 客户知识 | 客户偏好、历史问题、交付注意事项 | 支撑销售、客服、交付 Agent |
| 项目知识 | Issue、PR、变更记录、测试报告 | 支撑 Coding Agent 和安装管理层 |
| 培训知识 | 新员工教程、岗位 SOP、FAQ | 支撑真人可视化工作空间 |

---

## 6. 推荐核心 Schema

知识库层应拥有独立核心 schema。

```text
knowledge_documents
- id
- title
- document_type
- owner_entity_id
- source_type
- source_uri
- status
- version
- created_at
- updated_at

knowledge_chunks
- id
- document_id
- parent_chunk_id
- chunk_type
- title
- content
- content_hash
- order_index
- token_count
- source_range
- created_at
- updated_at

knowledge_embeddings
- id
- chunk_id
- embedding_model
- vector_id
- embedding_version
- created_at

knowledge_tags
- id
- name
- tag_type
- description

knowledge_chunk_tags
- chunk_id
- tag_id

knowledge_relations
- id
- source_id
- source_type
- target_id
- target_type
- relation_type
- confidence
- created_by
- created_at

knowledge_policies
- id
- knowledge_id
- visibility_scope
- permission_policy_id
- review_required
- expires_at

knowledge_retrieval_logs
- id
- requester_entity_id
- query
- retrieved_chunk_ids
- used_by_agent_id
- used_in_task_id
- created_at

knowledge_update_candidates
- id
- source_feedback_id
- source_event_id
- target_document_id
- proposed_change
- reason
- status
- created_at
```

---

## 7. 知识库与其他层级的关系

### 7.1 与数据库层

数据库层保存知识库的结构化元数据、版本记录、索引引用和审计记录。  
知识内容可以保存在数据库、对象存储、Git 仓库或外部文档系统中。

### 7.2 与对象权限层

知识也是对象。  
每个知识文档、知识块、知识图谱节点都应具备权限边界。

```text
谁可以读
谁可以编辑
谁可以引用
谁可以让 Agent 使用
谁可以将知识发布为正式规则
```

### 7.3 与原子操作层

知识库应提供基础操作：

```text
读取知识
搜索知识
创建知识文档
更新知识文档
创建知识块
绑定标签
建立知识关系
提交知识更新候选
批准知识更新
```

这些操作都应进入 Operation Registry。

### 7.4 与工作空间层

每个实体工作空间都应该有知识侧栏：

```text
相关 SOP
相关客户记录
相关项目文档
相关历史案例
相关风险规则
相关复盘经验
```

秘书 Agent 在生成周计划、任务建议、风险提示和工作流时，应优先从知识库取上下文。

### 7.5 与反馈层

反馈层是知识库进化的重要来源。

```text
用户反馈 → FAQ 候选
Agent 失败 → 经验知识候选
工作流阻塞 → 流程文档更新候选
权限冲突 → 权限知识更新候选
部署异常 → 运维知识更新候选
```

### 7.6 与项目控制平面

项目控制平面需要读取知识库中的：

```text
模块文档
安装规则
升级规则
回滚规则
增量规则
架构边界
```

### 7.7 与自进化引擎

Self-Evolution Engine 的上下文主要来自知识库。

它需要读取：

```text
架构文档
模块说明
增量规则
Issue 归因
操作日志总结
测试报告
历史 PR 经验
部署失败案例
```

自进化引擎输出的变更报告、复盘和新规则，也应该回写知识库。

---

## 8. 开源方案映射

| 功能 | 轻量方案 | MVP 方案 | 企业级方案 |
|---|---|---|---|
| 文档知识库 | Markdown + Git / BookStack / Wiki.js | Outline / Docmost / Wiki.js | Confluence 替代栈：Outline + OpenSearch + MinIO |
| 数据后台 | PocketBase / Directus | Directus / NocoDB | Directus + PostgreSQL + SSO |
| 全文检索 | SQLite FTS5 / Meilisearch | Meilisearch / Typesense | OpenSearch |
| 向量检索 | sqlite-vec / Chroma | pgvector / Qdrant | Qdrant / Milvus / Weaviate |
| 知识图谱 | SQLite relation tables | Neo4j Community / Kùzu | Neo4j / NebulaGraph / GraphDB 类方案 |
| 文档解析 | MarkItDown / Pandoc | Unstructured / Apache Tika | Unstructured + Tika + Docling |
| RAG 编排 | LangGraph / LlamaIndex | LangGraph + LlamaIndex | LangGraph + LlamaIndex + 专用评测链路 |
| 知识治理 | Git PR / 简单审批表 | Directus Workflow / GitHub PR | SSO + 权限策略 + 审计 + 发布流程 |
| 知识评测 | promptfoo | promptfoo + Ragas | promptfoo + Ragas + Langfuse / Phoenix |

---

## 9. 轻量级知识库组合

```text
SQLite
SQLite FTS5
sqlite-vec / Chroma
Markdown + Git
BookStack / Wiki.js / Docmost
Pandoc / MarkItDown
LangGraph / LlamaIndex
Casbin
GitHub Issues / Gitea Issues
Aider / Goose
```

### 轻量级设计

```text
docs/                    人类可读文档
knowledge.db             SQLite 知识元数据
knowledge_chunks         文档切片表
knowledge_relations      知识关系表
knowledge_fts            SQLite FTS5 全文索引
vector_store/            本地向量库或 sqlite-vec
issues/                  反馈与知识更新候选
```

适合：

```text
单机部署
小团队
项目早期
客户本地试点
知识量中小
不需要复杂多租户权限
```

---

## 10. MVP 知识库组合

```text
PostgreSQL
pgvector / Qdrant
Meilisearch / Typesense
Directus
Outline / Wiki.js / Docmost
Unstructured / Tika
LangGraph / LlamaIndex
Casbin / OpenFGA
Langfuse / Phoenix
GitHub Issues / Gitea Issues
```

适合：

```text
小型企业
多用户协作
较复杂权限
需要 RAG
需要 Agent 可观测
需要知识版本治理
```

---

## 11. 企业级知识库组合

```text
PostgreSQL
MinIO
OpenSearch
Qdrant / Milvus / Weaviate
Neo4j / NebulaGraph
Directus
Outline / Wiki.js / 自研知识门户
Unstructured + Apache Tika + Docling
OpenFGA + OPA
LangGraph + LlamaIndex
Langfuse + Phoenix + promptfoo + Ragas
Argo CD / GitOps 文档发布
```

适合：

```text
复杂组织
多部门知识治理
强权限隔离
大量文档
大量 Agent 调用
需要审计与合规
需要知识发布流程
```

---

## 12. 关键设计原则

### 12.1 Knowledge as Object

知识也应作为对象管理。

```text
知识文档是 Entity
知识块是 Entity
知识图谱节点是 Entity
知识更新候选是 Entity
```

这样知识才能进入权限、审计、工作流和自进化闭环。

### 12.2 Knowledge with Provenance

每条知识都必须知道来源。

```text
来源文件
来源反馈
来源任务
来源会议
来源 PR
来源日志
来源 Agent 总结
```

没有来源的知识不能进入正式知识库，只能进入草稿区或候选区。

### 12.3 Human-Readable First

知识库首先要让人能读懂。  
向量检索和 RAG 是增强能力，不能替代清晰文档结构。

### 12.4 Versioned Knowledge

知识必须版本化。

```text
当前版本
历史版本
变更说明
审批记录
回滚路径
```

### 12.5 Permissioned Retrieval

Agent 检索知识时也要经过权限过滤。  
不能因为 RAG 检索绕过文档权限。

### 12.6 Feedback-to-Knowledge

反馈层不只提交代码修改候选，也要提交知识更新候选。

```text
Bug → 文档补充候选
流程阻塞 → SOP 更新候选
Agent 失败 → 经验知识候选
权限冲突 → 权限规则说明候选
部署失败 → 运维手册更新候选
```

---

## 13. 对 GESA 主架构的修正建议

GESA 基础层应改为：

```text
第一基础：Database Layer
- 保存事实、状态、事件、审计、指标。

第二基础：Knowledge Base Layer
- 保存文档、规则、经验、解释、语义索引、知识关系。
```

首批核心 Schema 也应从五个扩展为六个：

```text
Entity
Operation
Permission
Workflow
Feedback
Knowledge
```

GESA 的一句话定义也应修正为：

> GESA 是一套以数据库为企业状态源、以知识库为企业认知源、以对象权限层管理所有实体、以原子操作层提供可审计动作、以工作空间层承载秘书 Agent 和可视化协作、以反馈层形成进化输入、再由项目控制平面和外部自进化引擎驱动长期迭代的企业自进化服务架构。
