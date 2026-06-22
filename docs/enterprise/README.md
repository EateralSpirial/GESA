# GESA 企业架构层

> Version: 0.1  
> Audience: CEO、企业管理者、部门首脑、企业管理员、业务用户、企业侧 Agent。  
> Scope: 企业如何使用 GESA 管理组织、权限、空间、任务、日程、Agent 和共享资源。

---

## 1. 定位

企业架构层描述 GESA 在企业内部呈现出来的使用形态。

它关注：

```text
1. 组织如何被表示？
2. 部门如何跨方向、跨层级存在？
3. 权限如何可视化管理？
4. 人类对象与 Agent 对象如何协作？
5. 个人工作区与共享空间如何隔离？
6. 任务、日程、脚本、文件、知识库、数据库如何被放入空间？
```

---

## 2. 当前企业侧文档

| 模块 | 文档 | 说明 |
|---|---|---|
| 空间系统 | [space-system/](./space-system/) | GESA 企业空间系统，包含组织权限图、共享空间、个人工作区、Agent Team 和五大界面。 |

---

## 3. 企业架构层与开发者架构层的关系

```text
企业架构层：用户看到和操作的企业结构。
开发者架构层：仓库中实现这些结构的服务、引擎和数据模型。
```

企业架构层中的对象最终会映射到开发者架构层：

```text
企业对象        → Entity / Space / Resource / Permission / Operation
组织权限图      → Permission Engine + Org Graph Service
共享空间        → Space Service + Resource Service
个人工作区      → Workspace Layer + File System Object
Agent Team      → Agent Engine + Delegated Permission Session
执行器          → Operation Engine + Executor Service
知识库          → Knowledge Base Layer + Semantic Engine
```

---

## 4. 企业侧文档维护原则

```text
1. 用企业能理解的对象语言写文档。
2. 不把具体代码实现、框架选型、部署细节混入企业侧文档。
3. 对每个大概念建立独立目录。
4. 对复杂模块拆成定位、对象模型、权限模型、界面模型、数据模型。
5. 企业侧规则必须能映射到开发侧确定性对象。
```