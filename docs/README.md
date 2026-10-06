# GESA 文档系统

> Version: 0.4  
> Purpose: 为 GESA 建立分面的、分层级的立体文档入口，并提供从概念架构进入可部署实体的基座路径。

---

## 1. 文档分面

GESA 同时存在两套面向不同读者的架构文档，并由可部署基座连接。

| 分面 | 目录 | 主要读者 | 说明 |
|---|---|---|---|
| 可部署基座 | [developer/foundation/](./developer/foundation/) | 核心开发者、架构师、部署维护者 | 定义信息流内核、Rust 基座、中央控制平面、企业运行节点、终端、模块、更新和安全边界。 |
| 开发者架构层 | [developer/](./developer/) | 开发者、维护者、Coding Agent、部署 Agent | 描述项目仓库、核心引擎、数据库层、知识库层、权限层、反馈层、自进化层等技术结构。 |
| 企业架构层 | [enterprise/](./enterprise/) | CEO、管理者、部门首脑、企业管理员、业务用户、企业侧 Agent | 描述组织权限图、共享空间、个人工作区、对象聊天、日程、执行器和企业可视化管理界面。 |

---

## 2. 推荐阅读路径

从概念进入实体构建：

```text
1. documentation-index.md
2. developer/README.md
3. developer/foundation/01-deployable-base-system-design.md
4. enterprise/space-system/README.md
```

开发者优先阅读：

```text
developer/README.md
developer/foundation/
layers/
engines/
solutions/
```

企业侧优先阅读：

```text
enterprise/README.md
enterprise/space-system/
```

---

## 3. 文档拆分原则

```text
1. 面向开发者的文档与面向企业的文档分开。
2. 单个文档只解释一个层级、一个方向或一个作用。
3. 总览文档只保留导航和边界，不承载过多细节。
4. 复杂概念拆成对象模型、权限模型、数据模型、界面模型、执行模型。
5. 可部署基座先给出整体契约，再将信息流、对象、权限、模块、更新、安全和运维拆为独立文档。
6. 后续新增模块时，优先新增独立文档，再回填索引。
```

---

## 4. 当前目录结构建议

```text
docs/
├── README.md
├── documentation-index.md
├── developer/
│   ├── README.md
│   ├── documentation-boundary.md
│   └── foundation/
│       └── 01-deployable-base-system-design.md
├── enterprise/
│   ├── README.md
│   └── space-system/
│       ├── README.md
│       ├── 01-positioning.md
│       ├── 02-object-model.md
│       ├── 03-organization-permission-graph.md
│       ├── 04-space-types-and-resources.md
│       ├── 05-personal-workspace.md
│       ├── 06-agent-team-and-execution.md
│       ├── 07-ui-surfaces.md
│       ├── 08-permission-engine.md
│       └── 09-data-model.md
├── engines/
├── layers/
└── solutions/
```

---

## 5. 核心判断

GESA 需要从项目早期区分三种架构语言：

```text
开发者架构语言：
服务、引擎、层级、Schema、API、部署、Issue、测试、PR、自进化。

企业架构语言：
组织、部门、层级、方向、空间、权限、对象、任务、日程、Agent、审计。

基座架构语言：
信息流、契约、模块、Profile、节点、发布、更新、回滚、隐私边界。
```

三种语言通过统一对象模型、信息流契约和确定性权限系统连接，并在文档组织上保持清晰边界。
