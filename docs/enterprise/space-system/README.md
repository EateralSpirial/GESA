# GESA 企业空间系统

> Version: 0.1  
> Position: Enterprise Architecture Layer / User-Facing Control Plane

---

## 1. 核心定义

GESA 企业空间系统是企业侧的第一入口。它把组织、权限、空间、对象、Agent、日程、任务和共享资源统一到一个可视化系统中。

```text
GESA 企业空间系统 = 组织权限图 + 共享空间系统 + 个人工作区 + Agent Team + 企业可视化界面。
```

---

## 2. 文档拆分

| 序号 | 文档 | 作用 |
|---|---|---|
| 01 | [定位与边界](./01-positioning.md) | 说明企业空间系统在 GESA 中的位置。 |
| 02 | [对象模型](./02-object-model.md) | 定义实体、空间、资源、权限、聊天、日程、任务等基础对象。 |
| 03 | [组织权限图](./03-organization-permission-graph.md) | 定义层级、方向、格子、部门、投影和 CEO 规则。 |
| 04 | [空间类型与资源](./04-space-types-and-resources.md) | 定义个人、格子、部门、方向、层级、全局空间及其资源。 |
| 05 | [个人工作区](./05-personal-workspace.md) | 定义个人空间、文件系统、聊天器、日程和设置。 |
| 06 | [Agent Team 与执行](./06-agent-team-and-execution.md) | 定义 AI 总管、AI 秘书、执行者、倾听者、创造者和权限继承。 |
| 07 | [五大界面](./07-ui-surfaces.md) | 定义对象聊天、日程功能库、执行器、组织管理、用户主页。 |
| 08 | [权限引擎](./08-permission-engine.md) | 定义权限判断、授权、委托、风险分级和审计。 |
| 09 | [数据模型](./09-data-model.md) | 定义核心数据库表草案。 |

---

## 3. 与开发者架构的映射

| 企业空间系统概念 | 开发者架构对应层 |
|---|---|
| 组织权限图 | Object & Permission Layer |
| 共享空间 | Workspace Layer / Resource Service |
| 个人工作区 | Workspace Layer |
| Agent Team | Agent Engine |
| 知识库对象 | Knowledge Base Layer / Semantic Engine |
| 数据库对象 | Database Layer |
| 执行器对象 | Atomic Operation Layer / Operation Engine |
| 项目自我迭代状态 | Self-Evolution Engine / Issue Engine / Deployment Engine |

---

## 4. 第一版实现重点

```text
1. 实体对象：人类与 Agent。
2. 组织图：层级、方向、格子、部门。
3. 空间：个人空间、格子空间、部门空间。
4. 权限：CEO、最高权限管理器、部门首脑、授权器。
5. 资源：文件、知识库、数据库、执行器、Agent Team。
6. 界面：对象聊天、日程功能库、执行器、组织架构、用户主页。
7. 审计：所有 Agent 代用户执行行为必须记录。
```