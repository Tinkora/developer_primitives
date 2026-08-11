# Developer Primitives 产品规格

[English](product_spec.md)

## 目的

Developer Primitives 帮助开发者和 agent 在本地生成、检查 UUID v4、UUID v7 和
ULID 标识符。它提供静态浏览器工作台与可脚本化 CLI，不引入服务依赖。

## 支持的工作流

1. 生成单个标识符后复制。
2. 生成 1 到 10,000 个标识符，并复制或下载按生成顺序排列的换行结果。
3. 检查 UUID 或规范的大写 ULID，并查看结构化元数据。
4. 通过 `tinkora-id` 使用同样的生成和检查能力。

## 产品边界

首个版本支持 UUID v4、UUID v7 和规范的大写 ULID。不支持托管 API、持久化、
UUID v1/v3/v5/v6/v8、自定义 UUID 布局、同一毫秒内的单调生成，也不提供可运行的
MCP server。

`skills/mcp-tools.json` 中的静态 schema 是未来集成使用的机器可读草案；它们不会
启动进程、打开 transport，也不使这些操作可被 MCP client 调用。

## 公开契约

生成操作接受 `uuid-v4`、`uuid-v7` 或 `ulid`，数量必须在 1 到 10,000 之间。
批量输出保持生成顺序。

检查操作返回带版本的结构：

```json
{
  "schema_version": 1,
  "input": "01890f3e-e7c8-7cc3-98c8-4c0a1d2b3c4d",
  "canonical": "01890f3e-e7c8-7cc3-98c8-4c0a1d2b3c4d",
  "kind": "uuid",
  "version": 7,
  "variant": "RFC4122",
  "timestamp_ms": 1688177928136
}
```

ULID 检查不包含 UUID version/variant，但会包含其时间戳。无效输入返回稳定错误，
绝不会以成功的 `valid: false` 响应表示。

## 隐私与安全

- 浏览器操作通过 WebAssembly 在本地执行。
- 页面不使用遥测、Cookie、local storage 或远程字体。
- OS 或 Web Crypto 随机源不可用时，生成操作会显式失败，绝不回退到弱随机数。
- 解析前，输入被限制为最多 128 UTF-8 bytes。
- 分配内存前会校验批量数量。
- 剪贴板和下载必须由用户显式触发。

UUID v4 的随机性适合生成标识符，但不是访问控制机制。应用仍必须对每项资源进行
授权校验。

## 验证

- Rust 测试覆盖 RFC bit layout、固定时间戳、边界、严格解析、随机源/时钟失败、CLI
  输出与退出码，以及 WASM 错误。
- 浏览器测试在 375、768、1024、1440 像素宽度上验证工作流、键盘操作、live status、
  对比度、无横向溢出、无外部请求和零运行时错误。
- 文档检查验证 UTF-8、本地链接、双语 README/规格入口、schema 草案边界和旧仓库链接。
