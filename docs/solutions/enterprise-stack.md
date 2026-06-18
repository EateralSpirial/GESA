# 企业级方案：高可用、强治理、自进化组合

> Version: 0.1  
> Status: Solution Design  
> Target: 中大型企业 / 多部门 / 多服务 / 强审计 / 长期演化

---

## 1. 方案定位

企业级方案面向真实组织规模化运行。它要求高可靠、强权限隔离、审计、复杂工作流、大规模知识库、Agent 可观测和受控的项目自进化流程。

核心目标：

```text
高可靠
高可观测
强权限
强审计
知识可治理
Agent 可控制
部署可回滚
自进化有边界
```

---

## 2. 总体组合

```text
PostgreSQL HA
MinIO / Ceph
OpenSearch
ClickHouse
Kafka / NATS + Debezium
Qdrant / Milvus / Weaviate
Neo4j / NebulaGraph
Keycloak / Zitadel
OpenFGA / SpiceDB + OPA
Temporal + Flowable / Operaton
Backstage
Argo CD + Flux
OpenTofu + Ansible
Harbor + OpenBao + SOPS + Kyverno
OpenTelemetry + Prometheus + Grafana + Loki + Jaeger
LangGraph + Langfuse + Phoenix + LlamaFirewall
OpenHands + SWE-agent + Sourcegraph + Semgrep + Trivy + promptfoo + Ragas
```

---

## 3. 按层级映射

| 层级 | 组件 | 用途 |
|---|---|---|
| L1 Database | PostgreSQL HA + MinIO + OpenSearch + ClickHouse + Kafka/NATS | 状态源、对象存储、搜索、分析、事件流。 |
| L1.5 Knowledge Base | OpenSearch + Qdrant/Milvus/Weaviate + Neo4j + 文档门户 | 全文检索、语义检索、知识图谱、知识治理。 |
| L2 Object & Permission | Keycloak/Zitadel + OpenFGA/SpiceDB + OPA | 身份、对象级权限、策略决策。 |
| L3 Atomic Operation | Temporal + Operation Gateway + MCP + sandbox | 可靠操作执行、Agent 工具、隔离执行和审计。 |
| L4 Workspace | 自研 Workspace + Appsmith/Grafana + OpenProject/Plane + Cal.com | 工作空间、可视化、任务、日程、报表。 |
| L5 Feedback | Formbricks + Chatwoot/Zammad + PostHog + Langfuse/Phoenix | 用户反馈、客服、产品分析、Agent 反馈。 |
| M1 Project Control Plane | Backstage + Argo CD + Flux + OpenTofu + Harbor | 模块目录、GitOps、基础设施、制品管理。 |
| M2 Self-Evolution Engine | OpenHands + SWE-agent + Sourcegraph + Semgrep + Trivy + Eval | 外部 Coding Agent、代码搜索、测试、扫描。 |

---

## 4. L1 Database

企业级数据库层要支持：

```text
高可用
备份恢复
事件流
数据分析
搜索
审计
对象存储
跨模块数据边界
```

推荐：

| 组件 | 用途 |
|---|---|
| PostgreSQL HA | 主状态源。 |
| Debezium | 数据库变更捕获。 |
| Kafka / NATS | 事件总线。 |
| MinIO / Ceph | 对象存储。 |
| OpenSearch | 全文检索和日志搜索。 |
| ClickHouse | 大规模行为、日志和指标分析。 |

---

## 5. L1.5 Knowledge Base

企业知识库必须成为正式基础设施。

| 组件 | 用途 |
|---|---|
| OpenSearch | 全文索引。 |
| Qdrant / Milvus / Weaviate | 向量检索。 |
| Neo4j / NebulaGraph | 知识图谱和关系查询。 |
| MinIO | 文档原文和附件。 |
| PostgreSQL | 知识元数据和治理状态。 |
| LangGraph / LlamaIndex | RAG 上下文编排。 |
| promptfoo / Ragas | 知识检索质量评测。 |

治理要求：

```text
知识所有者
知识版本
知识权限
知识过期审查
知识来源
知识发布流程
Agent 检索审计
```

---

## 6. L2 Object & Permission

企业权限建议组合：

```text
Keycloak / Zitadel：身份认证、SSO、OIDC。
OpenFGA / SpiceDB：对象关系权限。
OPA：策略判断。
OpenTelemetry：权限判断链路追踪。
```

权限策略覆盖：

```text
数据访问
知识检索
工具调用
工作流节点
审批动作
部署动作
自进化候选处理
```

---

## 7. L3 Atomic Operation

企业级原子操作层要求可靠执行和隔离。

```text
Temporal：长事务、重试、可恢复执行。
Operation Gateway：统一操作入口。
MCP：Agent 工具协议。
Playwright / RPA：浏览器操作。
gVisor / Firecracker：高风险操作隔离。
OpenTelemetry + Loki：执行追踪和日志。
```

高风险操作必须经过：

```text
权限判断
人工审批
隔离执行
全链路审计
失败回滚
```

---

## 8. L4 Workspace

企业级工作空间通常需要部分自研，因为实体类型、权限、知识侧栏和秘书 Agent 交互高度定制。

可组合：

```text
自研 Workspace Portal：统一入口。
OpenProject / Plane：项目和任务。
Cal.com：预约和日程。
Appsmith：内部工具和兜底表单。
Grafana / Metabase / Superset：指标与分析。
LangGraph：秘书 Agent。
Langfuse：Agent 调用观测。
```

---

## 9. L5 Feedback

企业反馈层需要多来源统一进入 feedbacks。

```text
Formbricks：调查和用户反馈。
Chatwoot / Zammad：客服和工单。
PostHog：产品行为分析。
OpenReplay：会话回放。
Langfuse / Phoenix：Agent 反馈。
Sentry / Loki：错误和日志。
```

所有反馈最终应进入：

```text
feedbacks
evolution_candidates
knowledge_update_candidates
workflow_improvement_candidates
permission_review_candidates
```

---

## 10. M1 Project Control Plane

企业级 M1 是平台工程核心。

```text
Backstage：服务目录、模块目录、文档入口。
OpenTofu：基础设施声明。
Ansible：配置管理。
Argo CD / Flux：GitOps 部署。
Harbor：镜像和制品仓库。
OpenBao：密钥管理。
SOPS：Git 中加密配置。
Kyverno / OPA Gatekeeper：Kubernetes 策略。
Prometheus + Grafana + Loki：健康和回滚判断。
```

---

## 11. M2 Self-Evolution Engine

企业级 M2 必须工程化。

```text
OpenHands：主要 Coding Agent 平台。
SWE-agent：Issue 修复评估。
Sourcegraph / Zoekt：代码搜索。
Semgrep：静态规则扫描。
Trivy / Grype：依赖和镜像扫描。
Syft：SBOM。
pytest / Playwright：测试。
promptfoo / Ragas：LLM 和 RAG 评测。
Langfuse / Phoenix：Agent 观测。
```

M2 不直接部署，只提交 PR、变更报告和部署建议。

---

## 12. 部署形态

推荐：

```text
Kubernetes
Helm / Kustomize
Argo CD / Flux
独立 PostgreSQL HA
独立对象存储
独立日志和观测栈
独立权限服务
独立 Agent 执行环境
```

环境分层：

```text
dev
staging
production
sandbox-agent
sandbox-evolution
```

---

## 13. 优点

```text
可支撑真实组织复杂度。
权限和知识治理能力强。
工作流和操作执行可靠。
Agent 行为可追踪、可评测、可限制。
部署和回滚可治理。
长期适合自进化。
```

---

## 14. 成本和复杂度

```text
运维复杂度高。
需要平台工程能力。
需要监控、日志、权限、GitOps 经验。
需要清晰模块边界。
不适合早期直接上全套。
```

---

## 15. 建议进入条件

```text
真实多人使用超过 20-50 人。
业务对象数量持续增长。
出现复杂权限和多部门协作。
Agent 调用成本和风险需要治理。
客户数据和审计要求提高。
部署频率和回滚需求增加。
知识库规模明显扩大。
```

---

## 16. 最小完成定义

```text
核心服务高可用。
权限统一由 SSO + 对象权限 + 策略引擎治理。
知识库支持全文、语义、图谱检索和权限过滤。
所有关键操作有审计和追踪。
工作流可可靠执行和恢复。
反馈可进入结构化候选。
自进化引擎只通过 PR 和 M1 部署。
部署有 GitOps、健康检查和回滚。
```
