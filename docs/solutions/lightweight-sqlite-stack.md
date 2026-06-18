# 轻量级方案：SQLite 本地优先组合

> Version: 0.1  
> Status: Solution Design  
> Target: 单机 / 小团队 / 客户本地私有化试点 / 早期 MVP

---

## 1. 方案定位

轻量级方案追求：

```text
少服务
少中间件
少运维
文件级可备份
Docker Compose 可启动
普通服务器可运行
客户本地可部署
后期可平滑迁移
```

它适合验证 GESA 的核心抽象：Entity、Knowledge、Operation、Permission、Workflow、Feedback。

---

## 2. 总体组合

```text
SQLite
SQLite FTS5
sqlite-vec / Chroma
Markdown + Git
PocketBase / FastAPI + SQLModel
Casbin
LangGraph
RQ / APScheduler
Playwright + MCP SDK
Directus / Appsmith Lite / NiceGUI
Vikunja / Kanboard
Fider / GitHub Issues / Gitea Issues
DuckDB
Docker Compose + Makefile
Aider / Goose / OpenHands 单机模式
```

---

## 3. 按层级映射

| 层级 | 组件 | 用途 |
|---|---|---|
| L1 Database | SQLite + DuckDB + 本地文件目录 | SQLite 保存状态；DuckDB 做本地分析；文件目录保存附件和导出。 |
| L1.5 Knowledge Base | Markdown + Git + SQLite FTS5 + sqlite-vec / Chroma | 文档可读、可检索、可嵌入、可版本化。 |
| L2 Object & Permission | PocketBase / FastAPI + SQLModel + Casbin | 对象管理、API、认证和轻量权限。 |
| L3 Atomic Operation | Playwright + MCP SDK + RQ | 浏览器操作、Agent 工具、后台任务。 |
| L4 Workspace | NiceGUI / Directus / Appsmith Lite + Vikunja / Kanboard + APScheduler | 工作空间、表单、任务看板、定时调度。 |
| L5 Feedback | Fider + GitHub Issues / Gitea Issues | 功能反馈、Bug、进化候选承接。 |
| M1 Project Control Plane | Docker Compose + Makefile + shell scripts | 安装、备份、迁移、升级、回滚。 |
| M2 Self-Evolution Engine | Aider / Goose / OpenHands 单机模式 | 外部 Coding Agent 读取 Issue 和日志生成补丁。 |

---

## 4. 推荐目录结构

```text
gesa/
  docs/
    architecture.md
    layers/
    solutions/
    rules/
  data/
    gesa.db
    knowledge.db
    files/
    vector_store/
    backups/
  services/
    api/
    workspace-ui/
    workers/
    agents/
  scripts/
    install.sh
    backup.sh
    restore.sh
    migrate.sh
    upgrade.sh
  docker-compose.yml
  Makefile
  .env.example
```

---

## 5. L1 Database 实现

```text
gesa.db
- entities
- permissions
- operations
- operation_logs
- tasks
- workflows
- workflow_runs
- calendar_items
- feedbacks
- evolution_candidates
- audit_logs
- module_registry
```

SQLite 设置建议：

```sql
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA foreign_keys=ON;
PRAGMA busy_timeout=5000;
```

DuckDB 用于：

```text
操作日志统计
反馈聚类前的数据整理
周报分析
CSV/Parquet 导出分析
```

---

## 6. L1.5 Knowledge Base 实现

```text
docs/                         人类可读知识
knowledge.db                  知识元数据
knowledge_chunks              文档切片
knowledge_relations           知识关系
knowledge_fts                 SQLite FTS5 全文索引
vector_store/                 Chroma 或 sqlite-vec
issues/                       知识更新候选
```

关键策略：

```text
Human-readable first：Markdown/Git 优先。
所有知识必须带 source_uri。
正式知识和候选知识分离。
Agent 检索必须记录 retrieval log。
```

---

## 7. L2 Object & Permission 实现

两种路线：

### 路线 A：PocketBase 优先

```text
优点：单文件、内置 SQLite、Auth、Realtime、管理 UI。
缺点：复杂业务权限和工作流后期需要拆出。
```

### 路线 B：FastAPI + SQLModel 优先

```text
优点：Python 原生，易于和 Agent / Operation / LangGraph 集成。
缺点：后台 UI 需要额外做。
```

权限：

```text
Casbin
- RBAC
- ABAC
- 简单 Subject-Object-Action 判断
```

---

## 8. L3 Atomic Operation 实现

第一批操作：

```text
create_entity
update_entity
create_task
update_task_status
search_knowledge
create_feedback
create_github_issue
run_playwright_action
read_file
write_file
```

执行组件：

```text
FastAPI Operation Gateway
MCP SDK 工具描述
RQ 后台任务
Playwright 浏览器引擎
operation_logs 审计
```

---

## 9. L4 Workspace 实现

推荐两段式：

```text
第一阶段：Directus / NiceGUI 直接做实体、任务、反馈、知识管理页面。
第二阶段：定制 Human Workspace 和 Secretary Workspace。
```

轻量工作空间需要最少 6 个页面：

```text
我的周计划
我的任务池
我的审批
我的风险
知识侧栏
秘书对话框
```

---

## 10. L5 Feedback 实现

```text
Fider：功能请求和投票。
GitHub Issues / Gitea Issues：Bug、进化候选、修复跟踪。
feedbacks 表：结构化反馈源。
```

反馈分流：

```text
Bug → Issue
知识缺失 → knowledge_update_candidate
流程低效 → workflow_improvement_candidate
系统性问题 → evolution_candidate
```

---

## 11. M1 Project Control Plane 实现

```text
Makefile
- make install
- make up
- make down
- make backup
- make restore
- make migrate
- make upgrade
- make test

docker-compose.yml
- api
- worker
- ui
- pocketbase/directus
- fider/gitea optional
```

模块注册：

```text
docs/module-registry.yaml
```

增量规则：

```text
docs/rules/add-operation.md
docs/rules/add-workflow.md
docs/rules/add-agent.md
docs/rules/add-knowledge.md
docs/rules/add-schema.md
```

---

## 12. M2 Self-Evolution Engine 实现

```text
Aider：适合快速本地 pair programming。
Goose：适合本地 Agent 工作流。
OpenHands：适合更完整自动修复场景。
```

轻量阶段规则：

```text
M2 只读 docs、issues、logs、tests。
M2 只提交 patch 或 PR。
M2 不直接部署。
M2 不修改自身核心规则。
```

---

## 13. 部署方式

```bash
make install
make up
make migrate
make seed
make backup
```

单机资源建议：

```text
CPU：2-4 核
内存：4-8 GB
磁盘：20-100 GB
系统：Ubuntu Server / Debian / 本地 Linux
```

---

## 14. 优点

```text
部署简单
可离线运行
成本低
适合客户本地私有化
数据文件易备份
方便快速试错
可作为未来企业级版本的种子系统
```

---

## 15. 边界

```text
不适合高并发多人写入。
不适合复杂多租户权限。
不适合大量 Agent 并发执行。
不适合强合规审计。
不适合跨区域高可用。
不适合大规模事件流。
```

---

## 16. 升级路径

```text
SQLite → PostgreSQL
PocketBase → FastAPI + PostgreSQL / Directus
Casbin → OpenFGA + OPA
RQ / APScheduler → Temporal / NATS
本地文件目录 → MinIO
SQLite FTS5 → Meilisearch / OpenSearch
sqlite-vec / Chroma → Qdrant / Milvus
Vikunja / Kanboard → Plane / OpenProject
Docker Compose → Kubernetes + Argo CD
Aider 单机 → OpenHands + CI/CD + GitOps
```

---

## 17. 最小完成定义

```text
单机可启动。
SQLite 保存核心状态。
Markdown/Git 保存知识。
Casbin 控制权限。
Operation Gateway 可执行基础操作。
Workspace UI 可管理任务和反馈。
Issue 可承接进化候选。
Aider/OpenHands 可读取项目并生成补丁。
```
