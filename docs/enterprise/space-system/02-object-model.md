# 02. 企业空间系统对象模型

> Version: 0.1  
> Parent: [GESA 企业空间系统](./README.md)

---

## 1. 总对象模型

企业空间系统中的所有可管理元素都应抽象为对象。

```text
GESAObject
├── EntityObject
├── SpaceObject
├── ResourceObject
├── PermissionObject
├── ChatObject
├── ScheduleObject
└── TaskObject
```

---

## 2. EntityObject：实体对象

实体对象包括人类和 Agent。

```text
EntityObject
├── HumanEntity
└── AgentEntity
```

规则：

```text
1. 人类和 Agent 都是对象。
2. Agent 实体对象拥有模板，可以被克隆。
3. CEO 必须是 HumanEntity。
4. Agent 不能成为 CEO。
5. Agent 可以成为部门首脑。
```

---

## 3. SpaceObject：空间对象

```text
SpaceObject
├── PersonalWorkspace
├── CellSpace
├── DepartmentSpace
├── DirectionSpace
├── LevelSpace
└── GlobalSpace
```

空间对象是资源的容器，也是权限判断的重要资源边界。

---

## 4. ResourceObject：资源对象

```text
ResourceObject
├── FileSystemObject
├── CodeExecutorObject
├── AgentTeamObject
├── DatabaseObject
├── KnowledgeBaseObject
└── ScriptLibraryObject
```

资源对象必须挂载到某个空间。

---

## 5. PermissionObject：权限对象

```text
PermissionObject
├── Role
├── Grant
├── Policy
├── DelegatedSession
├── DepartmentAuthorizer
└── AuditLog
```

权限对象负责记录谁对什么资源拥有何种操作能力。

---

## 6. ChatObject：聊天对象

```text
ChatObject
├── ChatThread
├── DefaultPinnedThread
├── CustomThread
└── Message
```

对象聊天中，人类、Agent、部门、空间都可以成为聊天目标。

---

## 7. ScheduleObject：日程对象

```text
ScheduleObject
├── ScheduleItem
├── ScheduleTemplate
├── ScriptBinding
└── ExecutionPlan
```

一个日程对象可以绑定多个脚本，并以用户权限执行。

---

## 8. TaskObject：任务对象

```text
TaskObject
├── ComputerUseTask
├── BrowserUseTask
├── CodeExecutionTask
├── InternalOperationTask
└── CompositeTask
```

任务对象进入执行器后，应产生状态记录、执行记录和审计记录。

---

## 9. 对象设计原则

```text
1. 所有可授权、可执行、可审计的元素都对象化。
2. 对象可以归属空间，也可以被空间引用。
3. Agent 对象与人类对象共享部分能力，但必须有额外审计。
4. 权限判断必须基于确定性对象关系。
5. 企业侧对象必须能映射到开发侧 Schema。
```