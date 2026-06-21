# GESA 文档索引

> Version: 0.2  
> Purpose: 快速定位 GESA 架构、核心引擎、层级设计、开源方案和组合方案文档。

---

## 1. 总览文档

| 文档 | 说明 |
|---|---|
| [architecture.md](./architecture.md) | GESA 企业自进化服务架构总览。 |
| [open-source-landscape.md](./open-source-landscape.md) | 按层级和模块映射开源方案。 |
| [knowledge-base-layer.md](./knowledge-base-layer.md) | 知识库层补充设计，说明为什么知识库是第二基础。 |

---

## 2. 核心引擎文档

| 引擎 | 文档 | 定位 |
|---|---|---|
| Core Engine Layer | [Core Engine Layer](./engines/core-engine-layer.md) | 横向基础能力抽象，将 Database、Semantic、Agent、Workflow、Issue、Policy、Operation、Deployment 等能力组织为同级核心引擎。 |
| Semantic Engine | [Semantic Engine](./engines/semantic-engine.md) | 语义记忆与上下文召回引擎。BGE-M3 可作为默认 embedding 模型候选，与 Agent Engine 同级。 |

---

## 3. 层级文档

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

## 4. 组合方案文档

| 方案 | 文档 | 适用场景 |
|---|---|---|
| 轻量级方案 | [Lightweight SQLite Stack](./solutions/lightweight-sqlite-stack.md) | 单机、小团队、客户本地试点、早期 MVP。 |
| MVP 方案 | [MVP Stack](./solutions/mvp-stack.md) | 小型企业、多用户协作、早期商业化。 |
| 企业级方案 | [Enterprise Stack](./solutions/enterprise-stack.md) | 中大型企业、多部门、强权限、强审计、高可用。 |

---

## 5. 推荐阅读顺序

```text
1. architecture.md
2. engines/core-engine-layer.md
3. engines/semantic-engine.md
4. knowledge-base-layer.md
5. layers/l1-database-layer.md
6. layers/l1-5-knowledge-base-layer.md
7. layers/l2-object-permission-layer.md
8. layers/l3-atomic-operation-layer.md
9. layers/l4-workspace-layer.md
10. layers/l5-feedback-layer.md
11. layers/m1-project-control-plane.md
12. layers/m2-self-evolution-engine.md
13. open-source-landscape.md
14. solutions/lightweight-sqlite-stack.md
15. solutions/mvp-stack.md
16. solutions/enterprise-stack.md
```

---

## 6. 当前核心判断

GESA 的基础不应只有数据库，也不应只按垂直层级理解。更准确的结构是：

```text
第一基础：Database Layer
- 保存事实、状态、事件、审计、指标。

第二基础：Knowledge Base Layer
- 保存文档、规则、经验、解释、语义索引、知识关系。

横向基础：Core Engine Layer
- 将 Database Engine、Semantic Engine、Agent Engine、Workflow Engine、Issue Engine、Policy Engine、Operation Engine、Deployment Engine 作为同级核心能力治理。
```

其中最新修正是：

```text
Semantic Engine 不应只是 Knowledge Base Layer 的普通子模块。
Semantic Engine 应作为语义记忆与上下文召回引擎，与 Agent Engine 同级。

Agent Engine    = 行动智能
Semantic Engine = 记忆智能
Database Engine = 事实状态
Workflow Engine = 流程秩序
Issue Engine    = 演化账本
Policy Engine   = 安全边界
```

首批核心 Schema / Engine 组合应为：

```text
Entity
Knowledge
Operation
Permission
Workflow
Feedback
Semantic
Agent
Issue
Deployment
```

---

## 7. 文档维护原则

```text
每个层级独立成文档。
每个核心引擎独立成文档。
每套组合方案独立成文档。
总览文档保留架构骨架。
核心引擎文档说明横向基础能力和调用边界。
层级文档说明业务系统结构和职责边界。
开源方案文档只负责横向比较。
具体落地细节进入对应层级、引擎或方案文档。
后续每次新增模块、操作、工作流、Agent、知识结构、语义索引能力，都应同步更新相关文档。
```
