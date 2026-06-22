# GESA 文档索引

> Version: 0.3  
> Purpose: 快速定位 GESA 的开发者架构层、企业架构层、核心引擎、层级设计、开源方案和组合方案文档。

---

## 1. 顶层入口

| 文档 | 说明 |
|---|---|
| [README.md](./README.md) | GESA 文档系统总入口，区分开发者架构层与企业架构层。 |
| [developer/README.md](./developer/README.md) | 面向开发者、维护者、Coding Agent、部署 Agent 的架构入口。 |
| [enterprise/README.md](./enterprise/README.md) | 面向企业管理者、部门首脑、业务用户和企业侧 Agent 的架构入口。 |
| [developer/documentation-boundary.md](./developer/documentation-boundary.md) | 定义开发者文档与企业文档的边界。 |

---

## 2. 企业架构层

| 模块 | 文档 | 定位 |
|---|---|---|
| GESA 企业空间系统 | [enterprise/space-system/](./enterprise/space-system/) | 企业侧第一入口，管理组织权限图、共享空间、个人工作区、Agent Team 和五大界面。 |
| 定位与边界 | [01-positioning](./enterprise/space-system/01-positioning.md) | 说明企业空间系统在 GESA 中的位置。 |
| 对象模型 | [02-object-model](./enterprise/space-system/02-object-model.md) | 定义实体、空间、资源、权限、聊天、日程、任务等基础对象。 |
| 组织权限图 | [03-organization-permission-graph](./enterprise/space-system/03-organization-permission-graph.md) | 定义层级、方向、格子、部门、投影和 CEO 规则。 |
| 空间类型与资源 | [04-space-types-and-resources](./enterprise/space-system/04-space-types-and-resources.md) | 定义个人、格子、部门、方向、层级、全局空间及其资源。 |
| 个人工作区 | [05-personal-workspace](./enterprise/space-system/05-personal-workspace.md) | 定义个人空间、文件系统、聊天器、日程和设置。 |
| Agent Team 与执行 | [06-agent-team-and-execution](./enterprise/space-system/06-agent-team-and-execution.md) | 定义 AI 总管、AI 秘书、AI 执行者、反馈收集者、创造者和权限继承。 |
| 五大界面 | [07-ui-surfaces](./enterprise/space-system/07-ui-surfaces.md) | 定义对象聊天、日程功能库、执行器、组织管理、用户主页。 |
| 权限引擎 | [08-permission-engine](./enterprise/space-system/08-permission-engine.md) | 定义权限判断、授权、委托、风险分级和审计。 |
| 数据模型 | [09-data-model](./enterprise/space-system/09-data-model.md) | 定义核心数据库表草案。 |

---

## 3. 开发者架构层：总览文档

| 文档 | 说明 |
|---|---|
| [architecture.md](./architecture.md) | GESA 企业自进化服务架构总览。 |
| [open-source-landscape.md](./open-source-landscape.md) | 按层级和模块映射开源方案。 |
| [knowledge-base-layer.md](./knowledge-base-layer.md) | 知识库层补充设计，说明为什么知识库是第二基础。 |

---

## 4. 核心引擎文档

| 引擎 | 文档 | 定位 |
|---|---|---|
| Core Engine Layer | [Core Engine Layer](./engines/core-engine-layer.md) | 横向基础能力抽象，将 Database、Semantic、Agent、Workflow、Issue、Policy、Operation、Deployment 等能力组织为同级核心引擎。 |
| Semantic Engine | [Semantic Engine](./engines/semantic-engine.md) | 语义记忆与上下文召回引擎。BGE-M3 可作为默认 embedding 模型候选，与 Agent Engine 同级。 |

---

## 5. 层级文档

| 层级 | 文档 | 定位 |
|---|---|---|
| L1 | [Database Layer](./layers/l1-database-layer.md) | 第一基础：企业事实状态源。 |
| L1.5 | [Knowledge Base Layer](./layers/l1-5-knowledge-base-layer.md) | 第二基础：企业认知结构源。 |
| L2 | [Object & Permission Layer](./layers/l2-object-permission-layer.md) | 实体、权限、Agent 和操作注册治理层。 |
| L3 | [Atomic Operation Layer](./layers/l3-atomic-operation-layer.md) | 原子操作定义、执行和审计层。 |
| L4 | [Workspace Layer](./layers/l4-workspace-layer.md) | 工作空间、工作流、日程、可视化和秘书 Agent 层。 |
| L5 | [Feedback Layer](./layers/l5-feedback-layer.md) | 反馈收集、分类、分流和进化输入层。 |
| M1 | [Project Control Plane](./layers/m1-project-control-plane.md) | 项目安装、部署、模块、规则和回滚控制平面。 |
| M2 | [Self-Evolution Engine](./layers/m2-self-evolution-engine.md) | 外部 Coding Agent 驱动的项目自进化引擎。 |

---

## 6. 组合方案文档

| 方案 | 文档 | 适用场景 |
|---|---|---|
| 轻量级方案 | [Lightweight SQLite Stack](./solutions/lightweight-sqlite-stack.md) | 单机、小团队、客户本地试点、早期 MVP。 |
| MVP 方案 | [MVP Stack](./solutions/mvp-stack.md) | 小型企业、多用户协作、早期商业化。 |
| 企业级方案 | [Enterprise Stack](./solutions/enterprise-stack.md) | 中大型企业、多部门、强权限、强审计、高可用。 |

---

## 7. 推荐阅读顺序

### 企业侧

```text
1. README.md
2. enterprise/README.md
3. enterprise/space-system/README.md
4. enterprise/space-system/01-positioning.md
5. enterprise/space-system/03-organization-permission-graph.md
6. enterprise/space-system/04-space-types-and-resources.md
7. enterprise/space-system/07-ui-surfaces.md
```

### 开发侧

```text
1. README.md
2. developer/README.md
3. architecture.md
4. engines/core-engine-layer.md
5. engines/semantic-engine.md
6. layers/l1-database-layer.md
7. layers/l1-5-knowledge-base-layer.md
8. layers/l2-object-permission-layer.md
9. layers/l3-atomic-operation-layer.md
10. layers/l4-workspace-layer.md
11. layers/l5-feedback-layer.md
12. layers/m1-project-control-plane.md
13. layers/m2-self-evolution-engine.md
```

---

## 8. 当前核心判断

GESA 现在应明确分为两面：

```text
企业架构层：企业对象、组织权限图、空间系统、个人工作区、Agent Team、界面与任务执行。
开发者架构层：数据库、知识库、权限服务、核心引擎、Issue、部署、自进化、代码实现。
```

底层仍然共享这些核心对象：

```text
Entity
Space
Resource
Permission
Operation
Workflow
Knowledge
Semantic
Agent
Issue
Deployment
Audit
```

---

## 9. 文档维护原则

```text
1. 面向开发者的文档与面向企业的文档分开。
2. 每个层级独立成文档。
3. 每个核心引擎独立成文档。
4. 每个企业侧大模块独立成目录。
5. 单个文档只解释一个层级、一个方向或一个作用。
6. 总览文档保留架构骨架。
7. 具体落地细节进入对应层级、引擎、空间、界面或数据模型文档。
8. 后续每次新增模块、操作、工作流、Agent、知识结构、语义索引能力，都应同步更新相关文档。
```