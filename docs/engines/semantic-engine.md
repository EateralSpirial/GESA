# GESA Semantic Engine

> Version: 0.1  
> Date: 2026-06-19  
> Status: Architecture Supplement  
> Position: Core Engine Layer / semantic memory and retrieval engine  
> Default model candidate: BGE-M3

---

## 1. 结论

Semantic Engine 是 GESA 的语义记忆与上下文召回引擎。它应与 Agent Engine 同级，作为全系统共享基础能力存在。

```text
Semantic Engine = 记忆智能
Agent Engine    = 行动智能
```

Semantic Engine 的核心职责：

```text
1. 将文档、代码、Issue、反馈、日志、PR、数据库 schema、部署记录等内容转化为可检索语义索引。
2. 为 Agent Engine、Issue Engine、Workflow Engine、Workspace Layer、Self-Evolution Engine 提供相关上下文。
3. 通过 dense / sparse / multi-vector / metadata filter / rerank 组合提升召回质量。
4. 把已解决问题、复盘、修复方案和人工 review 沉淀为可复用经验。
```

BGE-M3 可作为 Semantic Engine 的默认本地 embedding 模型候选。

---

## 2. 非职责边界

Semantic Engine 不负责：

```text
业务状态写入
权限最终裁决
审批流推进
代码修改
生产部署
财务计算
资金操作
客户数据删除
```

这些动作应由 Database Engine、Policy Engine、Workflow Engine、Agent Engine、Deployment Engine 按权限和流程执行。

Embedding 相似度只能辅助检索与排序，不能作为权限或业务事实判断依据。

---

## 3. 与其他引擎的关系

| 对象 | 关系 |
|---|---|
| Database Engine | 保存知识元数据、索引引用、检索日志、Context Pack 记录。 |
| Knowledge Base Layer | 保存知识内容、版本、来源、治理状态。 |
| Agent Engine | 调用 Semantic Engine 获取 Context Pack，再进行规划与执行。 |
| Issue / Feedback Engine | 调用 Semantic Engine 做相似 issue 检索、去重和历史根因召回。 |
| Workflow Engine | 调用 Semantic Engine 匹配流程模板、审批规则和历史阻塞案例。 |
| Policy / Permission Engine | 为 Semantic Engine 的检索结果做权限过滤。 |
| Deployment Engine | 将部署记录、健康检查、回滚记录写入语义记忆。 |
| Workspace Layer | 为真人工作空间提供相关 SOP、案例、规则和历史经验侧栏。 |

---

## 4. BGE-M3 在 Semantic Engine 中的角色

BGE-M3 的多功能能力可以映射为三类检索能力。

| BGE-M3 能力 | Semantic Engine 职责 | 适用内容 |
|---|---|---|
| dense vector | 语义召回、模糊匹配、跨语言检索 | 文档、反馈、Issue、经验、业务规则 |
| sparse lexical weights | 精确符号命中、关键词匹配 | 文件名、函数名、字段名、错误码、Issue 编号、配置项 |
| ColBERT-style multi-vector | 多条件细粒度对齐、长文档精排 | 复杂问题、长架构文档、代码影响分析、历史根因定位 |

推荐定位：

```text
dense        = 语义理解层
sparse       = 精确命中层
multi-vector = 局部对齐与精排层
```

---

## 5. 核心输入源

Semantic Engine 应索引以下内容：

```text
architecture docs
layer docs
solution docs
GitHub Issues
GitHub PRs
commit messages
code files
config files
API docs
database schema
migration files
error logs
test reports
deployment records
rollback records
customer feedback
workflow templates
permission policies
operation registry
human review comments
agent run reports
```

首批建议从仓库文档、Issue、PR、代码文件和部署日志开始。

---

## 6. 核心模块

```text
Semantic Engine
├── Source Connector
├── Document Ingestor
├── Chunking Service
├── Metadata Extractor
├── Embedding Worker
├── Dense Index
├── Sparse Index
├── Multi-vector Index
├── Hybrid Retriever
├── Reranker / Late Interaction
├── Permission Filter
├── Context Pack Builder
├── Memory Writer
├── Retrieval Logger
└── Retrieval Evaluator
```

---

## 7. 数据结构建议

### 7.1 semantic_sources

```text
semantic_sources
- id
- source_type              # doc | code | issue | pr | log | schema | deployment | feedback
- source_uri
- repository
- branch
- path
- external_id              # issue number / pr number / commit sha / deployment id
- title
- status
- created_at
- updated_at
```

### 7.2 semantic_chunks

```text
semantic_chunks
- id
- source_id
- parent_chunk_id
- chunk_type               # section | paragraph | code_block | function | issue_body | comment | log_excerpt
- title
- content
- content_hash
- language
- token_count
- source_range
- order_index
- created_at
- updated_at
```

### 7.3 semantic_embeddings

```text
semantic_embeddings
- id
- chunk_id
- model_name               # BGE-M3 / other
- model_version
- embedding_type           # dense | sparse | multi_vector
- vector_id
- vector_store
- dimension
- created_at
```

### 7.4 semantic_metadata

```text
semantic_metadata
- chunk_id
- project
- module
- domain
- risk_level
- visibility_scope
- permission_policy_id
- source_type
- tags
- entities
- created_at
```

### 7.5 context_packs

```text
context_packs
- id
- request_id
- requester_entity_id
- task_id
- issue_id
- purpose
- included_chunk_ids
- excluded_chunk_ids
- filters
- scores
- risk_notes
- created_at
```

### 7.6 retrieval_eval_cases

```text
retrieval_eval_cases
- id
- query
- expected_chunk_ids
- retrieved_chunk_ids
- recall_at_k
- precision_at_k
- mrr
- rerank_score
- missed_reason
- created_at
```

---

## 8. 入库管线

```text
Source Event
  ↓
Source Connector
  ↓
Document Ingestor
  ↓
Chunking Service
  ↓
Metadata Extractor
  ↓
Permission Binding
  ↓
Embedding Worker
  ↓
Dense / Sparse / Multi-vector Index
  ↓
Retrieval Evaluation Sampling
```

### 8.1 文档入库

```text
Markdown / PDF / HTML / Wiki
  ↓
章节切片
  ↓
保留标题层级、原文路径、行号、版本
  ↓
生成 embedding
  ↓
写入索引
```

### 8.2 代码入库

```text
代码文件
  ↓
按文件、类、函数、注释、配置块切片
  ↓
提取路径、语言、函数名、类名、import、测试关联
  ↓
生成 dense + sparse
  ↓
核心文件可选 multi-vector
```

### 8.3 Issue / PR 入库

```text
Issue / PR
  ↓
标题、正文、评论、label、状态、关联 commit
  ↓
提取模块、风险、错误码、路径、版本
  ↓
生成语义索引
  ↓
关闭后写入经验记忆
```

---

## 9. 检索管线

### 9.1 标准检索

```text
Query
  ↓
Normalize + Intent Classify
  ↓
Policy / Permission Filter
  ↓
Dense TopK
  ↓
Sparse TopK
  ↓
Merge + Deduplicate
  ↓
Rerank / Multi-vector Late Interaction
  ↓
Context Pack Builder
  ↓
Return to Caller
```

### 9.2 轻量检索

早期 MVP 可以采用：

```text
Query
  ↓
metadata filter
  ↓
dense vector search
  ↓
BM25 / sparse search
  ↓
score fusion
  ↓
TopK chunks
```

### 9.3 高价值检索

用于复杂 Issue、核心代码变更和架构分析：

```text
Query
  ↓
dense + sparse candidate recall
  ↓
multi-vector rerank
  ↓
LLM summary compression
  ↓
Context Pack
```

---

## 10. Context Pack 标准格式

Agent Engine 不应直接读取全仓库或全知识库。Semantic Engine 应生成受控 Context Pack。

```json
{
  "context_pack_id": "ctx_001",
  "purpose": "fix_issue | answer_question | plan_workflow | analyze_impact",
  "requester": {
    "entity_id": "agent_001",
    "permission_scope": "project:GESA"
  },
  "task": {
    "task_id": "task_001",
    "issue_id": "123",
    "summary": "修复财务报表导出缺失成本项"
  },
  "related_docs": [],
  "related_code": [],
  "related_issues": [],
  "related_prs": [],
  "related_schema": [],
  "related_tests": [],
  "historical_fixes": [],
  "risk_notes": [],
  "excluded_due_to_permission": [],
  "retrieval_metadata": {
    "dense_top_k": 50,
    "sparse_top_k": 50,
    "rerank_top_k": 12,
    "model": "BGE-M3"
  }
}
```

---

## 11. API 草案

```text
POST /semantic/search
POST /semantic/search/code
POST /semantic/search/issues
POST /semantic/search/schema
POST /semantic/search/decisions
POST /semantic/context-pack/build
POST /semantic/memory/write
POST /semantic/reindex/source
POST /semantic/eval/run
GET  /semantic/context-pack/{id}
GET  /semantic/chunk/{id}
```

### 11.1 /semantic/search

```json
{
  "query": "权限层数据库字段迁移导致登录失败",
  "filters": {
    "project": "GESA",
    "source_type": ["doc", "issue", "code"],
    "risk_level": ["low", "medium", "high"]
  },
  "retrieval": {
    "dense": true,
    "sparse": true,
    "multi_vector_rerank": false,
    "top_k": 20
  }
}
```

### 11.2 /semantic/context-pack/build

```json
{
  "purpose": "fix_issue",
  "issue_id": 123,
  "task_summary": "部署后登录失败",
  "scope": {
    "project": "GESA",
    "modules": ["auth", "database", "deployment"]
  },
  "risk_level": "high",
  "max_tokens": 12000
}
```

---

## 12. 权限过滤原则

Semantic Engine 的检索必须先做权限过滤或后置强过滤。

```text
检索请求进入
  ↓
识别 requester_entity_id
  ↓
读取 permission policy
  ↓
限制可访问 source / chunk / tag / module
  ↓
执行检索
  ↓
再次过滤结果
  ↓
记录 retrieval log
```

禁止：

```text
先检索全库，再把敏感内容摘要给无权限 Agent。
```

允许：

```text
无权限内容只返回存在性提示或完全隐藏，具体由 Policy Engine 决定。
```

---

## 13. 与自进化闭环的融合

```text
Issue Created
  ↓
Issue Engine records issue
  ↓
Semantic Engine retrieves similar issues and historical fixes
  ↓
Context Pack is generated
  ↓
Agent Engine plans and executes
  ↓
PR is created
  ↓
CI / Review / Deployment
  ↓
Result is written back to Issue and Knowledge Base
  ↓
Semantic Engine re-indexes the new experience
```

关闭 Issue 后，应生成经验记录：

```text
problem_summary
root_cause
fix_summary
affected_files
test_result
deployment_result
review_notes
future_prevention
```

---

## 14. BGE-M3 使用策略

### Phase 1：dense only

用于：

```text
架构文档
设计记录
聊天历史
技术资料
用户手册
```

### Phase 2：dense + sparse

用于：

```text
代码文件
配置文件
Issue
数据库 schema
API 文档
报错日志
```

这是 GESA 的主力阶段。

### Phase 3：multi-vector 精排

用于：

```text
复杂 Issue
核心架构文档
长文档问答
代码影响分析
历史修复经验
多条件检索
```

### Phase 4：加入专用 reranker 与 eval

```text
BGE-M3 recall
  ↓
reranker / late interaction
  ↓
retrieval eval
  ↓
LLM answer / agent action
```

---

## 15. 检索质量评估

Semantic Engine 必须可评估。

核心指标：

```text
recall@k
precision@k
MRR
nDCG
context_pack_usefulness
agent_success_rate_after_context
missed_critical_context_rate
permission_leak_rate
```

评估集来源：

```text
历史 Issue
人工标注问题
失败 Agent run
PR review 反馈
部署事故复盘
文档问答样例
```

---

## 16. 默认开源组件候选

| 能力 | 轻量方案 | MVP 方案 | 企业级方案 |
|---|---|---|---|
| embedding model | BGE-M3 本地运行 | BGE-M3 + reranker | BGE-M3 / 多模型路由 |
| dense vector | sqlite-vec / Chroma | Qdrant / pgvector | Qdrant / Milvus / Weaviate |
| sparse search | SQLite FTS5 / BM25 | Meilisearch / Typesense | OpenSearch |
| multi-vector | 暂缓 | Qdrant multivector / ColBERT service | Milvus / 专用 late interaction 服务 |
| metadata | SQLite | PostgreSQL | PostgreSQL + OpenFGA |
| eval | promptfoo | Ragas / Phoenix | Ragas + Phoenix + Langfuse |

---

## 17. 最终判断

Semantic Engine 是 GESA 的基础引擎，不是 Knowledge Base Layer 的普通子模块。

最终架构判断：

```text
Knowledge Base Layer 保存知识。
Semantic Engine 让知识可被机器召回、连接、评估和复用。
Agent Engine 基于召回上下文进行规划和行动。
```

如果 GESA 要成为自进化企业服务系统，Semantic Engine 必须与 Agent Engine 同级建设。否则系统会有行动能力，但缺少稳定、可治理、可复用的长期记忆能力。
