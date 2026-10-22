# GESA CLI 原型协议 v1

日期：2026-10-23。范围：Rust CLI 的受信发布、迭代门槛与可恢复激活。企业运行数据仍由企业节点管理；本原型仅保存控制工具自己的配置与更新状态。

## 1. 进程和信任边界

`gesa-launcher` 安装后命名为 `gesa`，根据本地状态选择已经校验的核心程序。核心包是另一个 Rust 可执行文件，编译目标为 `gesa`。下载内容按 SHA-256 分目录保存，更新过程中保持旧文件可用。

Linux 启动器通过 exec 交接给核心，因此 PID 和信号语义保留。Windows 启动器等待核心退出并传播退出码。当前执行任务保持其原二进制；后续调用使用新版本。长期运行的 `watch` 进程保持启动时的核心版本，升级后可由操作员重启，以使用新版轮询逻辑。

bootstrap 来自操作员最初安装的可信发行包。中心无法自动替换独立启动器或 bootstrap；这个独立恢复边界允许核心自更新失败时恢复命令入口。

## 2. 外部状态

| 内容 | 位置／约束 |
|---|---|
| 中心地址、账号、通道、公钥 | 私有目录 `config.json`，严格字段校验 |
| 密码、静态口令、短期 token | Linux 系统密钥环／Windows DPAPI；Unix 开发模式为私有文件 |
| 当前、上一、bootstrap 版本 | `update-state.json`，原子替换 |
| 候选执行文件 | `releases/<sha256>/gesa-core[.exe]` |
| 原始签名回执 | 同目录 `release-<sequence>.json`，保留中心原始 envelope |
| 更新互斥 | `update.lock`，OS 文件锁，进程退出后释放 |
| 高水位、隔离列表、激活日志 | 更新状态中的独立字段 |

凭据记录绑定规范化中心来源、账号、通道、公钥和本地目录。编辑中心或账号后，需要重新输入凭据。秘密输入使用隐藏终端；日志和 JSON 结果保持无密码、口令、token。

默认 HTTPS；仅在显式开发配置中允许数字回环 IP 的 HTTP。HTTP 重定向关闭，制品限制同一来源，文件限 64 MiB，网络、系统密钥环辅助程序和候选健康进程均有超时。

## 3. 中心 HTTP 接口

| 方法 | 路径 | 输入／输出 |
|---|---|---|
| POST | `/v1/login` | 账号、密码、口令 → 短期 access_token、expires_at |
| GET | `/v1/releases/latest?channel=stable&target=linux-x86_64` | Bearer token → 签名 envelope |
| GET | `/artifacts/<sha256>` | Bearer token → 精确二进制字节 |

服务端密码和口令使用独立随机盐与 scrypt 哈希；每项必须通过。开发中心限制登录请求，错误保持统一，token 在内存保存且重启失效。客户端遇到 401 最多重新认证一次。

签名 envelope：

```json
{"payload":"BASE64_EXACT_JSON_BYTES","signature":"ED25519_SIGNATURE_HEX"}
```

签名覆盖解码后的完整原始 JSON 字节，客户端在解析发布声明前验签。发布声明包含：

```text
schema, sequence, version, channel, target, artifact,
sha256, size, expires_at, protocol,
contract_epoch, supports_contracts, iteration
```

`target` 使用 `linux-x86_64` 或 `windows-x86_64` 等 OS／CPU 名称，v1 尚未区分 libc 与其他 ABI。`version` 是 SemVer；`sequence` 是递增整数。相同序列只有精确相同的签名载荷可重复使用。回滚保留最高已准入序列；中心继续发布时必须使用更高序列。

## 4. 迭代证据

以下示例须替换真实 Git 提交、制品 SHA-256 和测试报告 SHA-256：

```json
{
  "candidate_id": "GESA-42",
  "source_revision": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "artifact_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "change_summary": "修复重试边界并补充回归测试",
  "risk": "low",
  "tests": [
    {"name":"regression", "passed":true,
     "evidence_sha256":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"}
  ],
  "approval": {"decision":"approved", "actor":"release-policy-v1", "policy":"automatic-low-risk"},
  "rollback":"保留上一可用核心；本次无数据迁移",
  "migration": {"kind":"none", "reversible":true},
  "protected_core_changed": false
}
```

验证：

```bash
gesa protocol /absolute/path/iteration.json --sequence 1 --epoch 1 --supports 1
```

规则：

1. 每项测试通过且带报告摘要，测试名称唯一；迭代中的制品摘要与发布包逐字节绑定。
2. 自动策略仅允许 low、无迁移、无受保护核心变更。medium／high、受保护核心变更与数据迁移需要签名证据中的 manual 批准。
3. 当前候选的契约代数最多前进一步，须继续支持当前与此前两代契约。
4. N+3 以后撤销旧契约，需要额外 `retirement` 声明与 manual 批准：

```json
{"through_epoch":1,"migration_complete":true,"rollback_verified":true,
 "evidence_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}
```

把它放到迭代对象的 `retirement` 字段。v1 只验证声明；实际迁移、兼容性测试和回滚演练由发布流水线执行，结果由签名方负责。本原型不执行 SQL 迁移。`protected_core_changed` 等分类同样属于发布方的受信声明，客户端无法仅凭二进制自行判定。

发布：

```bash
python3 tools/demo_center.py publish --root /absolute/private/gesa-center \
  --binary /absolute/path/gesa --sequence 1 --version 0.1.1 \
  --target linux-x86_64 --iteration /absolute/path/iteration.json
```

版本须与程序 `__health` 返回一致。原型测试构建可以设置 `GESA_RELEASE_VERSION` 后编译；正式发布应由受控构建流水线提供版本和完整证据。

## 5. 激活事务

1. 获取更新互斥锁，恢复上次未完成的激活。
2. 登录并获取中心元数据，验证签名、来源、平台、版本、时效、序列和迭代门槛。
3. 持久保存序列高水位，下载到临时文件，校验大小和 SHA-256，再移入不可变版本目录。
4. 以清理后的环境执行候选 `__health`；检查 5 秒超时、退出码、JSON、协议、版本和平台。
5. 保存 `prepared` 日志，再原子保存 `activated` 状态。
6. 再次健康检查，通过后提交并保留上一版本；失败恢复旧版本并隔离失败制品。

准备／激活阶段中新调用继续使用已经提交的旧核心。中断后，启动器恢复日志中的来源版本；损坏时逐级尝试上一版本和 bootstrap。恢复可以脱离中心完成。

手动 `rollback` 同样隔离被撤回制品，后续 watch 保持恢复版本。要恢复被隔离的功能，发布修正后的新制品；仅提高序列、重复同一二进制无法绕过隔离。

本地哈希校验防止意外损坏和低权限篡改。已控制当前 OS 用户的攻击者可以修改工具、状态与配置，该情况超出本机存储的信任边界。

`gesa verify` 还会使用配置中的固定公钥离线复核当前和上一发布的原始签名回执、迭代证据与制品绑定。已经准入的历史回执允许其有效期后来到期；新安装仍检查过期与序列。bootstrap 的信任来自首次人工安装的发行包。

## 6. 服务平滑切流实验

`tools/rollout_demo.py` 使用真实 Redis 保存路由和在途计数。Lua 原子脚本绑定协调者租约、当前路由和单调代数，拒绝陈旧切换。HTTP 网关读取路由并增加在途计数属于同一原子步骤，避免“切流和请求计数”之间的竞态。

候选就绪后切流，再次健康失败时回滚；旧请求排空后停止旧实例。达到排空时限时保留旧实例，退休栅栏阻止再次激活已停止实例。

这个实验用于检验未来 Rust 模块管理器的协议。跨主机 Redis HA、ACL 身份分离、失联后的请求计数回收、PostgreSQL 持久业务事实、数据库服务升级仍属于后续集成工作。

## 7. 参考

- [TUF 规范](https://theupdateframework.github.io/specification/v1.0.36/)：序列、防回滚、过期和多角色信任设计；本原型实现其中有限的安全属性。
- [Rust fs::rename](https://doc.rust-lang.org/std/fs/fn.rename.html)：同文件系统文件替换的跨平台语义。
