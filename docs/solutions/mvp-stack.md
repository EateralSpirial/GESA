# MVP 方案：PostgreSQL + 权限 + 工作流 + Agent 组合

> Version: 0.1  
> Status: Solution Design  
> Target: 小型企业 / 多用户协作 / 早期商业化 / 可扩展 MVP

---

## 1. 方案定位

MVP 方案用于从单机原型进入真实企业运行。它需要支持多人协作、基础权限、稳定数据库、可视化工作空间、可审计操作、知识库、反馈闭环和外部 Coding Agent 自进化。

它追求：

```text
能商用
能多人使用
能私有化部署
能支持真实工作流
能保留升级到企业级的路径
```

---

## 2. 总体组合

```text
PostgreSQL + pgvector
MinIO optional
Meilisearch / Typesense
Directus
Keycloak / Zitadel
Casbin + OpenFGA
FastAPI + MCP
Temporal + Windmill
Appsmith / ToolJet
Plane / OpenProject
Cal.com
LangGraph + Dify
Langfuse / Phoenix
Fider + GitHub Issues / Gitea Issues
Docker Compose + OpenTofu + Ansible
Aider / OpenHands
```

---

## 3. 按层级映射

| 层级 | 组件 | 用途 |
|---|---|---|
| L1 Database | PostgreSQL + pgvector + MinIO optional | 状态源、向量记忆、文件产物。 |
| L1.5 Knowledge Base | Directus + Markdown/Git + Meilisearch/Typesense + pgvector/Qdrant | 文档、知识块、全文检索、语义检索。 |
| L2 Object & Permission | Directus + Keycloak/Zitadel + Casbin/OpenFGA | 实体管理、认证、对象权限。 |
| L3 Atomic Operation | FastAPI + MCP + Temporal/Windmill + Playwright | 操作网关、工具封装、可靠执行。 |
| L4 Workspace | Appsmith/ToolJet + Plane/OpenProject + Cal.com + LangGraph/Dify | 可视化工作空间、项目任务、日程、秘书 Agent。 |
| L5 Feedback | Fider + GitHub/Gitea Issues + Langfuse/Phoenix | 用户反馈、Issue、Agent 反馈和评测。 |
| M1 Project Control Plane | Docker Compose + OpenTofu + Ansible + module registry | 安装、部署、迁移、配置。 |
| M2 Self-Evolution Engine | Aider + OpenHands + pytest + Semgrep/Trivy | 外部 Coding Agent、测试、扫描、PR。 |

---

## 4. L1 Database

PostgreSQL 是 MVP 的主状态源。

核心表：

```text
entities
knowledge_documents
knowledge_chunks
permissions
operations
operation_logs
tasks
workflows
workflow_runs
calendar_items
feedbacks
evolution_candidates
agent_profiles
module_registry
audit_logs
schema_migrations
```

建议：

```text
使用 pgvector 支撑早期语义检索。
使用 PostgreSQL JSONB 支撑扩展字段。
使用 migration 工具管理 schema。
对象文件可以先本地，后接 MinIO。
```

---

## 5. L1.5 Knowledge Base

MVP 知识库应做到：

```text
文档可读
文档可切片
知识可全文检索
知识可语义检索
知识有权限
知识有版本
知识可从反馈沉淀
```

推荐组合：

```text
Markdown + Git：架构和规则文档。
Directus：知识元数据和后台。
Meilisearch / Typesense：全文检索。
pgvector / Qdrant：语义检索。
Unstructured / Tika：文档解析。
LangGraph / LlamaIndex：RAG 上下文组装。
```

---

## 6. L2 Object & Permission

认证：

```text
Keycloak：成熟、功能完整。
Zitadel：云原生、现代化。
```

权限：

```text
Casbin：嵌入式策略判断，适合早期。
OpenFGA：对象关系权限，适合 Entity 权限图。
```

实体管理：

```text
Directus 管理 entities、profiles、relations、operation definitions。
FastAPI 负责核心业务 API。
```

---

## 7. L3 Atomic Operation

MVP 需要一个统一 Operation Gateway。

```text
FastAPI：Operation API。
MCP：Agent 工具协议。
Temporal：长事务和可靠任务。
Windmill：开发者脚本和内部自动化。
Playwright：浏览器操作。
OpenTelemetry：操作追踪。
```

第一批操作：

```text
entity.create
entity.update
task.create
task.update_status
knowledge.search
knowledge.create_candidate
workflow.start
workflow.approve
feedback.submit
github.issue.create
browser.open_and_extract
file.read
file.write
```

---

## 8. L4 Workspace

工作空间组件：

```text
Appsmith / ToolJet：内部管理台。
Plane / OpenProject：任务和项目管理。
Cal.com：预约和日程。
LangGraph：可控 Secretary Agent。
Dify：业务人员可配置 LLM 应用。
Metabase optional：指标和报表。
```

MVP 工作空间页面：

```text
实体详情页
本周计划页
任务看板
日历页
审批页
反馈页
知识侧栏
秘书 Agent 对话页
```

---

## 9. L5 Feedback

```text
Fider：产品反馈和投票。
GitHub Issues / Gitea Issues：Bug 和进化候选。
Langfuse / Phoenix：Agent 调用反馈。
Sentry optional：前后端错误追踪。
```

反馈分流：

```text
Bug → Issue
Agent Failure → agent improvement candidate
Knowledge Gap → knowledge update candidate
Workflow Blocker → workflow improvement candidate
Systemic Issue → evolution candidate
```

---

## 10. M1 Project Control Plane

MVP 阶段仍可使用 Docker Compose，但要引入更清晰的安装和配置管理。

```text
Docker Compose：本地和单服务器部署。
OpenTofu：基础设施声明。
Ansible：服务器初始化和配置。
module_registry.yaml：模块注册表。
SOPS：敏感配置加密。
Makefile：统一命令入口。
```

必要命令：

```text
make install
make up
make migrate
make seed
make backup
make restore
make test
make upgrade
```

---

## 11. M2 Self-Evolution Engine

```text
Aider：快速本地修改和人工协作。
OpenHands：自动化 issue 修复。
pytest：代码测试。
Playwright：UI 测试。
Semgrep：静态规则扫描。
Trivy：依赖和镜像扫描。
promptfoo：Agent 行为评测。
```

M2 输出必须包括：

```text
PR
变更报告
测试结果
迁移说明
回滚说明
受影响模块
知识库更新说明
```

---

## 12. 部署形态

推荐：

```text
单服务器 Docker Compose
PostgreSQL 独立 volume
MinIO optional
Directus 独立服务
FastAPI 核心服务
Worker 服务
Workspace UI 服务
```

资源建议：

```text
CPU：4-8 核
内存：8-32 GB
磁盘：100 GB+
系统：Ubuntu Server
```

---

## 13. 优点

```text
进入真实多人使用阶段。
数据库能力明显强于 SQLite。
知识库和权限可正式建模。
工作流可进入可靠执行。
Agent 可被观测和评测。
仍然可以保持中等运维复杂度。
```

---

## 14. 边界

```text
不适合大型集团多租户复杂权限。
不适合大规模事件流。
不适合跨区域高可用。
不适合大规模 Agent 并发。
不适合极强合规场景。
```

---

## 15. 升级路径

```text
PostgreSQL 单机 → PostgreSQL HA
pgvector / Qdrant → Qdrant / Milvus / Weaviate 集群
Meilisearch → OpenSearch
Casbin + OpenFGA → OpenFGA + OPA + 审计网关
Docker Compose → Kubernetes + Helm + Argo CD
Plane / Appsmith → 自研工作空间门户
Temporal 单节点 → Temporal 集群
Aider / OpenHands → CI/CD + GitOps 自进化流水线
```

---

## 16. 最小完成定义

```text
多人可登录。
实体、知识、权限、操作、工作流、反馈可用。
任务和周计划可视化。
Agent 可执行受限操作。
反馈可进入 Issue 和 evolution_candidates。
M2 可提交 PR。
M1 可部署和回滚。
```
