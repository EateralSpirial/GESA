# GESA 文档索引

> Version: 0.1  
> Purpose: 快速定位 GESA 架构、层级设计、开源方案和组合方案文档。

---

## 1. 总览文档

| 文档 | 说明 |
|---|---|
| [architecture.md](./architecture.md) | GESA 企业自进化服务架构总览。 |
| [open-source-landscape.md](./open-source-landscape.md) | 按层级和模块映射开源方案。 |
| [knowledge-base-layer.md](./knowledge-base-layer.md) | 知识库层补充设计，说明为什么知识库是第二基础。 |

---

## 2. 层级文档

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

## 3. 组合方案文档

| 方案 | 文档 | 适用场景 |
|---|---|---|
| 轻量级方案 | [Lightweight SQLite Stack](./solutions/lightweight-sqlite-stack.md) | 单机、小团队、客户本地试点、早期 MVP。 |
| MVP 方案 | [MVP Stack](./solutions/mvp-stack.md) | 小型企业、多用户协作、早期商业化。 |
| 企业级方案 | [Enterprise Stack](./solutions/enterprise-stack.md) | 中大型企业、多部门、强权限、强审计、高可用。 |

---

## 4. 推荐阅读顺序

```text
1. architecture.md
2. knowledge-base-layer.md
3. layers/l1-database-layer.md
4. layers/l1-5-knowledge-base-layer.md
5. layers/l2-object-permission-layer.md
6. layers/l3-atomic-operation-layer.md
7. layers/l4-workspace-layer.md
8. layers/l5-feedback-layer.md
9. layers/m1-project-control-plane.md
10. layers/m2-self-evolution-engine.md
11. open-source-landscape.md
12. solutions/lightweight-sqlite-stack.md
13. solutions/mvp-stack.md
14. solutions/enterprise-stack.md
```

---

## 5. 当前核心判断

GESA 的基础不应只有数据库。更准确的基础结构是：

```text
第一基础：Database Layer
- 保存事实、状态、事件、审计、指标。

第二基础：Knowledge Base Layer
- 保存文档、规则、经验、解释、语义索引、知识关系。
```

首批核心 Schema 应为：

```text
Entity
Knowledge
Operation
Permission
Workflow
Feedback
```

---

## 6. 文档维护原则

```text
每个层级独立成文档。
每套组合方案独立成文档。
总览文档只保留架构骨架。
开源方案文档只负责横向比较。
具体落地细节进入对应层级或方案文档。
后续每次新增模块、操作、工作流、Agent、知识结构，都应同步更新相关文档。
```
