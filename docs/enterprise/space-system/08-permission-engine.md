# 08. 权限引擎

> Version: 0.1  
> Parent: [GESA 企业空间系统](./README.md)

---

## 1. 定位

权限引擎负责判断对象是否可以访问、修改、执行或管理某个资源。

企业侧看到的是授权关系和组织图。系统内部执行的是确定性权限判断。

```text
企业侧：谁能看、谁能写、谁能管理、谁能授权。
开发侧：Subject、Action、Resource、Policy、Grant、Audit。
```

---

## 2. 权限判断三元组

```text
PermissionDecision(subject, action, resource)
```

其中：

```text
subject  = 人类实体、Agent 实体、部门、角色、Agent Team
 action   = read、write、execute、manage、delegate、admin、audit
resource = 空间、文件、知识库、数据库、执行器、脚本、名单、授权器
```

---

## 3. 基础动作

```text
read      读取
write     写入
execute   执行
manage    管理
delegate  再授权
admin     完全管理
audit     查看审计
```

数据库、知识库、文件系统和执行器可以继续细分动作。

示例：

```text
db.read
db.write
db.query
kb.search
kb.read
kb.write
fs.read
fs.write
executor.run
executor.cancel
```

---

## 4. 权限来源

权限可以来自：

```text
1. CEO 权限
2. 最高权限管理器
3. 部门首脑权限
4. 部门向下投影覆盖
5. 直接授权
6. 角色授权
7. 委托权限会话
8. 全局策略
```

---

## 5. 授权规则

```text
1. 授权者只能授权自己已经拥有的权限。
2. 授权者不能授予比自己更高的权限。
3. delegate 权限必须显式授予。
4. CEO 任命权不可下放给 Agent。
5. Agent Team 代用户执行时必须绑定委托权限会话。
6. 部门授权器只管理本部门权限边界内的授权。
```

---

## 6. 权限判断流程

```text
操作请求
    ↓
识别 Subject
    ↓
识别 Action
    ↓
识别 Resource
    ↓
加载直接授权
    ↓
加载部门权限
    ↓
加载空间覆盖权限
    ↓
加载委托会话
    ↓
检查全局策略
    ↓
检查显式拒绝规则
    ↓
生成权限决策
    ↓
记录审计日志
```

---

## 7. 组合公式

```text
PermissionDecision =
    DirectGrant
  + DepartmentRole
  + SpaceCoverage
  + DelegatedSession
  + GlobalPolicy
  - ExplicitDeny
  - RiskRestriction
```

显式拒绝规则优先级应高于普通允许规则。

---

## 8. Agent 权限原则

```text
1. Agent 可以作为实体对象参与组织。
2. Agent 可以成为部门首脑。
3. Agent 不可成为 CEO。
4. Agent 不可拥有 CEO 任命权。
5. Agent 所有代用户操作必须审计。
6. Agent 不可绕过权限引擎直接访问资源。
```

---

## 9. 审计要求

审计记录至少包含：

```text
1. 操作主体
2. 委托来源
3. 操作动作
4. 操作资源
5. 权限决策
6. 决策原因
7. 风险等级
8. 时间戳
```

---

## 10. 关键边界

知识库检索可以帮助查找权限规则说明、权限异常、相似问题和历史决策，但不能替代权限判断。

```text
权限判断必须由确定性权限引擎完成。
语义检索只提供上下文，不决定访问权。
```