# M1 Project Control Plane 项目控制平面

> Version: 0.1  
> Status: Layer Design  
> Position: Meta Plane / Installation & Governance

---

## 1. 定位

项目控制平面是 GESA 的元管理层，负责项目安装、模块注册、模块依赖、增量规则、部署、迁移、升级、回滚和运行健康。

它不属于普通企业业务运行层，也不应被普通业务 Agent 直接修改。

它回答的问题是：

```text
项目由哪些模块组成？
每个模块当前版本是什么？
模块之间有什么依赖？
如何安装、升级、回滚？
添加新操作、新工作流、新 Agent、新 schema 时必须遵循什么规则？
自进化引擎提交的变更如何安全部署？
```

---

## 2. 核心职责

```text
1. 维护项目模块注册表。
2. 管理模块依赖、版本、健康状态和负责人。
3. 记录项目增量规则。
4. 管理安装、部署、迁移、升级和回滚流程。
5. 接收反馈层和自进化引擎形成的候选变更。
6. 约束自进化引擎如何修改项目。
7. 管理配置、密钥、制品、环境和发布策略。
8. 提供项目级审计和健康检查。
```

---

## 3. 模块拆分

```text
Module Registry
- 模块注册
- 模块版本
- 模块依赖
- 模块所有者
- 模块健康状态

Installation Manager
- 初始化安装
- 环境检查
- 依赖安装
- 配置生成
- 数据目录初始化

Deployment Manager
- 部署计划
- 部署执行
- 灰度发布
- 健康检查
- 回滚

Migration Manager
- 数据库迁移
- 知识库迁移
- 配置迁移
- 模块迁移

Increment Rule Module
- 添加基础操作规则
- 添加工作流规则
- 添加 Agent 规则
- 添加 schema 规则
- 添加 UI 规则
- 添加知识规则

Artifact & Secret Manager
- 镜像
- 包
- 配置
- 密钥
- 证书

Evolution Candidate Gateway
- 接收候选
- 校验候选
- 绑定规则
- 分派给 M2
```

---

## 4. 模块注册表

```text
module_id
module_name
module_type
version
dependencies
owner_entity_id
repository_uri
artifact_uri
config_schema_uri
migration_status
health_status
upgrade_policy
rollback_policy
created_at
updated_at
```

模块类型：

```text
database
knowledge_base
object_permission
atomic_operation
workspace
feedback
control_plane
self_evolution
ui
connector
agent
```

---

## 5. 增量规则

### 5.1 添加基础操作

```text
1. 注册 operation definition。
2. 绑定 operation category。
3. 声明 input/output schema。
4. 声明所需权限。
5. 声明执行引擎。
6. 添加审计规则。
7. 添加测试用例。
8. 添加 UI 操作入口或 Agent 工具描述。
9. 更新知识库文档。
10. 更新模块注册表。
```

### 5.2 添加工作流

```text
1. 创建 workflow definition。
2. 声明适用实体类型。
3. 绑定原子操作。
4. 设置权限边界。
5. 设置审批节点。
6. 设置异常处理。
7. 设置可视化节点。
8. 添加版本号。
9. 添加运行日志规则。
10. 更新知识库文档。
```

### 5.3 添加新 Agent

```text
1. 创建 agent profile。
2. 声明能力范围。
3. 绑定可用工具。
4. 绑定权限策略。
5. 设置上下文范围。
6. 设置成本限制。
7. 设置失败回退机制。
8. 设置审计规则。
9. 设置可视化管理入口。
10. 更新知识库文档。
```

### 5.4 添加数据库 schema

```text
1. 创建迁移文件。
2. 声明兼容性影响。
3. 声明回滚策略。
4. 更新 ORM / DAO 层。
5. 添加单元测试。
6. 添加集成测试。
7. 更新文档。
8. 在模块注册表记录版本。
```

### 5.5 添加知识库规则

```text
1. 创建知识文档或知识块 schema。
2. 声明来源和所有者。
3. 声明权限边界。
4. 声明版本策略。
5. 声明检索策略。
6. 添加知识更新候选规则。
7. 添加评测样例。
8. 更新知识库索引。
```

---

## 6. 部署链路

```text
变更候选进入 M1
  ↓
校验候选是否符合增量规则
  ↓
生成部署计划
  ↓
执行测试
  ↓
生成迁移和回滚计划
  ↓
部署到测试环境
  ↓
健康检查
  ↓
人工或策略审批
  ↓
部署到生产环境
  ↓
持续监控
  ↓
失败则回滚
```

---

## 7. 与自进化引擎关系

M2 可以生成代码、迁移、测试和 PR，但 M1 负责约束和部署。

```text
M2 负责改什么、怎么改。
M1 负责能不能改、按什么规则改、如何部署、如何回滚。
```

M2 不应绕过 M1 直接推送生产部署。

---

## 8. 核心 Schema

```text
module_registry
module_dependencies
module_versions
module_health_checks
increment_rules
deployment_plans
deployment_records
migration_records
rollback_plans
artifact_registry
secret_references
environment_configs
evolution_candidate_bindings
```

---

## 9. 开源方案

| 功能 | 轻量级 | MVP | 企业级 |
|---|---|---|---|
| 安装 | Docker Compose + Makefile | Docker Compose + Ansible | Kubernetes + Helm + Kustomize |
| 模块目录 | YAML registry | Backstage | Backstage + Catalog + Scorecards |
| 配置管理 | .env + SOPS | Ansible + SOPS | OpenTofu + External Secrets |
| GitOps | 手动 Pull | Woodpecker / GitHub Actions | Argo CD / Flux |
| 制品 | 本地镜像 | Gitea Packages / Harbor | Harbor + SBOM |
| 密钥 | .env 加密 | SOPS / OpenBao | OpenBao + External Secrets |
| 健康监控 | healthcheck scripts | Prometheus | Prometheus + Grafana + Loki |
| 回滚 | 脚本 | Compose rollback | Argo Rollouts / Flagger |

---

## 10. MVP 实现顺序

```text
1. module_registry.yaml
2. Makefile
3. docker-compose.yml
4. migrations/ 目录
5. docs/rules/ 增量规则
6. scripts/install.sh
7. scripts/backup.sh
8. scripts/restore.sh
9. scripts/upgrade.sh
10. deployment_records 表
```

---

## 11. 风险

| 风险 | 表现 | 对策 |
|---|---|---|
| 自进化绕过部署规则 | Coding Agent 直接改生产 | M2 只提交 PR，部署由 M1 控制 |
| 模块依赖混乱 | 升级一个模块破坏多个模块 | module_dependencies + 测试矩阵 |
| 无回滚 | 部署失败后无法恢复 | rollback_plan 必填 |
| 密钥泄漏 | 密钥进入仓库 | SOPS / OpenBao / Secret Scan |
| 增量规则缺失 | 项目越改越乱 | 所有新增类型先写规则再实现 |

---

## 12. 完成定义

```text
项目模块可注册、查询、版本化。
安装、备份、恢复、升级有统一命令。
新增 Operation/Workflow/Agent/Schema/Knowledge 有规则。
自进化候选必须绑定增量规则。
部署有健康检查和回滚计划。
普通业务 Agent 不能修改控制平面核心规则。
```
