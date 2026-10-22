# GESA

GESA（General Enterprise Service Architecture）探索以信息流规则构建企业服务系统。架构文档见 [docs/README.md](docs/README.md)。

本仓库现在包含可运行的 **Rust CLI 更新协议原型**：外部配置、账号／密码／口令登录、签名发布物、自动拉取、版本切换、失败恢复和离线校验。

## 快速测试整个闭环

需要 Rust stable、Python 3.12+；Redis 平滑切流实验需要 `redis-server`。以下完整进程测试在 Linux 运行，Windows 的 Rust 构建、权限和 DPAPI 测试由 CI 执行。

```bash
cargo build --locked
python3 -m pip install -r tools/requirements.txt
cargo test --locked
python3 -m unittest discover -s tools -p 'test_*.py' -v
python3 tools/e2e.py
```

`e2e.py` 自动建立临时中心和临时账号，编译 0.1.1／0.1.2 两个真实 CLI 版本，测试升级、回滚、异常签名、版本重放、契约门槛、故障隔离、恢复与旧进程存活。测试完成后清理临时凭据；测试证据均为合成夹具。请在专用 checkout 中运行，它会依次重新构建 debug 二进制。

Redis 测试会启动隔离的本机实例；找不到 `redis-server` 时明确显示跳过。可用环境变量 `REDIS_SERVER` 指定其位置。

## 安装并连接你的中心

```bash
cargo build --locked --release
./target/release/gesa init
./target/release/gesa install
```

Windows 使用 `target\release\gesa.exe`。发行目录中须同时保留 `gesa` 和 `gesa-launcher` 两个可执行文件。

`init` 交互询问中心 HTTPS 网址、账号、可信发布公钥，以及隐藏输入的密码和口令。公钥应由可信渠道提供。配置位置：

| 平台 | 默认目录 | 密码、口令、会话 |
|---|---|---|
| Linux | `$XDG_CONFIG_HOME/gesa` 或 `~/.config/gesa` | Secret Service 系统密钥环，需 `secret-tool` 和已解锁会话 |
| Windows | `%LOCALAPPDATA%\GESA` | 当前用户 DPAPI 加密文件，配合目录 ACL 校验 |

`GESA_HOME` 可指定绝对目录，命令无需重复传入网址或凭据。中心和账号保存在该目录的 `config.json`；密码和口令没有 CLI 参数。Unix 无桌面测试环境可在初始化时显式选择 `file`，此模式以 0600 文件保存测试秘密，目录要求 0700。正式凭据使用系统密钥存储。

`install` 将独立启动器放到配置目录的 `bin/gesa`（Windows 为 `bin\gesa.exe`）。将该 `bin` 目录加入 PATH，之后统一使用：

```bash
gesa login
gesa status
gesa check
gesa update
gesa verify
gesa watch
gesa rollback
gesa recover
```

常用命令输出统一 JSON，`schema: 1`、`ok`、`command` 和 `result` 可供脚本检查。失败返回非零退出码；参数帮助遵循常规 CLI 文本格式。`gesa --version` 显示当前实际运行版本。

修改中心、账号、公钥后，使用 `gesa credentials` 重新隐藏输入凭据；`gesa logout` 删除本机会话。中心会话过期或返回 401 时会进行一次重新认证。`watch` 是前台轮询，默认间隔 60 秒，可用 `--once` 运行一轮。

账号用于身份标识，密码与静态口令属于同类知识凭据。因此此原型实现的是三项校验；独立多因素认证还需要设备密钥或动态验证码等额外机制。

## 本机中心实验

中心服务为开发夹具，绑定 `127.0.0.1`。在仓库以外选择私有目录：

```bash
python3 tools/demo_center.py init --root /absolute/private/gesa-center
python3 tools/demo_center.py serve --root /absolute/private/gesa-center
```

`init` 隐藏询问密码和口令，输出发布公钥。客户端 `gesa init` 填入 `http://127.0.0.1:8787`，显式启用回环 HTTP，使用刚建立的账号和公钥。远程中心强制 HTTPS，禁止重定向和跨来源下载。

发布一个已测试的二进制需要独立迭代证据文件，格式与发布命令见 [协议文档](docs/developer/cli-prototype/protocol.md)。最快的完整演练入口是上述 `e2e.py`。

Redis 平滑切流演练使用外部 JSON 配置，例如：

```json
{"redis_host":"127.0.0.1","redis_port":6379,"namespace":"gesa-demo","requests":100}
```

```bash
python3 tools/rollout_demo.py --config /absolute/private/rollout.json
```

该实验运行稳定 HTTP 网关和新旧实例，检查新实例就绪，通过 Redis 原子切换路由，保留旧实例直到请求排空。Redis 密码也从配置文件读取。

## 原型边界

- CLI 自更新采用独立启动器与不可变版本目录：新调用使用新版本，已运行的进程完成原任务。启动器自身由发行包安装更新，并保留可信 bootstrap 恢复入口。
- 服务切流是独立的真实 Redis／HTTP 实验。生产模块调度、PostgreSQL 数据迁移、数据库引擎自更新和跨主机高可用仍需后续实现。
- 自动迭代覆盖“证据 → 确定性门槛 → 签名发布 → 安装 → 校验／回滚”。证据由发布方声明并签名；代码生成和 CI 证据来源验证属于外部 M2／发布流水线。
- 发布安全使用单个固定 Ed25519 公钥、摘要、序列高水位和过期检查。完整 TUF 根密钥轮换、委托、阈值签名及第三方审计仍需补充。
- CLI 健康检查验证协议、版本、平台和进程退出状态。企业业务正确性需要各模块的专门健康检查。
- 开发中心的签名私钥保存在测试目录中；生产中心应分离身份服务、受限签名服务及 HTTPS 网关。

实现细节见 [原型协议](docs/developer/cli-prototype/protocol.md) 和 [测试记录](docs/developer/cli-prototype/validation.md)。
