# GESA 企业自进化服务架构

> Version: 0.2  
> Date: 2026-06-16  
> Status: Architecture Draft

GESA，全称 **General Enterprise Service Architecture**，目标是构建一套可长期演化的企业服务系统。系统以数据库为企业状态源，以对象与权限层统一管理所有实体，以原子操作层提供可审计动作，以工作空间层承载秘书 Agent 与可视化协作，以反馈层形成进化输入，再由项目控制平面和外部自进化引擎完成项目级迭代。

---

## 1. 总体目标

GESA 的核心目标是：

1. 将企业中的真人、AI Agent、部门、项目、客户、订单、合同、设备、服务节点等对象统一建模为实体。
2. 为每个实体对象提供可管理的工作空间。
3. 为真人实体提供可视化秘书界面。
4. 为 AI Agent 实体提供任务队列、工具权限、执行记录与失败复盘机制。
5. 以周为基本管理周期，对实体对象进行目标、日程、任务、工作流与复盘管理。
6. 将所有基础操作原子化、权限化、审计化。
7. 将所有反馈结构化收集，提交给项目自进化引擎作为迭代依据。
8. 通过外部独立 Coding Agent 驱动项目自进化，避免项目内部 Agent 直接修改项目本体。

---

## 2. 架构总览

GESA 分为两个大域：

- **Runtime Plane：企业运行系统**
- **Meta Plane：项目元管理系统**

```text
┌──────────────────────────────────────────────┐
│  M2. Self-Evolution Engine                    │
│  独立 Coding Agent / 外部驱动 / 不依赖内部 API │
└──────────────────────▲───────────────────────┘
                       │
┌──────────────────────┴───────────────────────┐
│  M1. Project Control Plane                    │
│  模块协调 / 增量规则 / 安装部署 / 迭代编排      │
└──────────────────────▲───────────────────────┘
                       │
═══════════════════════╪════════════════════════
              Runtime Plane
═══════════════════════╪════════════════════════
                       │
┌──────────────────────┴───────────────────────┐
│  L5. Feedback Layer                           │
│  反馈收集 / 分类 / 归因 / 进化候选提交          │
└──────────────────────▲───────────────────────┘
                       │
┌──────────────────────┴───────────────────────┐
│  L4. Workspace Layer                          │
│  周计划 / 时间调度 / 工作流 / 可视化操作 / 秘书 │
└──────────────────────▲───────────────────────┘
                       │
┌──────────────────────┴───────────────────────┐
│  L3. Atomic Operation Layer                   │
│  操作定义 / 操作执行 / 操作审计 / 工具引擎      │
└──────────────────────▲───────────────────────┘
                       │
┌──────────────────────┴───────────────────────┐
│  L2. Object & Permission Layer                │
│  实体管理 / 权限管理 / Agent 管理 / 操作注册    │
└──────────────────────▲───────────────────────┘
                       │
┌──────────────────────┴───────────────────────┐
│  L1. Database Layer                           │
│  状态 / 事件 / 审计 / 记忆 / 文件索引 / 指标    │
└──────────────────────────────────────────────┘
```

---

## 3. Runtime Plane

Runtime Plane 是企业实际运行的主体系统，负责企业对象、权限、操作、工作流、日程、可视化协作与反馈收集。

---

# L1. Database Layer

数据库层是企业现实的唯一状态源，负责保存系统中的所有事实状态。

## 3.1.1 职责

数据库层保存：

- 实体状态
- 权限状态
- 任务状态
- 工作流状态
- 日程状态
- 操作记录
- 反馈记录
- AI Agent 记录
- 项目模块记录
- 自进化候选记录
- 审计日志
- 记忆与上下文索引
- 文件与产物索引
- 指标与统计数据

## 3.1.2 推荐核心表

```text
entities              所有实体对象
entity_profiles       实体画像
entity_relations      实体之间的关系
permissions           权限规则
operations            操作定义
operation_logs        操作日志
tasks                 任务
workflows             工作流
workflow_runs         工作流执行记录
calendar_items        日程
feedbacks             反馈
agent_profiles        AI Agent 档案
agent_sessions        AI Agent 会话
module_registry       项目模块注册表
evolution_candidates  自进化候选项
audit_logs            审计日志
memories              记忆记录
artifacts             文件与产物索引
metrics               指标记录
```

## 3.1.3 设计原则

1. 所有上层模块都围绕数据库读写、订阅与更新。
2. 所有重要变更必须写入事件与审计记录。
3. 数据库不直接承担复杂决策。
4. 数据库负责保存事实，上层模块负责解释事实。
5. 数据库 schema 应当支持版本化迁移。

---

# L2. Object & Permission Layer

对象与权限层统一管理所有实体对象、权限策略、AI Agent 接入与基础操作注册。

---

## 3.2.1 Entity Management Module

所有被系统管理的对象都注册为 Entity。

### 实体类型

```text
human                 真人
ai_agent              AI Agent
department            部门
project               项目
customer              客户
contract              合同
order                 订单
inventory_item        库存对象
device                设备
service_node          服务节点
file                  文件
workflow              工作流
task                  任务
```

### 实体基础字段

```text
entity_id
entity_type
name
status
owner_entity_id
parent_entity_id
metadata
created_at
updated_at
```

### 实体管理职责

- 创建实体
- 更新实体状态
- 维护实体关系
- 查询实体画像
- 绑定实体工作空间
- 绑定实体秘书 Agent
- 绑定实体权限策略

---

## 3.2.2 Permission Management Module

权限模块位于对象管理层，并对所有上层操作形成强制约束。

### 权限判断结构

```text
Subject      谁发起操作
Object       操作作用于谁
Action       做什么
Scope        在哪个范围内做
Condition    什么情况下允许
Approval     是否需要审批
Audit        是否必须记录
```

### 权限示例

```text
张三 可以 查看 客户A 的基本信息
张三 可以 修改 自己负责客户 的跟进状态
销售经理 可以 批准 报价折扣 小于 10%
财务 Agent 可以 读取 发票数据
财务 Agent 不可 对外发送付款指令
Coding Agent 可以 修改代码仓库
项目内部 Agent 不可 修改自进化引擎
```

### 权限原则

1. 所有操作必须经过权限检查。
2. AI Agent 默认最小权限。
3. 高风险操作必须进入审批流程。
4. 权限变更必须记录审计日志。
5. 权限策略应支持实体类型、角色、资源范围和上下文条件。

---

## 3.2.3 AI Agent Management Module

AI Agent 作为实体对象统一管理。

### Agent 档案字段

```text
agent_id
entity_id
agent_type
capability_scope
available_tools
permission_policy_id
context_scope
task_queue_id
cost_limit
failure_policy
model_provider
model_name
version
health_status
created_at
updated_at
```

### Agent 管理职责

- 注册 Agent
- 绑定 Agent 能力范围
- 绑定工具权限
- 设置上下文范围
- 管理任务队列
- 记录调用成本
- 记录失败原因
- 提供健康状态
- 提供版本记录

### Agent 权限原则

AI Agent 不天然拥有系统权限。  
AI Agent 必须通过权限模块获得可执行操作范围。

---

## 3.2.4 Basic Operation Registry Module

基础操作管理模块负责维护系统的操作注册表。

每个基础操作需要记录：

```text
operation_id
operation_name
operation_category
operation_description
required_permission
execution_engine
parameter_schema
result_schema
approval_required
audit_required
agent_callable
workflow_callable
ui_callable
risk_level
created_at
updated_at
```

### 操作注册职责

- 记录系统有哪些基础操作
- 为操作绑定类别
- 为操作绑定权限要求
- 为操作绑定执行引擎
- 定义参数 schema
- 定义结果 schema
- 定义审计规则
- 定义是否允许被 Agent 调用
- 定义是否允许进入工作流

---

# L3. Atomic Operation Layer

实体基础操作层是系统的动作能力层，负责定义、执行并审计不可分割操作。

---

## 3.3.1 原子操作定义

原子操作是系统中不可继续拆分的基础动作。

例如：

```text
读取客户信息
创建任务
修改任务状态
发送邮件
打开网页
点击按钮
填写表单
生成报价单
提交审批
创建日程
修改工作流节点
调用浏览器
调用数据库查询
读取文件
写入文件
创建 GitHub Issue
提交 Pull Request
部署服务
```

---

## 3.3.2 操作三分法

基础操作层必须区分：

```text
Operation Definition   操作定义
Operation Execution    操作执行
Operation Audit        操作审计
```

### Operation Definition

描述系统允许做什么。

### Operation Execution

调用具体执行引擎完成动作。

### Operation Audit

记录谁在什么时候、基于什么权限、对什么对象、执行了什么操作、结果如何。

---

## 3.3.3 操作分类

建议基础操作分为以下类别：

| Category | 中文 | 示例 |
|---|---|---|
| Read | 读取 | 查询客户、读取文件 |
| Write | 写入 | 修改状态、创建任务 |
| Execute | 执行 | 运行脚本、调用工具 |
| Communicate | 通信 | 发邮件、发消息 |
| Schedule | 调度 | 创建日程、排班 |
| Approve | 审批 | 批准付款、确认合同 |
| Delegate | 委托 | 转交任务、分配责任人 |
| Configure | 配置 | 修改规则、调整权限 |
| Evolve | 进化 | 创建改进候选、触发迭代 |

---

## 3.3.4 操作引擎

基础操作层可以挂载多个操作引擎。

```text
Browser Use Engine
Database Query Engine
File Operation Engine
Email Engine
Calendar Engine
GitHub Engine
ERP Engine
Finance Engine
Document Engine
Notification Engine
Deployment Engine
```

### 标准执行链路

```text
请求操作
  ↓
权限检查
  ↓
参数校验
  ↓
审批判断
  ↓
调用操作引擎
  ↓
记录执行结果
  ↓
写入事件
  ↓
通知相关工作空间
```

---

# L4. Workspace Layer

工作空间层是秘书 Agent 的主工作层，负责时间调度、可视化安排、工作流创建、工作流管理、工作流修改和工作流整合。

---

## 3.4.1 Workspace 定义

每个实体都可以拥有工作空间。

```text
真人工作空间
AI Agent 工作空间
部门工作空间
项目工作空间
客户工作空间
合同工作空间
订单工作空间
设备工作空间
服务节点工作空间
```

不同实体的工作空间侧重点不同：

- 真人工作空间：可视化、提醒、审批、自然语言交互。
- AI Agent 工作空间：任务队列、工具调用、执行记录、失败复盘。
- 业务对象工作空间：状态生命周期、相关任务、责任人、风险提示。
- 项目工作空间：目标、里程碑、任务流、依赖关系、进度追踪。

---

## 3.4.2 Secretary Agent

秘书 Agent 位于工作空间层。

### 秘书 Agent 职责

```text
读取实体状态
生成周计划
整理任务池
安排日程
创建工作流
调用基础操作
请求审批
汇总反馈
生成复盘
提出改进建议
```

### 秘书 Agent 行动链路

```text
Secretary Agent
  ↓
Workspace Module
  ↓
Atomic Operation Layer
  ↓
Permission Management Module
  ↓
Database Layer
```

秘书 Agent 不应绕过权限与操作层。

---

## 3.4.3 Workspace 模块组成

```text
Workspace Core
- 工作空间创建
- 工作空间绑定实体
- 工作空间状态聚合

Workflow Module
- 工作流创建
- 工作流修改
- 工作流合并
- 工作流版本管理
- 工作流运行记录

Schedule Module
- 周计划
- 日程
- 截止时间
- 会议
- 排班
- 时间冲突检测

Visualization Module
- 看板
- 日历
- 时间线
- 审批入口
- 风险提示
- 操作按钮

Secretary Agent Module
- 周计划生成
- 任务协调
- 自然语言交互
- 复盘总结
- 反馈提交
```

---

## 3.4.4 周期管理

GESA 以周为基本管理周期。

推荐节奏：

```text
年度目标
  ↓
季度战役
  ↓
月度项目
  ↓
周工作流
  ↓
日任务 / 实时事件
```

每周循环：

```text
周一：生成本周计划
周二至周五：执行、提醒、协调、处理异常
周六：自动生成复盘
周日：沉淀经验，更新规则和工作流候选
```

---

## 3.4.5 无 Agent 兜底机制

工作空间层必须支持无 Agent 状态下的人工可视化操作。

```text
有 Agent：Agent 生成建议，用户确认或修改
无 Agent：系统展示按钮、表单、看板、日程
Agent 故障：回退到人工可视化操作
权限不足：显示申请权限或请求审批入口
```

兜底机制可以避免系统过度依赖 AI Agent 可用性。

---

# L5. Feedback Layer

使用反馈层是自进化系统的输入口。任何实体对象都可以提交反馈。

---

## 3.5.1 反馈来源

```text
真人反馈
AI Agent 失败反馈
客户反馈
任务超时反馈
工作流阻塞反馈
权限冲突反馈
UI 使用困难反馈
部署异常反馈
数据缺失反馈
安全风险反馈
```

---

## 3.5.2 反馈结构

```text
feedback_id
source_entity_id
target_entity_id
related_task_id
related_workflow_id
related_operation_id
feedback_type
severity
description
evidence
suggested_fix
created_at
status
```

---

## 3.5.3 反馈分类

```text
Bug
流程低效
权限不足
权限过宽
数据结构缺失
UI 不清晰
Agent 能力不足
工具调用失败
业务规则冲突
部署问题
安全风险
新增功能需求
```

---

## 3.5.4 反馈处理路径

```text
普通反馈 → 工作空间处理
严重反馈 → 管理者审批
系统性反馈 → 自进化候选项
安全反馈 → 阻断操作并升级
```

反馈层不能直接改变系统。  
反馈进入候选池后，由自进化流程处理。

---

# M1. Project Control Plane

项目控制平面负责管理整个项目如何被安装、部署、升级、拆分和扩展。

它属于 Meta Plane，不属于普通企业运行实体域。

---

## 4.1.1 职责

```text
项目安装
模块注册
模块依赖管理
模块版本管理
模块健康检查
模块升级策略
模块回滚策略
项目增量规则管理
部署协调
迁移协调
自进化候选接收
```

---

## 4.1.2 模块注册表

项目所有模块都应该注册到 Project Control Plane。

```text
数据库模块
对象管理模块
权限模块
Agent 管理模块
基础操作模块
工作空间模块
反馈模块
可视化模块
部署模块
自进化候选模块
```

每个模块记录：

```text
module_id
module_name
module_type
version
dependencies
migration_status
owner
health_status
upgrade_policy
rollback_policy
```

---

## 4.1.3 项目增量规则模块

项目不能随意进化。  
Project Control Plane 需要记录各种增量规则。

### 添加基础操作规则

```text
1. 注册 operation definition
2. 绑定 operation category
3. 声明所需权限
4. 声明执行引擎
5. 添加参数 schema
6. 添加结果 schema
7. 添加审计规则
8. 添加测试用例
9. 添加 UI 操作入口
10. 更新文档
```

### 添加工作流规则

```text
1. 创建 workflow definition
2. 声明适用实体类型
3. 绑定原子操作
4. 设置权限边界
5. 设置审批节点
6. 设置异常处理
7. 设置可视化节点
8. 添加版本号
9. 添加运行日志规则
10. 更新文档
```

### 添加新 Agent 规则

```text
1. 创建 agent profile
2. 声明能力范围
3. 绑定可用工具
4. 绑定权限策略
5. 设置上下文范围
6. 设置成本限制
7. 设置失败回退机制
8. 设置审计规则
9. 设置可视化管理入口
10. 更新文档
```

### 添加数据库 schema 规则

```text
1. 创建迁移文件
2. 声明兼容性影响
3. 声明回滚策略
4. 更新 ORM / DAO 层
5. 添加单元测试
6. 添加集成测试
7. 更新文档
8. 在模块注册表记录版本
```

---

# M2. Self-Evolution Engine

项目自进化引擎是最高层外部驱动。它由独立 Coding Agent 执行，不依赖项目内部 AI Agent API。

---

## 4.2.1 定位

Self-Evolution Engine 负责按照 Project Control Plane 中记录的规则去迭代整个项目。

它可以修改普通项目模块，但不自动修改自身核心逻辑。

---

## 4.2.2 输入

```text
反馈数据
操作日志
错误日志
工作流阻塞记录
模块注册表
项目增量规则
测试结果
部署状态
安全审计记录
```

---

## 4.2.3 输出

```text
代码修改
数据库迁移
工作流修改
权限规则修改
UI 修改
测试用例
文档更新
部署计划
回滚计划
Pull Request
变更报告
```

---

## 4.2.4 自进化流程

```text
收集反馈与日志
  ↓
聚类问题
  ↓
判断是否值得迭代
  ↓
读取项目增量规则
  ↓
生成修改方案
  ↓
修改代码 / 配置 / schema
  ↓
运行测试
  ↓
生成变更报告
  ↓
提交 PR 或补丁
  ↓
等待人工或外部策略批准
  ↓
Project Control Plane 执行部署
```

---

## 4.2.5 安全边界

```text
项目内部 Agent 不可直接修改项目代码
项目内部 Agent 不可修改 Self-Evolution Engine
Self-Evolution Engine 不依赖项目内部 Agent API
Self-Evolution Engine 修改项目模块需要经过测试和部署流程
Self-Evolution Engine 自身核心逻辑不进入自动迭代闭环
```

Self-Evolution Engine 可以生成自身升级建议，但自身升级应由外部人工或更高层维护机制确认。

---

# 5. 核心运行链路

## 5.1 普通任务链路

```text
用户 / Agent 提出任务
  ↓
Workspace Layer 创建任务
  ↓
Object & Permission Layer 检查权限
  ↓
Atomic Operation Layer 执行基础操作
  ↓
Database Layer 记录状态与事件
  ↓
Workspace Layer 更新可视化界面
```

---

## 5.2 工作流链路

```text
实体工作空间生成工作流
  ↓
绑定实体、任务、日程与操作
  ↓
权限模块检查每一步操作边界
  ↓
原子操作层执行具体动作
  ↓
工作流模块记录运行状态
  ↓
异常进入反馈层
```

---

## 5.3 反馈到自进化链路

```text
实体提交反馈
  ↓
Feedback Layer 分类
  ↓
形成 evolution candidate
  ↓
Project Control Plane 记录候选
  ↓
Self-Evolution Engine 读取候选
  ↓
生成修改方案
  ↓
测试、提交、部署
```

---

# 6. 核心设计原则

## 6.1 Entity First

所有可管理对象都抽象为 Entity。

```text
人是 Entity
AI Agent 是 Entity
客户是 Entity
项目是 Entity
合同是 Entity
订单是 Entity
服务器是 Entity
工作流也是 Entity
```

统一建模后，系统可以用同一种对象关系、权限体系、工作空间机制和反馈机制管理企业现实。

---

## 6.2 Permission First

所有操作必须经过权限检查。

```text
工作空间调用操作 → 检查权限
Agent 执行工具 → 检查权限
用户修改工作流 → 检查权限
反馈提交进化候选 → 检查权限
安装管理层变更模块 → 检查权限
```

---

## 6.3 Atomic Operations

复杂任务由工作流编排，基础操作必须原子化。

```text
基础操作负责最小动作
工作流负责编排动作
权限层负责限制动作
审计层负责记录动作
工作空间负责呈现动作
```

---

## 6.4 Human-Visible Workspace

真人实体需要低负担可视化界面。

真人只需要看到：

```text
我要做什么
什么时候做
为什么重要
谁在等我
我需要审批什么
哪里出问题了
下一步建议是什么
```

---

## 6.5 Agent as Managed Labor

AI Agent 需要被管理。

AI Agent 应当拥有：

```text
实体档案
能力边界
工具权限
任务队列
成本记录
错误记录
上下文范围
周复盘
健康状态
```

---

## 6.6 Feedback-Driven Evolution

自进化的输入来自结构化反馈、日志、阻塞记录、错误记录和使用数据。

反馈不能直接改变系统。  
反馈进入候选池后，再由自进化流程处理。

---

## 6.7 External Evolution Engine

自进化引擎应保持外部独立性。

项目内部 Agent 可以提出建议、提交反馈和辅助运行，但不能绕过项目控制平面直接修改项目本体。

---

# 7. 最小 MVP

第一版不需要完整覆盖企业所有职能。建议先实现最小闭环。

## 7.1 MVP 对象

```text
真人实体
AI Agent 实体
项目实体
任务实体
工作流实体
反馈实体
```

## 7.2 MVP 模块

```text
Database Layer
Entity Management Module
Permission Management Module
Operation Registry
Task Module
Workflow Module
Workspace UI
Feedback Module
Project Control Plane Lite
```

## 7.3 MVP 功能

```text
1. 创建实体对象
2. 创建权限规则
3. 注册基础操作
4. 创建任务
5. 创建工作流
6. 生成周计划
7. 可视化任务看板
8. 自然语言更新状态
9. 提交反馈
10. 生成自进化候选项
```

---

# 8. 首批核心 Schema

最先需要定稿的五个 schema：

```text
Entity
Operation
Permission
Workflow
Feedback
```

这五个 schema 稳定后，其他模块可以围绕它们持续生长。

---

# 9. 架构一句话定义

GESA 是一套以数据库为企业状态源、以对象权限层管理所有实体、以原子操作层提供可审计动作、以工作空间层承载秘书 Agent 和可视化协作、以反馈层形成进化输入、再由项目控制平面和外部自进化引擎驱动长期迭代的企业自进化服务架构。
