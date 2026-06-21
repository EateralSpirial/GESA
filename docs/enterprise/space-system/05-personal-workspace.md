# 05. 个人工作区

> Version: 0.1  
> Parent: [GESA 企业空间系统](./README.md)

---

## 1. 定位

个人工作区是每个实体对象的私有操作入口。

```text
PersonalWorkspace = 用户自己的工作空间、文件空间、Agent Team 控制台和任务入口。
```

---

## 2. 基础规则

```text
1. 个人工作区属于个人隐私空间。
2. 他人默认不可查看。
3. 用户所处层级不同，可配置不同空间容量。
4. 个人工作区可拥有代码执行权限和联网权限，但必须通过 Agent Team 或执行器对象控制。
5. 个人工作区不可拥有长期个人向量知识库。
6. 个人工作区不可拥有个人数据库。
```

---

## 3. 个人空间拥有的对象

```text
PersonalWorkspace
├── ChatObject
├── AgentTeamObject
├── WorkObject
├── ScheduleObject
├── ScriptObject
├── FileSystemObject
├── PermissionManager
└── WorkspaceManager
```

---

## 4. 聊天器对象

聊天器对象允许用户按照权限与组织内部对象沟通。

可沟通对象包括：

```text
1. 自己的 Agent 对象
2. 同级对象
3. 直属上司对象
4. 下属对象
5. 被授权沟通的部门对象
6. 被授权沟通的空间对象
```

---

## 5. Agent Team 对象

个人工作区内的 Agent Team 默认对用户可见。

它可以包含：

```text
1. AI 总管
2. AI 秘书
3. AI 执行者
4. AI 倾听者
5. AI 创造者
6. 自定义 Agent
```

Agent Team 通过委托权限会话代用户执行任务。

---

## 6. 工作对象

工作对象包括：

```text
1. 待办任务
2. 日程安排
3. 功能入口
4. 脚本绑定
5. 执行记录
```

工作对象是个人日常操作的基础单位。

---

## 7. 文件系统对象

个人文件系统保存：

```text
1. 个人设置
2. 工作区布局
3. Agent 设置
4. 日程安排
5. 私有脚本
6. 任务日志
7. 聊天线程索引
```

实现代码部分仍然归属组织资源，普通用户默认不可查看。

---

## 8. 推荐文件结构

```text
/personal/{user_id}/
├── profile/
│   ├── settings.json
│   ├── preferences.json
│   └── workspace-layout.json
├── agents/
│   ├── team.json
│   ├── memory/
│   └── instructions/
├── schedules/
│   ├── weekly.json
│   └── templates/
├── scripts/
│   ├── private/
│   └── bindings/
├── chats/
│   ├── threads/
│   └── pinned/
├── tasks/
│   ├── active/
│   ├── completed/
│   └── logs/
└── files/
```

---

## 9. 权限管理系统

个人工作区内的权限管理系统只展示与用户相关的权限摘要。

它不直接替代企业权限引擎。

```text
个人权限视图 = 用户可见权限的展示层。
企业权限引擎 = 权限判断的确定性执行层。
```

---

## 10. 工作区管理系统

工作区管理系统负责：

```text
1. 控制展示内容。
2. 保存个人信息与设置。
3. 管理个人脚本入口。
4. 管理 Agent Team 展示方式。
5. 管理日程视图和默认时间范围。
```