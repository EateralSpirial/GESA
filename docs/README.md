# GESA 文档系统

> Version: 0.3  
> Purpose: 为 GESA 建立分面的、分层级的立体文档入口。

---

## 1. 文档分面

GESA 同时存在两套面向不同读者的架构文档。

| 分面 | 目录 | 主要读者 | 说明 |
|---|---|---|---|
| 开发者架构层 | [developer/](./developer/) | 开发者、维护者、Coding Agent、部署 Agent | 描述项目仓库、核心引擎、数据库层、知识库层、权限层、反馈层、自进化层等技术结构。 |
| 企业架构层 | [enterprise/](./enterprise/) | CEO、管理者、部门首脑、企业管理员、业务用户、企业侧 Agent | 描述组织权限图、共享空间、个人工作区、对象聊天、日程、执行器和企业可视化管理界面。 |

---

## 2. 推荐阅读路径

```text
1. documentation-index.md
2. developer/README.md
3. enterprise/README.md
4. enterprise/space-system/README.md
```

开发者优先阅读：

```text
developer/README.md
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
5. 后续新增模块时，优先新增独立文档，再回填索引。
```

---

## 4. 当前目录结构建议

```text
docs/
├── README.md
├── documentation-index.md
├── developer/
│   ├── README.md
│   └── documentation-boundary.md
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

GESA 需要从项目早期就区分两种架构语言：

```text
开发者架构语言：服务、引擎、层级、Schema、API、部署、Issue、测试、PR、自进化。
企业架构语言：组织、部门、层级、方向、空间、权限、对象、任务、日程、Agent、审计。
```

这两套语言应通过底层对象模型和权限系统连接，但不应在文档组织上混写。