# CLI 原型验证记录

日期：2026-10-23。验证环境：Linux x86_64、Rust 1.99.1、Python 3.12、Redis 7.4.2。测试使用临时合成账号、发布密钥和隔离状态目录。

## 已在开发环境执行

| 验证 | 结果 | 范围 |
|---|---|---|
| Rust 单元测试 | 29 通过 | 配置、凭据、会话、迭代门槛、签名、时效、防降级、锁、权限、健康检查、恢复、离线回执 |
| Python 中心测试 | 12 通过 | 三项凭据、盐哈希、会话、限流、制品签名、大小、路径、并发发布 |
| 真实 Redis 切流测试 | 9 通过 | 并发请求、租约、路由 CAS、排空、健康失败回滚、实例退休 |
| CLI 端到端场景组 | 10 通过 | 真实编译版本 0.1.0 → 0.1.1 → 0.1.2，恢复和攻击夹具 |
| Rustfmt / Clippy | 通过 | `cargo fmt --check`；`cargo clippy --locked --all-targets -- -D warnings` |

切流测试中，80 个 HTTP 请求完整演练为 0 失败；另一个 48 请求并发测试观测到了新旧版本，全部成功。该结果适用于本机实验负载，生产可用性需要独立负载与故障域验证。

端到端测试机器可读输出保存在 [e2e-report.json](e2e-report.json)。

## 端到端场景

1. 外部私有配置、无凭据参数、独立启动器安装与重复安装。
2. 发布签名验证和真实核心更新，新调用进入 0.1.1。
3. 中心撤销会话后，客户端重新登录一次。
4. 更新到 0.1.2；原有 0.1.0 watch 进程继续完成下一次轮询；重复更新保持幂等。
5. 手动回滚到 0.1.1，下次 watch 保持回滚版本，序列高水位保持。
6. 签名篡改、旧发布重放、过期、平台错误、跨来源制品、契约跳代和同序列变更被拒绝。
7. 候选健康检查失败后保留旧核心，并隔离失败制品。
8. 注入持久化 `activated` 日志边界后，通过启动器离线恢复；这是日志状态重放测试，电源故障与文件系统故障仍需专门测试。
9. 当前文件损坏后通过独立 bootstrap 恢复，高水位保留。
10. 错误的已存密码导致登录失败，错误输出保持无密码和口令。

## 验证边界

- Windows 专项检查由 `.github/workflows/cli.yml` 的 Windows runner 执行，包括中文路径、DPAPI 和 ACL。开发环境本身为 Linux；CI 实际结果以 GitHub Actions 为准。
- `tools/e2e.py` 的完整进程测试在 Linux 运行。Windows 还有服务管理器终止行为、系统重启、杀进程阶段和不同文件系统的端到端测试需要扩充。
- 证据模板是合成测试夹具。签名准入证明发布方承诺的证据与包绑定，CI 报告来源的独立认证需要后续集成。
- Redis 实验未连接 PostgreSQL；本次没有业务数据迁移或数据库服务自更新。
- 完整 TUF、多密钥轮换、跨主机 HA、生产滚动发布仍属于后续开发。

## 复现

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 -m pip install -r tools/requirements.txt
python3 -m unittest discover -s tools -p 'test_*.py' -v
python3 tools/e2e.py --report /absolute/path/e2e-report.json
```

为复现 Redis 测试，确保 `redis-server` 在 PATH，或设置 `REDIS_SERVER`。测试会绑定随机回环端口并自动清理进程。
