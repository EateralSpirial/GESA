# GESA 开发者架构层

> Version: 0.4  
> Audience: 开发者、维护者、Coding Agent、部署 Agent。  
> Scope: 项目仓库内部的技术架构、可部署基座、核心引擎、层级、组合方案与自进化机制。

---

## 1. 定位

开发者架构层解释 GESA 如何被实现、维护、部署和自我迭代。

它主要回答：

```text
1. GESA 第一份可部署实体由什么组成？
2. 数据如何存储？
3. 权限如何判断？
4. 知识库如何检索？
5. Agent 如何获得上下文？
6. Issue 如何进入自进化队列？
7. 原子操作如何注册、执行和审计？
8. 部署、更新、回滚、测试和 PR 如何形成闭环？
```

---

## 2. 与企业架构层的边界

企业架构层描述用户可理解的企业空间、组织、部门、权限和界面。

开发者架构层描述这些能力背后的技术实现：

| 企业侧概念 | 开发侧对应实现 |
|---|---|
| 组织权限图 | Permission Service / Object Permission Layer |
| 共享空间 | Workspace Layer / Resource Service |
| 个人工作区 | Entity Workspace / File System Resource |
| Agent Team | Agent Engine / Delegated Permission Session |
| 对象聊天 | Chat Service / Entity Graph |
| 日程与功能库 | Workflow Engine / Schedule Module / Script Library |
| 执行器 | Operation Engine / Executor Service |
| 自我迭代 | Issue Engine / Self-Evolution Engine / Deployment Engine |

---

## 3. 可部署基座入口

| 文档 | 定位 |
|---|---|
| [GESA 可部署基座系统设计](./foundation/01-deployable-base-system-design.md) | 定义信息流内核、Rust 模块化单体、中央控制平面、企业运行节点、Windows/Linux 终端、企业剖面、更新和安全边界。 |

该文档是从架构研究进入实体构建的第一入口。后续对象模型、信息流契约、模块 SDK、更新平面和安全模型将继续拆分到 `foundation/` 目录。

---

## 4. 现有开发者文档入口

| 分类 | 文档 |
|---|---|
| 可部署基座 | [foundation/01-deployable-base-system-design.md](./foundation/01-deployable-base-system-design.md) |
| 总览 | [../architecture.md](../architecture.md) |
| 总文档索引 | [../documentation-index.md](../documentation-index.md) |
| 核心引擎层 | [../engines/core-engine-layer.md](../engines/core-engine-layer.md) |
| 语义引擎 | [../engines/semantic-engine.md](../engines/semantic-engine.md) |
| 数据库层 | [../layers/l1-database-layer.md](../layers/l1-database-layer.md) |
| 知识库层 | [../layers/l1-5-knowledge-base-layer.md](../layers/l1-5-knowledge-base-layer.md) |
| 对象与权限层 | [../layers/l2-object-permission-layer.md](../layers/l2-object-permission-layer.md) |
| 原子操作层 | [../layers/l3-atomic-operation-layer.md](../layers/l3-atomic-operation-layer.md) |
| 工作空间层 | [../layers/l4-workspace-layer.md](../layers/l4-workspace-layer.md) |
| 反馈层 | [../layers/l5-feedback-layer.md](../layers/l5-feedback-layer.md) |
| 项目控制平面 | [../layers/m1-project-control-plane.md](../layers/m1-project-control-plane.md) |
| 自进化引擎 | [../layers/m2-self-evolution-engine.md](../layers/m2-self-evolution-engine.md) |
| 轻量级方案 | [../solutions/lightweight-sqlite-stack.md](../solutions/lightweight-sqlite-stack.md) |
| MVP 方案 | [../solutions/mvp-stack.md](../solutions/mvp-stack.md) |
| 企业级方案 | [../solutions/enterprise-stack.md](../solutions/enterprise-stack.md) |

---

## 5. 开发者文档维护原则

```text
1. 每个层级独立成文档。
2. 每个核心引擎独立成文档。
3. 每套部署组合方案独立成文档。
4. 技术实现细节不写入企业侧文档。
5. 企业侧对象进入开发实现时，应落到 GesaObject、Entity、Space、Resource、Permission、Operation、Event、Audit 等基础对象上。
6. 基座总览只锁定边界、契约和实施顺序；具体 Schema、协议、威胁模型和运维流程进入 foundation/ 独立文档。
```
