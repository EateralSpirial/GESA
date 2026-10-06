# GESA 可部署基座

> Version: 0.1  
> Scope: 从概念架构进入 Rust 实体构建所需的运行时、契约、模块、节点、更新与安全设计。

---

## 当前文档

| 编号 | 文档 | 定位 |
|---|---|---|
| 01 | [可部署基座系统设计](./01-deployable-base-system-design.md) | 定义信息流内核、统一对象模型、组织与权限、中央控制平面、企业运行节点、Windows/Linux 终端、企业剖面、更新、隐私 API 和首条端到端垂直切片。 |

---

## 计划拆分

```text
02-information-flow-contract.md
03-core-object-model.md
04-organization-and-permission-engine.md
05-module-sdk-and-enterprise-profile.md
06-central-control-and-update-plane.md
07-database-schema-and-migrations.md
08-api-and-protocol-contracts.md
09-security-threat-model.md
10-deployment-and-operations.md
```

每份后续文档只处理一个实现边界，并通过 ADR 记录关键技术选择。
