# Developer Primitives

[English](README.md) | [产品规格](docs/product_spec.zh-CN.md) |
[更新日志](CHANGELOG.md) | [贡献指南](CONTRIBUTING.md)

Developer Primitives 是一个本地优先的浏览器工作台和双 CLI 工具，用于标识符
生成、标识符检查和可复现的时区转换。浏览器应用将 Rust 编译为 WebAssembly 后
在本机运行，不发送应用网络请求。`tinkora-id` 处理 UUID 和 ULID 工作流；
`tinkora-time` 处理时间戳和 IANA 时区工作流。

## 能力

- 生成 UUID v4、UUID v7 和规范的大写 ULID，支持单个生成或按顺序批量生成
  1 到 10,000 个结果。
- 严格检查 UUID 与 ULID，并在适用时返回 UUID 版本、变体和 v7/ULID 内嵌
  时间戳。
- 将显式的 Unix 秒、Unix 毫秒或 RFC 3339 瞬间转换为 UTC 和 1 到 8 个按输入
  顺序排列的 IANA 时区结果。
- 将本地民用日期时间解析为 `UNAMBIGUOUS`、`GAP` 或 `FOLD`，不会移动 gap，
  也不会擅自选择 fold 的一侧。
- Rust core、CLI 和浏览器统一使用内置的 IANA tzdb 2026c，不依赖宿主机时区
  数据库。
- Rust core、双 CLI 与 WebAssembly bridge 返回带版本的结果和稳定的机器可读
  错误码。

本项目不是标识符分配服务、托管 API、数据库、调度器、会议规划工具，也不是
可运行的 MCP server。[`skills/`](skills/) 下的 schema 仅是文档草案。

## 使用浏览器工作台

在本地运行静态工作台：

```bash
cd crates/uuid_factory_web
npm ci
npm run build:wasm
python3 -m http.server 8080 --bind 127.0.0.1 --directory static
```

在浏览器打开 `http://127.0.0.1:8080`。Identifiers 模块用于生成和检查 UUID 与
ULID；Time 模块根据 IANA 规则转换显式瞬间和解析本地时间，并提供 UTC 主摘要与
按顺序排列的对比时区。生成值、输入、结果和显式剪贴板操作均保留在浏览器中。
页面不使用遥测、Cookie、持久化存储、CDN、远程字体或时区 API。

## 使用 CLI

两个 CLI 都将成功结果写入标准输出，将诊断信息写入标准错误：

```bash
# Generate and inspect identifiers.
cargo run -p uuid_factory_cli --bin tinkora-id -- generate --kind uuid-v7 --count 3
cargo run -p uuid_factory_cli --bin tinkora-id -- generate --kind ulid --count 2 --json
printf '%s\n' '550e8400-e29b-41d4-a716-446655440000' \
  | cargo run -p uuid_factory_cli --bin tinkora-id -- inspect --json

# Convert one explicit instant into ordered zones.
cargo run -p timestamp_zone_cli --bin tinkora-time -- convert \
  --unix-seconds 0 --zone UTC --zone Asia/Shanghai --json

# Resolve a local civil time without hiding a DST fold.
cargo run -p timestamp_zone_cli --bin tinkora-time -- resolve \
  --local 2026-11-01T01:30:00 --zone America/New_York --json

# Discover bundled IANA names with an exact lookup or bounded filter.
cargo run -p timestamp_zone_cli --bin tinkora-time -- zones --name Asia/Shanghai
cargo run -p timestamp_zone_cli --bin tinkora-time -- zones --filter shanghai --json
```

`tinkora-id generate` 接受 `uuid-v4`、`uuid-v7` 或 `ulid`。
`tinkora-time convert` 必须且只能指定 `--unix-seconds`、
`--unix-milliseconds` 或 `--rfc3339` 之一，并重复传入 1 到 8 个 `--zone`。
`tinkora-time resolve` 必须传入一个 `--local` 和一个 `--zone`。所有成功 JSON
均使用 `schema_version: 1`。成功退出码为 `0`；命令行用法错误为 `2`；运行错误
为 `1`，并包含稳定错误码。

## 标识符语义

| 类型 | 规范输出 | 时间戳 | 适用场景 |
| --- | --- | --- | --- |
| UUID v4 | 小写、带连字符的 UUID | 无 | 随机标识符 |
| UUID v7 | 小写、带连字符的 UUID | Unix 毫秒 | 需要时间排序的标识符 |
| ULID | 大写 Crockford Base32 | Unix 毫秒 | 更短、需要时间排序的标识符 |

UUID v7 和 ULID 按各自内嵌的毫秒时间戳排序；同一毫秒内生成的值不承诺单调
排序。标识符不是认证或授权凭据。

## 时间语义

瞬间转换绝不根据数值大小猜测秒或毫秒。RFC 3339 输入必须包含 `Z` 或数字
offset，且小数秒最多三位。结果保持时区输入顺序并拒绝重复名称；每个结果都包含
规范的 Unix 秒、Unix 毫秒、UTC RFC 3339 和内置 tzdb 版本。

本地时间解析接受 `YYYY-MM-DDTHH:MM:SS` 和一个 IANA 时区：

- `UNAMBIGUOUS` 包含一个候选瞬间。
- `GAP` 不包含候选瞬间，只报告相邻的有效 offset。
- `FOLD` 明确包含 earlier 和 later 两个候选瞬间及其 offset。

## 稳定错误码

标识符错误码包括 `INVALID_UUID`、`INVALID_ULID`、`INVALID_IDENTIFIER`、
`BATCH_OUT_OF_RANGE`、`UNSUPPORTED_KIND`、`RANDOM_UNAVAILABLE`、
`CLOCK_UNAVAILABLE` 和 `SERIALIZATION_FAILED`。

时间错误码包括 `INVALID_TIMESTAMP`、`INVALID_RFC3339`、
`INVALID_LOCAL_DATETIME`、`INVALID_TIMEZONE`、`DUPLICATE_TIMEZONE`、
`TIMEZONE_LIMIT_EXCEEDED`、`INPUT_TOO_LONG` 和 `SERIALIZATION_FAILED`。
无效输入绝不会作为成功的 `{ valid: false }` 对象返回。

## 架构

- `uuid_factory_core` 负责标识符生成和严格检查。
- `timestamp_zone_core` 负责显式瞬间解析、内置 IANA 转换、时区查找以及本地
  gap/fold 解析。
- `uuid_factory_cli` 提供 `tinkora-id`；`timestamp_zone_cli` 提供
  `tinkora-time`。两个 CLI 都直接调用对应 core。
- `uuid_factory_web` 是唯一的 WASM package 和静态工作台，在不引入 server
  runtime 的前提下提供增量式标识符和时间 binding。

## 开发

完整本地验证需要 Rust `1.95.0`、`wasm32-unknown-unknown` target、`wasm-pack`、
Node.js 20 或更高版本，以及 npm：

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p uuid_factory_web --target wasm32-unknown-unknown --locked
wasm-pack test --node crates/uuid_factory_web --locked
cd crates/uuid_factory_web && npm run test:browser
ruby scripts/check_docs.rb
```

贡献流程见 [CONTRIBUTING.md](CONTRIBUTING.md)，私密漏洞报告见
[SECURITY.md](SECURITY.md)。

## 许可证

MIT。参见 [LICENSE](LICENSE) 与 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
