# 06. Agent Team 与执行

> Version: 0.1  
> Parent: [GESA 企业空间系统](./README.md)

---

## 1. 定义

Agent Team 是由 Agent 秘书长统一管理的一组 subagents。

```text
AgentTeam
├── SecretaryGeneralAgent
├── SecretaryAgent
├── ExecutorAgent
├── FeedbackAgent
├── CreatorAgent
├── AnalystAgent
└── CustomAgents
```

企业侧名称：

```text
AI 总管
AI 秘书
AI 执行者
AI 反馈收集者
AI 创造者
AI 分析者
自定义 Agent
```

---

## 2. AI 总管

```text
1. 接收用户指令。
2. 判断任务类型。
3. 分配给合适的子 Agent。
4. 调用日程、脚本、执行器、知识库和文件系统。
5. 汇总结果并向用户反馈。
```

---

## 3. AI 秘书

```text
1. 安排日程。
2. 维护七日计划。
3. 检测时间冲突。
4. 生成提醒。
5. 协调对象之间的沟通。
```

---

## 4. AI 执行者

AI 执行者负责执行用户授权的任务。

```text
1. 浏览器任务
2. 代码任务
3. 企业内部脚本任务
4. 文件生成任务
5. 综合任务
```

---

## 5. AI 反馈收集者

```text
1. 接收建议。
2. 归类反馈。
3. 生成改进候选。
4. 创建反馈任务。
5. 将有效反馈交给反馈层或 Issue Engine。
```

---

## 6. AI 创造者

```text
1. 发现重复任务。
2. 生成脚本草案。
3. 生成日程模板。
4. 建议新的功能入口。
5. 把高价值功能候选交给开发侧自进化流程。
```

---

## 7. 权限继承

Agent Team 通过委托权限会话代用户执行任务。

```text
User Permission
    ↓
Delegated Permission Session
    ↓
Agent Team
    ↓
Subagent
    ↓
Action
    ↓
Audit Log
```

规则：

```text
1. Agent Team 不应永久持有用户全部权限。
2. 每次代用户执行都应绑定委托会话。
3. 重要操作需要额外确认或审批。
4. 所有执行行为必须记录审计日志。
```

---

## 8. 执行原则

```text
1. Agent 执行必须绑定用户或部门授权上下文。
2. Agent 执行必须产生任务记录。
3. Agent 执行必须产生审计记录。
4. Agent 不可绕过企业权限引擎直接访问资源。
5. Agent 创建的新功能应进入脚本库或开发侧自进化流程。
```