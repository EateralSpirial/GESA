# GESA 开发者架构层

> Version: 0.3  
> Audience: 开发者、维护者、Coding Agent、部署 Agent。  
> Scope: 项目仓库内部的技术架构、核心引擎、层级、组合方案与自进化机制。

---

## 1. 定位

开发者架构层解释 GESA 如何被实现、维护、部署和自我迭代。

它主要回答：

```text
1. 数据如何存储？
2. 权限如何判断？
3. 知识库如何检索？
4. Agent 如何获得上下文？
5. Issue 如何进入自进化队列？
6. 原子操作如何注册、执行和审计？
7. 部署、回滚、测试和 PR 如何形成闭环？
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

## 3. 现有开发者文档入口

| 分类 | 文档 |
|---|---|
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

## 4. 开发者文档维护原则

```text
1. 每个层级独立成文档。
2. 每个核心引擎独立成文档。
3. 每套部署组合方案独立成文档。
4. 技术实现细节不写入企业侧文档。
5. 企业侧对象进入开发实现时，应落到 Entity、Space、Resource、Permission、Operation、Audit 等基础对象上。
```