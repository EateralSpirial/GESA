# 开发者文档与企业文档边界

> Version: 0.1  
> Purpose: 防止 GESA 文档在项目变复杂后混合叙事、重复叙事和过长叙事。

---

## 1. 两个分面

GESA 的文档系统分为两面：

```text
开发者架构层：解释系统如何实现。
企业架构层：解释系统如何被企业使用和治理。
```

两者共享同一套底层对象，但面向不同读者。

---

## 2. 写入开发者文档的内容

以下内容进入 `docs/developer/`、`docs/layers/`、`docs/engines/` 或 `docs/solutions/`：

```text
1. 服务拆分
2. 数据库 Schema
3. API 设计
4. 权限引擎实现
5. Agent Engine
6. Semantic Engine
7. Issue Engine
8. Deployment Engine
9. 自进化流程
10. GitHub Actions / PR / CI / CD
11. Docker / SQLite / PostgreSQL / Qdrant / Milvus 等实现方案
```

---

## 3. 写入企业文档的内容

以下内容进入 `docs/enterprise/`：

```text
1. CEO 与最高权限管理器
2. 层级、方向、格子、部门
3. 组织权限图
4. 共享空间
5. 个人工作区
6. 对象聊天
7. 日程表与功能库
8. 执行器
9. 企业可视化管理界面
10. 人类实体与 Agent 实体的组织关系
```

---

## 4. 交叉概念如何处理

交叉概念允许同时出现在两边，但写法不同。

| 概念 | 企业侧写法 | 开发侧写法 |
|---|---|---|
| 权限 | 谁能看、谁能写、谁能管理、谁能授权 | RBAC / ABAC / Policy / Grant / Audit |
| 空间 | 部门空间、格子空间、个人工作区 | Space Service / Resource Binding / Storage Policy |
| Agent | AI 总管、AI 秘书、AI 执行者 | Agent Engine / Tool Permission / Execution Session |
| 知识库 | 共享知识、部门知识、可检索资料 | Knowledge Base Layer / Semantic Engine / BGE-M3 |
| 执行器 | 任务执行、浏览器操作、脚本执行 | Operation Engine / Sandbox / Job Queue |

---

## 5. 文档粒度规则

```text
1. 一个文档只解释一个核心概念。
2. 一个文档超过约 300 行时，应考虑拆分。
3. 总览文档不承载细节。
4. 数据模型、权限模型、对象模型、界面模型、流程模型应分开。
5. 新增大概念时，先建目录，再建 README，再拆子文档。
```

---

## 6. 推荐新增文档流程

```text
1. 判断文档属于企业侧还是开发侧。
2. 判断它是层级、引擎、空间、对象、流程、界面还是方案。
3. 新建短文档。
4. 在对应 README 中加入入口。
5. 在 documentation-index.md 中加入一级入口。
```