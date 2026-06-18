# GESA 开源方案调研：按层级与模块映射

> Version: 0.1  
> Date: 2026-06-18  
> Scope: GESA Runtime Plane + Meta Plane  
> 说明：本文优先列出严格开源项目；少数行业常用但采用 source-available / open-core 授权的项目会用“授权注意”标注。链接优先指向项目官网或 GitHub 仓库。  
> 目的：为 GESA 的数据库、对象权限、原子操作、工作空间、反馈、安装控制平面和自进化引擎选择可组合的开源底座。

---

## 0.1 授权注意

本文把方案分成两类：

- **严格开源优先**：Apache-2.0、MIT、BSD、GPL/AGPL、MPL 等常见 OSI 开源许可证。
- **开放源码但需审查授权**：例如 n8n 的 Sustainable Use License、MongoDB 的 SSPL、Sentry 的新授权、部分 open-core 项目。它们可以自托管或查看源码，但未必满足 OSI 开源定义。

进入 GESA 依赖树前，每个项目都应记录：

```text
license
commercial_restrictions
cloud_service_restrictions
redistribution_obligations
source_modification_obligations
enterprise_risk_level
```

---

## 0. 快速判断

GESA 不适合用一个巨型系统一次性覆盖全部层级。更稳的路线是：

```text
PostgreSQL / Supabase / Directus
  + Keycloak / OpenFGA / Casbin / OPA
  + Temporal / Windmill / n8n / Node-RED
  + Appsmith / Plane / OpenProject / Cal.com
  + LangGraph / Dify / OpenHands / Aider
  + Backstage / Argo CD / OpenTofu / Ansible
```

其中：

- **PostgreSQL** 适合作为企业状态源。
- **OpenFGA / Casbin / OPA / Keycloak** 适合作为权限与身份层。
- **Temporal / Windmill / n8n / Node-RED** 适合工作流与原子操作编排。
- **Appsmith / ToolJet / Budibase / Plane / OpenProject** 适合真人可视化工作空间。
- **LangGraph / AutoGen / CrewAI / Dify / Flowise** 适合 AI Agent 管理与秘书 Agent。
- **OpenHands / Aider / SWE-agent / Continue / Roo Code** 适合外部自进化 Coding Agent。
- **Backstage / Argo CD / Flux / OpenTofu / Ansible** 适合项目安装与部署控制平面。

---

## L1. Database Layer

数据库层负责保存企业事实状态、事件、记忆、审计和指标。GESA 的第一底座建议以 PostgreSQL 为核心，再按需要外挂对象存储、向量库、事件流和观测系统。

| # | 状态数据库 | 数据 API / 后台管理 | 事件流 / 消息队列 | 向量记忆 / RAG | 对象存储 / 产物 | 搜索 / 分析 / 指标 | 观测 / 审计 |
|---|---|---|---|---|---|---|---|
| 1 | [PostgreSQL](https://www.postgresql.org/) | [Supabase](https://github.com/supabase/supabase) | [Apache Kafka](https://github.com/apache/kafka) | [pgvector](https://github.com/pgvector/pgvector) | [MinIO](https://github.com/minio/minio) | [OpenSearch](https://github.com/opensearch-project/OpenSearch) | [OpenTelemetry](https://github.com/open-telemetry/opentelemetry-collector) |
| 2 | [MariaDB](https://github.com/MariaDB/server) | [Directus](https://github.com/directus/directus) | [NATS](https://github.com/nats-io/nats-server) | [Qdrant](https://github.com/qdrant/qdrant) | [SeaweedFS](https://github.com/seaweedfs/seaweedfs) | [ClickHouse](https://github.com/ClickHouse/ClickHouse) | [Prometheus](https://github.com/prometheus/prometheus) |
| 3 | [SQLite](https://www.sqlite.org/src/doc/trunk/README.md) | [Hasura GraphQL Engine](https://github.com/hasura/graphql-engine) | [RabbitMQ](https://github.com/rabbitmq/rabbitmq-server) | [Milvus](https://github.com/milvus-io/milvus) | [Garage](https://github.com/deuxfleurs-org/garage) | [Apache Superset](https://github.com/apache/superset) | [Grafana](https://github.com/grafana/grafana) |
| 4 | [Valkey](https://github.com/valkey-io/valkey) | [PocketBase](https://github.com/pocketbase/pocketbase) | [Debezium](https://github.com/debezium/debezium) | [Weaviate](https://github.com/weaviate/weaviate) | [Ceph](https://github.com/ceph/ceph) | [Metabase](https://github.com/metabase/metabase) | [Grafana Loki](https://github.com/grafana/loki) |
| 5 | [MongoDB Community](https://github.com/mongodb/mongo) | [Appwrite](https://github.com/appwrite/appwrite) | [Apache Pulsar](https://github.com/apache/pulsar) | [Chroma](https://github.com/chroma-core/chroma) | [OpenDAL](https://github.com/apache/opendal) | [DuckDB](https://github.com/duckdb/duckdb) | [Jaeger](https://github.com/jaegertracing/jaeger) |
| 6 | [Neo4j Community](https://github.com/neo4j/neo4j) | [NocoDB](https://github.com/nocodb/nocodb) | [Redpanda](https://github.com/redpanda-data/redpanda) | [Vespa](https://github.com/vespa-engine/vespa) | [LakeFS](https://github.com/treeverse/lakeFS) | [Apache Druid](https://github.com/apache/druid) | [SigNoz](https://github.com/SigNoz/signoz) |

### L1 方案特点

| 方案 | 主要覆盖 | 特点 | 适配 GESA 的建议 |
|---|---|---|---|
| PostgreSQL | 状态数据库 | 成熟、强一致、事务、JSONB、行级安全、扩展生态强。 | 作为 `entities / tasks / workflows / feedbacks / permissions` 的主状态源。 |
| Supabase | 数据 API / Auth / Realtime / Storage | 基于 PostgreSQL，提供 Auth、Realtime、Storage、Edge Functions。 | 适合快速 MVP；后期可逐步拆出权限和工作流。 |
| Directus | 数据后台 / API | 面向 SQL 数据库的 Headless CMS / Data Platform，可快速生成管理后台和 API。 | 适合做对象管理后台、实体编辑器、基础数据维护界面。 |
| Hasura | GraphQL API | 从 PostgreSQL 等数据库自动生成 GraphQL API，权限规则较强。 | 适合给工作空间层提供数据查询 API。 |
| PocketBase | 轻量后端 | 单文件 Go 后端，内置 SQLite、Auth、Realtime、管理 UI。 | 适合极小部署版或边缘设备版。 |
| Kafka / Pulsar / NATS / RabbitMQ | 事件流 / 消息队列 | Kafka/Pulsar 偏大规模事件流；NATS 偏轻量消息总线；RabbitMQ 偏传统任务消息。 | GESA 早期可用 NATS/RabbitMQ；企业级事件日志可引入 Kafka/Pulsar。 |
| Debezium | CDC | 捕获数据库变更并发送到 Kafka 等事件流。 | 适合把数据库状态变化转成事件，支撑自进化分析。 |
| pgvector / Qdrant / Milvus / Weaviate / Chroma / Vespa | 向量记忆 | pgvector 简单；Qdrant 部署轻；Milvus 扩展强；Weaviate 功能完整；Chroma 适合本地开发；Vespa 适合搜索+排序。 | 人和 Agent 的记忆层可以从 pgvector/Qdrant 起步。 |
| MinIO / SeaweedFS / Garage / Ceph | 对象存储 | MinIO 兼容 S3；SeaweedFS 轻量；Garage 分布式友好；Ceph 功能重但完整。 | 存合同、报告、图片、日志包、部署产物。 |
| OpenSearch / ClickHouse / Superset / Metabase / DuckDB / Druid | 搜索与分析 | OpenSearch 做全文检索；ClickHouse/Druid 做大规模分析；Metabase/Superset 做 BI；DuckDB 做本地分析。 | 用于操作日志、反馈聚类、任务效率分析。 |
| OpenTelemetry / Prometheus / Grafana / Loki / Jaeger / SigNoz | 观测与审计 | 指标、日志、链路追踪、APM。 | 所有 Agent 调用、操作引擎和工作流执行都应产生可观测数据。 |

---

## L2. Object & Permission Layer

对象与权限层负责实体管理、权限管理、AI Agent 接入、基础操作注册和治理审计。它是 GESA 的安全边界核心。

| # | 实体管理 / 后台 | 身份认证 / SSO | 授权 / 策略引擎 | AI Agent 管理 | 操作注册 / API 契约 | 审计 / 治理 |
|---|---|---|---|---|---|---|
| 1 | [Directus](https://github.com/directus/directus) | [Keycloak](https://github.com/keycloak/keycloak) | [OpenFGA](https://github.com/openfga/openfga) | [LangGraph](https://github.com/langchain-ai/langgraph) | [Backstage Catalog](https://github.com/backstage/backstage) | [OpenTelemetry](https://github.com/open-telemetry/opentelemetry-collector) |
| 2 | [NocoDB](https://github.com/nocodb/nocodb) | [Zitadel](https://github.com/zitadel/zitadel) | [Casbin](https://github.com/casbin/casbin) | [Microsoft AutoGen](https://github.com/microsoft/autogen) | [OpenAPI Generator](https://github.com/OpenAPITools/openapi-generator) | [Langfuse](https://github.com/langfuse/langfuse) |
| 3 | [Baserow](https://github.com/bram2w/baserow) | [Authentik](https://github.com/goauthentik/authentik) | [Open Policy Agent](https://github.com/open-policy-agent/opa) | [CrewAI](https://github.com/crewAIInc/crewAI) | [Apicurio Registry](https://github.com/Apicurio/apicurio-registry) | [Arize Phoenix](https://github.com/Arize-ai/phoenix) |
| 4 | [Strapi](https://github.com/strapi/strapi) | [Ory Kratos](https://github.com/ory/kratos) | [SpiceDB](https://github.com/authzed/spicedb) | [Dify](https://github.com/langgenius/dify) | [AsyncAPI](https://github.com/asyncapi/spec) | [LlamaFirewall](https://github.com/meta-llama/PurpleLlama/tree/main/LlamaFirewall) |
| 5 | [Appwrite](https://github.com/appwrite/appwrite) | [Ory Hydra](https://github.com/ory/hydra) | [Ory Keto](https://github.com/ory/keto) | [Flowise](https://github.com/FlowiseAI/Flowise) | [Model Context Protocol](https://github.com/modelcontextprotocol) | [Cerbos](https://github.com/cerbos/cerbos) |
| 6 | [Backstage](https://github.com/backstage/backstage) | [Authelia](https://github.com/authelia/authelia) | [Permify](https://github.com/Permify/permify) | [SuperAGI](https://github.com/TransformerOptimus/SuperAGI) | [FastAPI](https://github.com/fastapi/fastapi) | [Sentry](https://github.com/getsentry/sentry) |

### L2 方案特点

| 方案 | 主要覆盖 | 特点 | 适配 GESA 的建议 |
|---|---|---|---|
| Directus / NocoDB / Baserow / Strapi / Appwrite | 实体管理、后台、数据 API | 都能快速生成对象管理后台；Directus 更偏 SQL 数据平台，NocoDB/Baserow 更偏表格化管理，Strapi 更偏内容模型，Appwrite 更偏 BaaS。 | 早期可用 Directus/NocoDB 快速管理 `entities` 和基础配置。 |
| Backstage | 服务目录、软件目录、插件生态 | 适合记录模块、服务、API、所有者、文档和运行状态。 | 可作为 Project Control Plane 的服务目录，也可在 L2 记录实体化的软件模块。 |
| Keycloak / Zitadel / Authentik / Authelia / Ory Kratos / Ory Hydra | 身份认证、SSO、OIDC、OAuth2 | Keycloak 最成熟；Zitadel 云原生；Authentik 易用；Ory 组件化强。 | 真人实体登录、组织身份、服务间认证建议优先 Keycloak 或 Zitadel。 |
| OpenFGA / SpiceDB / Ory Keto | 关系型授权 | 适合“谁可以对哪个对象做什么”的 Zanzibar 风格权限。 | 非常适合 GESA 的 `Subject-Object-Action-Scope` 权限模型。 |
| Casbin / OPA / Cerbos / Permify | 策略引擎 | Casbin 轻量易嵌入；OPA 通用策略强；Cerbos 偏应用授权；Permify 偏权限服务。 | Casbin 适合早期嵌入式权限；OPA 适合平台级策略；OpenFGA 适合对象级权限图。 |
| LangGraph / AutoGen / CrewAI / Dify / Flowise / SuperAGI | Agent 管理和编排 | LangGraph 偏可控状态机；AutoGen 偏多 Agent 对话；CrewAI 偏角色团队；Dify/Flowise 偏可视化 LLM 应用。 | Secretary Agent 建议优先 LangGraph；业务人员搭建 Agent 可用 Dify/Flowise。 |
| OpenAPI Generator / Apicurio Registry / AsyncAPI / MCP / FastAPI | 操作注册、API 契约、工具协议 | OpenAPI/AsyncAPI 适合定义操作接口；Apicurio 可做 schema/API 注册表；MCP 适合 Agent 工具接入。 | 原子操作应统一转成 OpenAPI / MCP 工具定义。 |
| OpenTelemetry / Langfuse / Phoenix / LlamaFirewall / Sentry | 审计、Agent 观测、安全治理 | OpenTelemetry 做基础追踪；Langfuse/Phoenix 做 LLM 可观测；LlamaFirewall 做 Agent 安全防线；Sentry 做错误追踪。 | 对每次 Agent 操作保留 trace、输入、输出、权限判断和结果。 |

---

## L3. Atomic Operation Layer

原子操作层负责不可分割操作的定义、执行和审计。这里的工具应被包装成统一的 Operation Definition，并受到 L2 权限模块约束。

| # | Browser / RPA | API / 工具执行 | 文件 / 文档操作 | 通信操作 | 任务队列 / Worker | 沙箱 / 审计 |
|---|---|---|---|---|---|---|
| 1 | [Playwright](https://github.com/microsoft/playwright) | [MCP SDK](https://github.com/modelcontextprotocol) | [Apache Tika](https://github.com/apache/tika) | [Chatwoot](https://github.com/chatwoot/chatwoot) | [Temporal](https://github.com/temporalio/temporal) | [OpenTelemetry](https://github.com/open-telemetry/opentelemetry-collector) |
| 2 | [Selenium](https://github.com/SeleniumHQ/selenium) | [FastAPI](https://github.com/fastapi/fastapi) | [Unstructured](https://github.com/Unstructured-IO/unstructured) | [Mattermost](https://github.com/mattermost/mattermost) | [Celery](https://github.com/celery/celery) | [Sentry](https://github.com/getsentry/sentry) |
| 3 | [browser-use](https://github.com/browser-use/browser-use) | [LangChain Tools](https://github.com/langchain-ai/langchain) | [Pandoc](https://github.com/jgm/pandoc) | [Rocket.Chat](https://github.com/RocketChat/Rocket.Chat) | [Dramatiq](https://github.com/Bogdanp/dramatiq) | [Docker](https://github.com/docker) |
| 4 | [Robot Framework](https://github.com/robotframework/robotframework) | [n8n](https://github.com/n8n-io/n8n) | [LibreOffice](https://git.libreoffice.org/core) | [Postal](https://github.com/postalserver/postal) | [RQ](https://github.com/rq/rq) | [gVisor](https://github.com/google/gvisor) |
| 5 | [Robocorp](https://github.com/robocorp) | [Node-RED](https://github.com/node-red/node-red) | [MinIO Client](https://github.com/minio/mc) | [Mautic](https://github.com/mautic/mautic) | [Apache Airflow](https://github.com/apache/airflow) | [Firecracker](https://github.com/firecracker-microvm/firecracker) |
| 6 | [TagUI](https://github.com/aisingapore/TagUI) | [Windmill](https://github.com/windmill-labs/windmill) | [Docling](https://github.com/docling-project/docling) | [Zulip](https://github.com/zulip/zulip) | [Prefect](https://github.com/PrefectHQ/prefect) | [LlamaFirewall](https://github.com/meta-llama/PurpleLlama/tree/main/LlamaFirewall) |

### L3 方案特点

| 方案 | 主要覆盖 | 特点 | 适配 GESA 的建议 |
|---|---|---|---|
| Playwright / Selenium / browser-use | 浏览器操作 | Playwright 稳定现代；Selenium 生态广；browser-use 面向 LLM 浏览器操作。 | Browser Operation Engine 可优先 Playwright，Agent 浏览器任务可叠加 browser-use。 |
| Robot Framework / Robocorp / TagUI | RPA / 自动化测试 | 适合把重复性的 UI 操作和业务流程脚本化。 | 可把企业遗留系统操作封装成原子操作。 |
| MCP / FastAPI / LangChain Tools / n8n / Node-RED / Windmill | 工具执行与操作编排 | MCP 适合 Agent 工具协议；FastAPI 适合自定义操作 API；n8n/Node-RED/Windmill 适合可视化和脚本化动作。 | 所有操作引擎建议提供 OpenAPI 或 MCP 描述。 |
| Apache Tika / Unstructured / Pandoc / LibreOffice / Docling | 文档解析与转换 | 覆盖 PDF、Office、HTML、Markdown 等格式。 | 合同、发票、报告、附件解析可以封装为 Document Operation Engine。 |
| Chatwoot / Mattermost / Rocket.Chat / Postal / Mautic / Zulip | 通信操作 | 覆盖客服、团队通信、邮件服务器、营销自动化。 | `Communicate` 类操作可以对接这些系统。 |
| Temporal / Celery / Dramatiq / RQ / Airflow / Prefect | 异步任务、Worker、重试 | Temporal 最适合长事务和可靠执行；Celery/RQ/Dramatiq 适合 Python 任务队列；Airflow/Prefect 适合数据流。 | 原子操作执行可以走任务队列，重要操作走 Temporal。 |
| Docker / gVisor / Firecracker / OpenTelemetry / Sentry / LlamaFirewall | 沙箱、审计、安全 | Docker 通用；gVisor/Firecracker 隔离更强；OpenTelemetry/Sentry 记录追踪与错误；LlamaFirewall 做 Agent 安全扫描。 | 高风险操作需要隔离执行并记录完整审计。 |

---

## L4. Workspace Layer

工作空间层是秘书 Agent、工作流、时间调度和可视化协作的主层。它需要同时服务真人、AI Agent、项目、客户、合同、订单等不同实体。

| # | 工作流 / BPM | 时间调度 / 日历 | 任务 / 项目管理 | 可视化 / Dashboard | 秘书 Agent 工作台 | 人工兜底 / 表单 |
|---|---|---|---|---|---|---|
| 1 | [Temporal](https://github.com/temporalio/temporal) | [Cal.com](https://github.com/calcom/cal.com) | [Plane](https://github.com/makeplane/plane) | [Appsmith](https://github.com/appsmithorg/appsmith) | [Dify](https://github.com/langgenius/dify) | [Appsmith](https://github.com/appsmithorg/appsmith) |
| 2 | [Flowable](https://github.com/flowable/flowable-engine) | [Nextcloud Calendar](https://github.com/nextcloud/calendar) | [OpenProject](https://github.com/opf/openproject) | [ToolJet](https://github.com/ToolJet/ToolJet) | [OpenWebUI](https://github.com/open-webui/open-webui) | [ToolJet](https://github.com/ToolJet/ToolJet) |
| 3 | [Camunda 7](https://github.com/camunda/camunda-bpm-platform) | [Radicale](https://github.com/Kozea/Radicale) | [AppFlowy](https://github.com/AppFlowy-IO/AppFlowy) | [Budibase](https://github.com/Budibase/budibase) | [Flowise](https://github.com/FlowiseAI/Flowise) | [Budibase](https://github.com/Budibase/budibase) |
| 4 | [Operaton](https://github.com/operaton/operaton) | [Rallly](https://github.com/lukevella/rallly) | [Vikunja](https://kolaente.dev/vikunja/vikunja) | [Grafana](https://github.com/grafana/grafana) | [LangGraph](https://github.com/langchain-ai/langgraph) | [Directus](https://github.com/directus/directus) |
| 5 | [n8n](https://github.com/n8n-io/n8n) | [Cronicle](https://github.com/jhuckaby/Cronicle) | [Taiga](https://github.com/taigaio) | [Metabase](https://github.com/metabase/metabase) | [AutoGen Studio](https://github.com/microsoft/autogen/tree/main/python/packages/autogen-studio) | [NocoDB](https://github.com/nocodb/nocodb) |
| 6 | [Node-RED](https://github.com/node-red/node-red) | [Apache DolphinScheduler](https://github.com/apache/dolphinscheduler) | [Redmine](https://github.com/redmine/redmine) | [Apache Superset](https://github.com/apache/superset) | [CrewAI](https://github.com/crewAIInc/crewAI) | [Baserow](https://github.com/bram2w/baserow) |
| 7 | [Windmill](https://github.com/windmill-labs/windmill) | [Temporal Schedules](https://docs.temporal.io/schedule) | [Kanboard](https://github.com/kanboard/kanboard) | [Redash](https://github.com/getredash/redash) | [Rasa](https://github.com/RasaHQ/rasa) | [Formbricks](https://github.com/formbricks/formbricks) |

### L4 方案特点

| 方案 | 主要覆盖 | 特点 | 适配 GESA 的建议 |
|---|---|---|---|
| Temporal | 长事务工作流、可靠执行 | 适合长时间运行、可重试、可恢复的业务流程。 | 高可靠核心工作流优先用 Temporal。 |
| Flowable / Camunda 7 / Operaton | BPMN / CMMN / DMN | 适合传统企业流程、审批、人任务、流程建模。Operaton 是 Camunda 7 社区分支方向。 | 企业审批和标准流程可用 BPMN 系列。 |
| n8n / Node-RED / Windmill | 低代码工作流 / 自动化 | n8n 偏 SaaS 集成；Node-RED 偏事件流和 IoT；Windmill 偏开发者脚本和内部工具。 | 适合把业务人员能理解的流程转成可执行自动化。 |
| Cal.com / Nextcloud Calendar / Radicale / Rallly / Cronicle / DolphinScheduler | 日历与调度 | Cal.com 适合预约；Nextcloud/Radicale 适合 CalDAV；Cronicle 偏定时任务；DolphinScheduler 偏数据任务调度。 | 真人日程用 CalDAV/Cal.com；后台任务用 Cronicle/Temporal。 |
| Plane / OpenProject / AppFlowy / Vikunja / Taiga / Redmine / Kanboard | 任务与项目管理 | Plane 类似 Linear；OpenProject 偏企业项目；AppFlowy 偏 Notion；Redmine/Taiga/Kanboard 经典稳定。 | 早期可直接集成 Plane/OpenProject，少造项目管理轮子。 |
| Appsmith / ToolJet / Budibase / Grafana / Metabase / Superset / Redash | 可视化和内部工具 | Appsmith/ToolJet/Budibase 做内部操作台；Grafana 做监控；Metabase/Superset/Redash 做 BI。 | 真人工作空间 MVP 可以用 Appsmith 或 ToolJet 快速搭建。 |
| Dify / OpenWebUI / Flowise / LangGraph / AutoGen Studio / CrewAI / Rasa | 秘书 Agent 工作台 | Dify/Flowise/OpenWebUI 易用；LangGraph 可控；AutoGen Studio 可视化多 Agent；Rasa 偏传统对话系统。 | Secretary Agent 推荐 LangGraph + 可视化外壳；业务搭建可用 Dify。 |
| Directus / NocoDB / Baserow / Formbricks | 人工兜底表单 | 能在没有 Agent 时让人通过表单、表格、按钮继续操作。 | 所有 Agent 自动化都应有人工操作入口。 |

---

## L5. Feedback Layer

反馈层收集所有实体的反馈，包括真人反馈、客户反馈、Agent 失败反馈、工作流阻塞、权限冲突、UI 问题和部署异常。

| # | 用户反馈 | Issue / Bug | 客服 / Helpdesk | 产品分析 / 回放 | Agent 反馈 / Eval | 知识库 / 分流 |
|---|---|---|---|---|---|---|
| 1 | [Fider](https://github.com/getfider/fider) | [GitHub Issues](https://github.com/features/issues) | [Chatwoot](https://github.com/chatwoot/chatwoot) | [PostHog](https://github.com/PostHog/posthog) | [Langfuse](https://github.com/langfuse/langfuse) | [Zammad Knowledge Base](https://github.com/zammad/zammad) |
| 2 | [Formbricks](https://github.com/formbricks/formbricks) | [Gitea Issues](https://github.com/go-gitea/gitea) | [Zammad](https://github.com/zammad/zammad) | [Matomo](https://github.com/matomo-org/matomo) | [Arize Phoenix](https://github.com/Arize-ai/phoenix) | [BookStack](https://github.com/BookStackApp/BookStack) |
| 3 | [Astuto](https://github.com/riggraz/astuto) | [Forgejo Issues](https://codeberg.org/forgejo/forgejo) | [FreeScout](https://github.com/freescout-help-desk/freescout) | [Plausible](https://github.com/plausible/analytics) | [Helicone](https://github.com/Helicone/helicone) | [Outline](https://github.com/outline/outline) |
| 4 | [Votail](https://github.com/votail/votail) | [Plane](https://github.com/makeplane/plane) | [OTOBO](https://github.com/RotherOSS/otobo) | [Umami](https://github.com/umami-software/umami) | [promptfoo](https://github.com/promptfoo/promptfoo) | [Wiki.js](https://github.com/Requarks/wiki) |
| 5 | [LimeSurvey](https://github.com/LimeSurvey/LimeSurvey) | [Redmine](https://github.com/redmine/redmine) | [osTicket](https://github.com/osTicket/osTicket) | [OpenReplay](https://github.com/openreplay/openreplay) | [DeepEval](https://github.com/confident-ai/deepeval) | [Docmost](https://github.com/docmost/docmost) |
| 6 | [OhMyForm](https://github.com/ohmyform/ohmyform) | [Bugzilla](https://github.com/bugzilla/bugzilla) | [MantisBT](https://github.com/mantisbt/mantisbt) | [Sentry](https://github.com/getsentry/sentry) | [Ragas](https://github.com/explodinggradients/ragas) | [OpenSearch](https://github.com/opensearch-project/OpenSearch) |

### L5 方案特点

| 方案 | 主要覆盖 | 特点 | 适配 GESA 的建议 |
|---|---|---|---|
| Fider / Formbricks / Astuto / Votail / LimeSurvey / OhMyForm | 用户反馈和表单 | Fider/Astuto 偏功能投票；Formbricks 偏调查和体验管理；LimeSurvey 功能重；OhMyForm 简单。 | 用于收集真人、客户、内部员工的结构化反馈。 |
| GitHub Issues / Gitea / Forgejo / Plane / Redmine / Bugzilla / MantisBT | Issue 与缺陷跟踪 | GitHub 生态强；Gitea/Forgejo 可自托管；Plane 现代；Redmine/Bugzilla/MantisBT 经典稳定。 | 项目自进化候选项可以落到 Issue 系统。 |
| Chatwoot / Zammad / FreeScout / OTOBO / osTicket | 客服与 Helpdesk | Chatwoot 偏多渠道客服；Zammad 更像现代工单系统；FreeScout/osTicket 简洁；OTOBO 偏 ITSM。 | 客户反馈和内部支持请求可进入统一 feedbacks 表。 |
| PostHog / Matomo / Plausible / Umami / OpenReplay / Sentry | 产品分析、会话回放、错误反馈 | PostHog 功能最全；Matomo 稳定；Plausible/Umami 轻量；OpenReplay 做会话回放；Sentry 做错误追踪。 | UI 使用困难、错误、性能问题可以自动转反馈。 |
| Langfuse / Phoenix / Helicone / promptfoo / DeepEval / Ragas | Agent 反馈和评测 | Langfuse/Helicone 做调用观测；Phoenix 做 tracing/evals；promptfoo/DeepEval/Ragas 做评测。 | 对 Secretary Agent 和 Coding Agent 记录失败、成本、质量和评测结果。 |
| BookStack / Outline / Wiki.js / Docmost / OpenSearch | 知识库与分流 | 知识库沉淀问题，OpenSearch 做检索和聚类。 | 反馈处理后应沉淀为 FAQ、流程改进和进化候选。 |

---

## M1. Project Control Plane

项目控制平面负责项目安装、模块注册、模块依赖、增量规则、部署、迁移、回滚和运行健康。它是 GESA 的元管理层。

| # | 模块目录 / 服务目录 | 安装 / 部署 | IaC / 配置管理 | CI/CD / GitOps | 安全 / 策略 / 密钥 | 制品 / Registry | 健康 / 回滚 |
|---|---|---|---|---|---|---|---|
| 1 | [Backstage](https://github.com/backstage/backstage) | [Docker Compose](https://github.com/docker/compose) | [OpenTofu](https://github.com/opentofu/opentofu) | [Argo CD](https://github.com/argoproj/argo-cd) | [OpenBao](https://github.com/openbao/openbao) | [Harbor](https://github.com/goharbor/harbor) | [Prometheus](https://github.com/prometheus/prometheus) |
| 2 | [Portainer CE](https://github.com/portainer/portainer) | [Helm](https://github.com/helm/helm) | [Pulumi](https://github.com/pulumi/pulumi) | [Flux CD](https://github.com/fluxcd/flux2) | [Mozilla SOPS](https://github.com/getsops/sops) | [Verdaccio](https://github.com/verdaccio/verdaccio) | [Grafana](https://github.com/grafana/grafana) |
| 3 | [KubeVela](https://github.com/kubevela/kubevela) | [Kustomize](https://github.com/kubernetes-sigs/kustomize) | [Ansible](https://github.com/ansible/ansible) | [Jenkins](https://github.com/jenkinsci/jenkins) | [Sealed Secrets](https://github.com/bitnami-labs/sealed-secrets) | [Nexus Repository OSS](https://github.com/sonatype/nexus-public) | [Grafana Loki](https://github.com/grafana/loki) |
| 4 | [Rancher](https://github.com/rancher/rancher) | [Kubernetes](https://github.com/kubernetes/kubernetes) | [Salt](https://github.com/saltstack/salt) | [Woodpecker CI](https://github.com/woodpecker-ci/woodpecker) | [External Secrets](https://github.com/external-secrets/external-secrets) | [Gitea Packages](https://github.com/go-gitea/gitea) | [Argo Rollouts](https://github.com/argoproj/argo-rollouts) |
| 5 | [Cycloid Console](https://github.com/cycloidio/youdeploy) | [Devbox](https://github.com/jetify-com/devbox) | [Crossplane](https://github.com/crossplane/crossplane) | [Tekton Pipelines](https://github.com/tektoncd/pipeline) | [OPA Gatekeeper](https://github.com/open-policy-agent/gatekeeper) | [Forgejo Packages](https://codeberg.org/forgejo/forgejo) | [Flagger](https://github.com/fluxcd/flagger) |
| 6 | [Artifact Hub](https://github.com/artifacthub/hub) | [Nix](https://github.com/NixOS/nix) | [Rudder](https://github.com/Normation/rudder) | [Dagger](https://github.com/dagger/dagger) | [Kyverno](https://github.com/kyverno/kyverno) | [OCI Distribution](https://github.com/distribution/distribution) | [Kured](https://github.com/kubereboot/kured) |

### M1 方案特点

| 方案 | 主要覆盖 | 特点 | 适配 GESA 的建议 |
|---|---|---|---|
| Backstage / Portainer / KubeVela / Rancher / Artifact Hub | 模块目录、服务目录、平台控制台 | Backstage 适合服务目录和开发者门户；Portainer 适合 Docker/K8s 管理；KubeVela/Rancher 适合应用平台；Artifact Hub 适合包和插件发现。 | Project Control Plane 可优先 Backstage，轻量部署可用 Portainer。 |
| Docker Compose / Helm / Kustomize / Kubernetes / Devbox / Nix | 安装与部署 | Compose 适合单机；Helm/Kustomize 适合 K8s；Devbox/Nix 适合可重复开发环境。 | MVP 用 Docker Compose；企业部署用 Helm/Kustomize。 |
| OpenTofu / Pulumi / Ansible / Salt / Crossplane / Rudder | IaC 和配置管理 | OpenTofu 是 Terraform 开源分支；Pulumi 用通用语言写 IaC；Ansible/Salt 做配置自动化；Crossplane 把云资源变成 K8s API。 | 安装管理层的环境创建优先 OpenTofu + Ansible。 |
| Argo CD / Flux / Jenkins / Woodpecker / Tekton / Dagger | CI/CD 与 GitOps | Argo/Flux 偏 GitOps；Jenkins 生态老牌；Woodpecker 轻量；Tekton K8s 原生；Dagger 适合可组合 CI。 | 部署闭环建议 GitOps 化：Git 提交即部署候选。 |
| OpenBao / SOPS / Sealed Secrets / External Secrets / OPA Gatekeeper / Kyverno | 密钥与策略 | OpenBao 是 Vault 社区方向；SOPS 适合 Git 加密；Gatekeeper/Kyverno 做 K8s 策略。 | 权限和密钥管理必须独立于业务 Agent。 |
| Harbor / Verdaccio / Nexus / Gitea Packages / Forgejo Packages / OCI Distribution | 制品仓库 | Harbor 做容器镜像与扫描；Verdaccio 做 npm；Nexus 通用制品；Gitea/Forgejo 自带包。 | 安装控制平面应记录每个模块的镜像、包版本和迁移版本。 |
| Prometheus / Grafana / Loki / Argo Rollouts / Flagger / Kured | 健康检查与回滚 | Prometheus/Grafana/Loki 做观测；Argo Rollouts/Flagger 做渐进发布；Kured 做节点重启。 | 自进化引擎提交变更后，部署层必须有健康检查和回滚策略。 |

---

## M2. Self-Evolution Engine

项目自进化引擎由独立 Coding Agent 驱动，不依赖项目内部 AI Agent API。它读取反馈、日志、增量规则和测试结果，生成代码修改、迁移、测试、PR 和部署计划。

| # | Coding Agent | Repo / PR 自动化 | 代码搜索 / 上下文 | 静态分析 / 安全 | 测试 / Eval | Agent 观测 / Guardrail | 发布 / 变更管理 |
|---|---|---|---|---|---|---|---|
| 1 | [OpenHands](https://github.com/All-Hands-AI/OpenHands) | [Renovate](https://github.com/renovatebot/renovate) | [Sourcegraph](https://github.com/sourcegraph/sourcegraph) | [Semgrep](https://github.com/semgrep/semgrep) | [pytest](https://github.com/pytest-dev/pytest) | [Langfuse](https://github.com/langfuse/langfuse) | [Argo CD](https://github.com/argoproj/argo-cd) |
| 2 | [SWE-agent](https://github.com/SWE-agent/SWE-agent) | [release-please](https://github.com/googleapis/release-please) | [Zoekt](https://github.com/sourcegraph/zoekt) | [Trivy](https://github.com/aquasecurity/trivy) | [Playwright](https://github.com/microsoft/playwright) | [Phoenix](https://github.com/Arize-ai/phoenix) | [Flux CD](https://github.com/fluxcd/flux2) |
| 3 | [Aider](https://github.com/Aider-AI/aider) | [GitHub CLI](https://github.com/cli/cli) | [OpenGrok](https://github.com/oracle/opengrok) | [Bandit](https://github.com/PyCQA/bandit) | [Cypress](https://github.com/cypress-io/cypress) | [Helicone](https://github.com/Helicone/helicone) | [Changesets](https://github.com/changesets/changesets) |
| 4 | [Continue](https://github.com/continuedev/continue) | [Gitea](https://github.com/go-gitea/gitea) | [ctags](https://github.com/universal-ctags/ctags) | [Ruff](https://github.com/astral-sh/ruff) | [promptfoo](https://github.com/promptfoo/promptfoo) | [LlamaFirewall](https://github.com/meta-llama/PurpleLlama/tree/main/LlamaFirewall) | [Jenkins](https://github.com/jenkinsci/jenkins) |
| 5 | [Roo Code](https://github.com/RooVetGit/Roo-Code) | [Forgejo](https://codeberg.org/forgejo/forgejo) | [ripgrep](https://github.com/BurntSushi/ripgrep) | [ESLint](https://github.com/eslint/eslint) | [DeepEval](https://github.com/confident-ai/deepeval) | [Guardrails AI](https://github.com/guardrails-ai/guardrails) | [Woodpecker CI](https://github.com/woodpecker-ci/woodpecker) |
| 6 | [Cline](https://github.com/cline/cline) | [GitLab CE](https://gitlab.com/gitlab-org/gitlab) | [CodeQL CLI](https://github.com/github/codeql-cli-binaries) | [OSV-Scanner](https://github.com/google/osv-scanner) | [Ragas](https://github.com/explodinggradients/ragas) | [AgentSight](https://github.com/agent-sight/agentsight) | [Tekton](https://github.com/tektoncd/pipeline) |
| 7 | [Goose](https://github.com/block/goose) | [Gerrit](https://github.com/GerritCodeReview/gerrit) | [Sourcetrail](https://github.com/CoatiSoftware/Sourcetrail) | [Syft](https://github.com/anchore/syft) | [SWE-bench](https://github.com/SWE-bench/SWE-bench) | [OpenTelemetry](https://github.com/open-telemetry/opentelemetry-collector) | [Dagger](https://github.com/dagger/dagger) |
| 8 | [AutoGPT](https://github.com/Significant-Gravitas/AutoGPT) | [Woodpecker CI](https://github.com/woodpecker-ci/woodpecker) | [Marimo](https://github.com/marimo-team/marimo) | [Grype](https://github.com/anchore/grype) | [AgentIssue-Bench](https://github.com/alfin06/AgentIssue-Bench) | [promptfoo](https://github.com/promptfoo/promptfoo) | [GoCD](https://github.com/gocd/gocd) |

### M2 方案特点

| 方案 | 主要覆盖 | 特点 | 适配 GESA 的建议 |
|---|---|---|---|
| OpenHands / SWE-agent / Aider / Continue / Roo Code / Cline / Goose / AutoGPT | Coding Agent | OpenHands 更像完整软件工程 Agent 平台；SWE-agent 偏 issue 修复；Aider 适合终端 pair programming；Continue/Roo/Cline 偏 IDE；Goose 偏本地 Agent；AutoGPT 偏通用自主 Agent。 | 自进化引擎可优先 OpenHands 或 Aider；复杂 issue 修复可评估 SWE-agent。 |
| Renovate / release-please / GitHub CLI / Gitea / Forgejo / GitLab CE / Gerrit | Repo 与 PR 自动化 | Renovate 做依赖升级；release-please 做版本发布；GitHub CLI 做自动 PR；Gitea/Forgejo/GitLab/Gerrit 可自托管代码平台。 | 自进化引擎应只通过 PR/patch 修改项目，避免直接改生产代码。 |
| Sourcegraph / Zoekt / OpenGrok / ctags / ripgrep / CodeQL CLI / Sourcetrail | 代码搜索和上下文 | Sourcegraph/Zoekt 做大规模代码搜索；ripgrep/ctags 轻量；OpenGrok 适合代码浏览；CodeQL 适合语义查询。 | Coding Agent 需要代码索引，避免上下文盲改。 |
| Semgrep / Trivy / Bandit / Ruff / ESLint / OSV-Scanner / Syft / Grype | 静态分析、安全和供应链 | Semgrep 做规则扫描；Trivy/Grype 做漏洞扫描；Syft 做 SBOM；Bandit/Ruff/ESLint 做语言级质量检查。 | 所有自进化 PR 必须通过安全和质量扫描。 |
| pytest / Playwright / Cypress / promptfoo / DeepEval / Ragas / SWE-bench / AgentIssue-Bench | 测试与评测 | pytest/Playwright/Cypress 覆盖传统测试；promptfoo/DeepEval/Ragas 覆盖 LLM 评测；SWE-bench/AgentIssue-Bench 覆盖 Agent 修复评测。 | 自进化引擎应同时跑代码测试和 Agent 行为评测。 |
| Langfuse / Phoenix / Helicone / LlamaFirewall / Guardrails AI / AgentSight / OpenTelemetry | Agent 观测与安全护栏 | Langfuse/Phoenix/Helicone 做 LLM 调用追踪；LlamaFirewall/Guardrails 做安全约束；AgentSight 做系统级 Agent 观测。 | 自进化 Agent 的每次工具调用、代码编辑和 PR 生成都应可追踪。 |
| Argo CD / Flux / Jenkins / Woodpecker / Tekton / Dagger / GoCD | 发布与变更管理 | GitOps 和 CI/CD 工具负责部署验证、环境推进和回滚。 | 自进化引擎只提交变更；部署由 Project Control Plane 执行。 |

---

# 结论：GESA 的推荐开源组合

## MVP 组合

```text
PostgreSQL + pgvector
Directus
Keycloak + Casbin / OpenFGA
FastAPI + MCP
Temporal + Windmill
Appsmith + Plane + Cal.com
LangGraph + Dify
Fider + GitHub Issues / Gitea Issues
Docker Compose + OpenTofu + Ansible
Aider / OpenHands
```

## 企业级组合

```text
PostgreSQL + MinIO + OpenSearch + ClickHouse
Kafka / NATS + Debezium
Keycloak + OpenFGA + OPA
Temporal + Flowable / Operaton
Backstage + Argo CD + Flux + OpenTofu
Harbor + OpenBao + SOPS + Kyverno
OpenTelemetry + Prometheus + Grafana + Loki
LangGraph + Langfuse + Phoenix + LlamaFirewall
OpenHands + Semgrep + Trivy + promptfoo + SWE-bench
```

## 架构判断

GESA 的关键不在于选择单个“万能开源项目”，而在于定义稳定的五个核心 schema：

```text
Entity
Operation
Permission
Workflow
Feedback
```

只要这五个 schema 稳定，以上开源方案都可以作为可替换模块接入。每个方案进入 GESA 时，都应被封装为：

```text
Module
Operation Engine
Workspace Component
Feedback Source
Evolution Candidate Source
```

这样 GESA 才能长期自进化，而不会被某个工具锁死。
