# Developer Primitives

[English](README.md) | [产品规格](docs/product_spec.zh-CN.md) |
[更新日志](CHANGELOG.md) | [贡献指南](CONTRIBUTING.md)

Developer Primitives 是一个本地优先的 UUID v4、UUID v7 与 ULID 工作台和
命令行工具。浏览器应用将 Rust 编译为 WebAssembly 后在本机运行，不发送应用
网络请求；`tinkora-id` CLI 为脚本和 AI agent 提供同一套契约。

## 能力

- 生成 UUID v4、UUID v7 和规范的大写 ULID。
- 按生成顺序批量产出 1 到 10,000 个标识符。
- 严格检查 UUID 与 ULID，并在适用时返回 UUID 版本、变体和 v7/ULID 时间戳。
- Rust core、CLI 与 WebAssembly bridge 共享稳定的机器可读错误码。

本项目不是标识符分配服务、托管 API、数据库，也不是可运行的 MCP server。
[`skills/`](skills/) 下的 schema 仅是文档草案。

## 使用浏览器工作台

首个版本发布后，工作台将由 GitHub Pages 提供。本地运行：

```bash
cd crates/uuid_factory_web
npm ci
npm run build:wasm
python3 -m http.server 8080 --bind 127.0.0.1 --directory static
```

在浏览器打开 `http://127.0.0.1:8080`。生成值、待检查输入和剪贴板操作均保留
在浏览器内存中；页面不使用遥测、Cookie、持久化存储或外部字体。

## 使用 CLI

CLI 将结果写入标准输出，将诊断信息写入标准错误：

```bash
cargo run -p uuid_factory_cli --bin tinkora-id -- generate --kind uuid-v7 --count 3
cargo run -p uuid_factory_cli --bin tinkora-id -- generate --kind ulid --count 2 --json
printf '%s\n' '550e8400-e29b-41d4-a716-446655440000' \
  | cargo run -p uuid_factory_cli --bin tinkora-id -- inspect --json
```

生成类型为 `uuid-v4`、`uuid-v7` 和 `ulid`。JSON 输出使用
`schema_version: 1`。成功退出码为 `0`；命令行用法错误为 `2`；运行错误为 `1`，
并包含稳定错误码。

## 标识符语义

| 类型 | 规范输出 | 时间戳 | 适用场景 |
| --- | --- | --- | --- |
| UUID v4 | 小写、带连字符的 UUID | 无 | 随机标识符 |
| UUID v7 | 小写、带连字符的 UUID | Unix 毫秒 | 需要时间排序的标识符 |
| ULID | 大写 Crockford Base32 | Unix 毫秒 | 更短、需要时间排序的标识符 |

UUID v7 和 ULID 能按不同毫秒时间戳排序；同一毫秒内生成的值不承诺单调排序。
标识符不是认证或授权凭据。

## 稳定错误码

`INVALID_UUID`、`INVALID_ULID`、`INVALID_IDENTIFIER`、
`BATCH_OUT_OF_RANGE`、`UNSUPPORTED_KIND`、`RANDOM_UNAVAILABLE`、
`CLOCK_UNAVAILABLE` 和 `SERIALIZATION_FAILED` 是公开契约的一部分。
无效输入不会作为成功的 `{ valid: false }` 对象返回。

## 开发

完整本地验证需要 Rust `1.95.0`、`wasm32-unknown-unknown` target、`wasm-pack`、
Node.js 20 或更高版本，以及 npm：

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
wasm-pack test --node crates/uuid_factory_web --locked
cd crates/uuid_factory_web && npm run test:browser
ruby scripts/check_docs.rb
```

贡献流程见 [CONTRIBUTING.md](CONTRIBUTING.md)，私密漏洞报告见
[SECURITY.md](SECURITY.md)。

## 许可证

MIT。参见 [LICENSE](LICENSE) 与 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
