# L1.5 Knowledge Base Layer 知识库层

> Version: 0.1  
> Status: Layer Design  
> Position: Runtime Plane / Foundation 2

---

## 1. 定位

知识库层是 GESA 的第二基础，位于数据库层之上、对象与权限层之下。

数据库层保存企业事实；知识库层保存企业认知。

```text
数据库层：发生了什么？现在是什么状态？
知识库层：这件事意味着什么？应该如何理解、复用和改进？
```

知识库层为真人、AI Agent、工作流、自进化引擎提供可检索、可治理、可引用、可版本化的知识结构。

---

## 2. 核心职责

```text
1. 保存项目文档、业务规则、操作手册、流程说明、FAQ、案例和复盘经验。
2. 将文档切分为可检索、可引用、可权限控制的知识块。
3. 建立全文索引、向量索引、标签索引、实体索引和知识关系。
4. 为秘书 Agent、Coding Agent、工作流和可视化空间提供上下文。
5. 将反馈、日志、失败、周复盘沉淀为知识更新候选。
6. 支持知识版本、所有者、可信度、适用范围和过期审查。
7. 防止 Agent 通过 RAG 绕过权限边界。
```

---

## 3. 数据库与知识库分工

| 维度 | Database Layer | Knowledge Base Layer |
|---|---|---|
| 核心对象 | 状态、事件、任务、权限、日志 | 文档、规则、经验、知识块、知识图谱 |
| 主要查询 | SQL / API / Event Query | 全文检索 / 语义检索 / 图谱检索 / RAG |
| 主要使用者 | 系统模块、工作流、Agent | 真人、秘书 Agent、自进化引擎、培训系统 |
| 典型内容 | task.status = done | 为什么这个任务这样做，下次如何做更好 |
| 更新方式 | 业务操作写入 | 人工编辑、Agent 总结、反馈沉淀、文档导入 |

---

## 4. 模块拆分

```text
Knowledge Document Module
- 文档创建
- 文档导入
- 文档版本
- 文档归档
- 文档所有者

Knowledge Chunk Module
- 文档切片
- 章节层级
- 引用范围
- 原文定位
- 内容哈希

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
- 审批状态
- 可信度
- 过期时间
- 适用范围
- 权限策略

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

| 知识类型 | 示例 | 主要用途 |
|---|---|---|
| 架构知识 | 分层、模块边界、schema 设计 | 自进化引擎、开发者 |
| 业务知识 | 报价规则、客户分类、售后流程 | 业务工作流、秘书 Agent |
| 操作知识 | 如何创建任务、如何审批、如何部署 | 真人和 Agent 执行操作 |
| 权限知识 | 角色权限、高风险操作清单 | 权限配置和审计 |
| 工作流知识 | 报销、采购、客户跟进流程 | 工作流生成和修改 |
| 经验知识 | 常见失败、复盘结论、处理建议 | 自进化和错误规避 |
| 客户知识 | 客户偏好、历史问题、交付注意事项 | 销售、客服、交付 Agent |
| 项目知识 | Issue、PR、变更记录、测试报告 | Coding Agent 和安装管理层 |
| 培训知识 | 新员工教程、岗位 SOP、FAQ | 真人工作空间 |

---

## 6. 核心 Schema

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

## 7. 知识生命周期

```text
导入 / 创建
  ↓
切片
  ↓
打标签
  ↓
建立索引
  ↓
权限绑定
  ↓
人工或 Agent 审查
  ↓
发布为正式知识
  ↓
被工作空间 / Agent / 自进化引擎检索
  ↓
反馈和复盘触发更新候选
  ↓
版本升级或归档
```

---

## 8. 检索流程

```text
请求者提交 query
  ↓
识别 requester_entity_id
  ↓
权限层过滤可访问知识范围
  ↓
全文检索 + 语义检索 + 图谱检索
  ↓
去重、排序、重排
  ↓
组装上下文
  ↓
返回引用、来源、可信度、版本
  ↓
写入 retrieval log
```

关键要求：

```text
所有 RAG 结果必须带引用。
所有 Agent 检索必须经过权限过滤。
正式知识与草稿知识必须区分。
知识过期后不能直接进入高风险操作上下文。
```

---

## 9. 与其他层关系

| 层级 | 关系 |
|---|---|
| L1 Database | 保存知识元数据、版本、索引引用、审计记录。 |
| L2 Object & Permission | 知识也是对象，必须有读写、引用、发布权限。 |
| L3 Atomic Operation | 提供 search_knowledge、update_knowledge、approve_knowledge 等原子操作。 |
| L4 Workspace | 每个实体工作空间展示相关 SOP、案例、规则、风险知识。 |
| L5 Feedback | 反馈形成知识更新候选。 |
| M1 Project Control Plane | 读取模块文档、安装规则、升级规则、回滚规则。 |
| M2 Self-Evolution Engine | 读取架构文档、Issue 归因、测试报告、历史 PR 经验。 |

---

## 10. 开源方案

| 功能 | 轻量级 | MVP | 企业级 |
|---|---|---|---|
| 文档库 | Markdown + Git / BookStack | Outline / Wiki.js / Docmost | 自研知识门户 + SSO |
| 元数据 | SQLite | PostgreSQL | PostgreSQL HA |
| 全文检索 | SQLite FTS5 / Meilisearch | Meilisearch / Typesense | OpenSearch |
| 向量检索 | sqlite-vec / Chroma | pgvector / Qdrant | Qdrant / Milvus / Weaviate |
| 图谱 | SQLite relation tables | Neo4j Community / Kùzu | Neo4j / NebulaGraph |
| 文档解析 | Pandoc / MarkItDown | Unstructured / Tika | Unstructured + Tika + Docling |
| RAG 编排 | LangGraph / LlamaIndex | LangGraph + LlamaIndex | LangGraph + LlamaIndex + Eval |
| 治理 | Git PR / 简单审批 | Directus Workflow | SSO + OpenFGA + OPA + 审计 |
| 评测 | promptfoo | promptfoo + Ragas | promptfoo + Ragas + Langfuse + Phoenix |

---

## 11. MVP 实现顺序

```text
1. docs/ 目录作为人类可读知识源。
2. knowledge_documents 保存文档元数据。
3. knowledge_chunks 保存切片。
4. SQLite FTS5 或 Meilisearch 建全文索引。
5. sqlite-vec / pgvector / Qdrant 建向量索引。
6. Casbin / OpenFGA 做权限过滤。
7. LangGraph / LlamaIndex 做检索上下文组装。
8. feedbacks 转 knowledge_update_candidates。
9. 知识更新走人工审批。
10. Self-Evolution Engine 读取正式知识库。
```

---

## 12. 风险

| 风险 | 表现 | 对策 |
|---|---|---|
| 知识污染 | Agent 总结错误进入正式知识库 | 草稿区、候选区、审批区、正式区分离 |
| RAG 越权 | Agent 检索到无权限文档 | 检索前权限过滤，检索后引用审计 |
| 知识过期 | Agent 使用旧 SOP | expires_at、review_required、版本提示 |
| 文档不可读 | 只面向向量库，不面向人 | Human-readable first，Markdown/文档树优先 |
| 来源丢失 | 无法判断可信度 | 所有知识必须带 provenance |

---

## 13. 完成定义

知识库层完成的最低标准：

```text
文档可创建、导入、版本化。
文档可切片并带来源引用。
知识块可全文检索和语义检索。
检索经过权限过滤。
反馈可转为知识更新候选。
知识更新有审批和发布状态。
Agent 使用知识会记录 retrieval log。
自进化引擎能读取正式架构知识。
```
