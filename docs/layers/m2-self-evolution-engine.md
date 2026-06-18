# M2 Self-Evolution Engine 项目自进化引擎

> Version: 0.1  
> Status: Layer Design  
> Position: Meta Plane / External Evolution Driver

---

## 1. 定位

项目自进化引擎是 GESA 的最高层外部驱动，由独立 Coding Agent 或外部工程自动化系统执行。它不依赖项目内部 AI Agent API，不由项目内部业务 Agent 直接控制。

它回答的问题是：

```text
哪些反馈值得改项目？
应该修改哪些模块？
应该怎样修改代码、schema、工作流、权限和文档？
修改后如何测试、提交、说明和交付给项目控制平面？
如何避免自进化引擎被内部业务上下文污染？
```

---

## 2. 核心职责

```text
1. 读取反馈、日志、Issue、知识库、测试报告和模块注册表。
2. 聚类问题并判断是否值得项目级迭代。
3. 读取 M1 Project Control Plane 的增量规则。
4. 生成修改方案。
5. 修改代码、配置、schema、文档或测试。
6. 运行测试和安全扫描。
7. 生成变更报告、部署建议和回滚说明。
8. 提交 PR 或补丁。
9. 将结果交给 M1 执行部署。
10. 记录自进化过程用于审计和复盘。
```

---

## 3. 安全边界

```text
项目内部 Agent 不可直接修改项目代码。
项目内部 Agent 不可修改 Self-Evolution Engine。
Self-Evolution Engine 不依赖项目内部 Agent API。
Self-Evolution Engine 修改项目模块需要经过测试和部署流程。
Self-Evolution Engine 自身核心逻辑不进入自动迭代闭环。
```

M2 可以生成自身升级建议，但自身升级应由外部人工或更高层维护机制确认。

---

## 4. 输入

```text
feedbacks
evolution_candidates
operation_logs
audit_logs
workflow_blockers
agent_failure_logs
knowledge_update_candidates
module_registry
increment_rules
test_results
deployment_records
security_scan_results
architecture_documents
```

---

## 5. 输出

```text
code_patch
database_migration
knowledge_update
workflow_update
permission_policy_update
operation_definition_update
ui_update
test_cases
security_fix
documentation_update
deployment_plan
rollback_plan
pull_request
change_report
```

---

## 6. 自进化流程

```text
收集反馈与日志
  ↓
聚类问题
  ↓
判断是否值得迭代
  ↓
读取知识库和架构文档
  ↓
读取项目增量规则
  ↓
定位受影响模块
  ↓
生成修改方案
  ↓
修改代码 / 配置 / schema / 文档
  ↓
运行测试和静态分析
  ↓
生成变更报告
  ↓
提交 PR 或补丁
  ↓
等待人工或 M1 策略批准
  ↓
M1 执行部署
```

---

## 7. 候选类型

| 类型 | 输入来源 | 输出方向 |
|---|---|---|
| Bug Fix | 反馈、错误日志、Issue | 代码修复、测试 |
| Workflow Improvement | 工作流阻塞、复盘 | 工作流定义修改 |
| Permission Fix | 权限冲突、安全反馈 | 权限策略修改 |
| Operation Fix | 工具失败、操作日志 | Operation 定义或引擎修复 |
| Knowledge Update | FAQ、Agent 失败、人工复盘 | 文档和知识库更新 |
| Schema Evolution | 数据缺失、流程需求 | 数据库迁移和 API 更新 |
| UI Improvement | UI 反馈、使用分析 | 工作空间界面修改 |
| Deployment Fix | 部署失败、健康检查 | 安装脚本和部署配置修改 |
| Test Improvement | 回归缺失、Agent 误判 | 测试和评测用例 |

---

## 8. 工程链路

```text
Issue / Candidate
  ↓
Context Retrieval
  ↓
Repo Index Search
  ↓
Patch Planning
  ↓
Code Editing
  ↓
Local Test
  ↓
Static Analysis
  ↓
LLM Eval / Agent Eval
  ↓
PR Creation
  ↓
Review
  ↓
M1 Deployment
```

---

## 9. 测试要求

每次自进化 PR 至少说明：

```text
修改了什么
为什么修改
影响哪些模块
如何测试
是否需要迁移
是否需要回滚计划
是否修改权限边界
是否修改知识库
是否修改工作流
```

测试类型：

```text
unit tests
integration tests
migration tests
workflow tests
permission tests
agent behavior evals
security scans
smoke tests
```

---

## 10. 核心 Schema

```text
evolution_runs
- id
- candidate_id
- runner_agent
- input_summary
- affected_modules
- status
- started_at
- finished_at

patch_plans
- id
- evolution_run_id
- plan_text
- risk_level
- required_tests
- rollback_strategy

change_reports
- id
- evolution_run_id
- changed_files
- changed_schemas
- changed_operations
- changed_workflows
- changed_permissions
- changed_knowledge
- test_summary

self_evolution_audit_logs
- id
- run_id
- action
- input_ref
- output_ref
- created_at
```

---

## 11. 开源方案

| 功能 | 轻量级 | MVP | 企业级 |
|---|---|---|---|
| Coding Agent | Aider / Goose | OpenHands / Aider | OpenHands + SWE-agent + CI |
| 代码搜索 | ripgrep / ctags | Sourcegraph / Zoekt | Sourcegraph + CodeQL |
| Repo 自动化 | GitHub CLI / Gitea | GitHub CLI + Renovate | GitLab/GitHub + policy gates |
| 静态分析 | Ruff / ESLint / Bandit | Semgrep / Trivy | Semgrep + Trivy + SBOM |
| 测试 | pytest / Playwright | pytest + promptfoo | pytest + promptfoo + Ragas + SWE-bench |
| 观测 | logs | Langfuse / Phoenix | Langfuse + Phoenix + OpenTelemetry |
| 发布 | 手动 PR | Woodpecker / GitHub Actions | Argo CD / Flux / Tekton |

---

## 12. MVP 实现顺序

```text
1. evolution_candidates 表
2. docs/rules/ 增量规则
3. GitHub Issues 输入
4. Aider / OpenHands 外部执行
5. pytest / lint / security scan
6. PR 模板
7. change_report 模板
8. M1 接收 PR 和部署计划
9. evolution_runs 审计记录
10. 知识库回写机制
```

---

## 13. 风险

| 风险 | 表现 | 对策 |
|---|---|---|
| 自我修改失控 | M2 修改自身核心逻辑 | 自身核心逻辑不自动迭代 |
| 项目内部污染 | 业务 Agent 诱导修改代码 | M2 独立上下文，读取正式知识和候选 |
| 无测试修改 | PR 看似合理但破坏系统 | 测试、扫描、评测必须过关 |
| 直接部署 | 未审查变更进入生产 | 只提交 PR，M1 部署 |
| 架构漂移 | 越改越偏离分层 | 每次改动必须引用架构文档和增量规则 |

---

## 14. 完成定义

```text
M2 可读取 evolution_candidates。
M2 可读取知识库和架构文档。
M2 可生成修改计划和 PR。
M2 可运行测试和扫描。
M2 可生成变更报告和回滚建议。
M2 不直接部署生产。
M2 的执行过程有审计记录。
```
