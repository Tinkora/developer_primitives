# Developer Primitives 产品规格

[English](product_spec.md)

## 目的

Developer Primitives 帮助开发者和 agent 在本地生成、检查 UUID v4、UUID v7 和
ULID 标识符，并执行可复现的 IANA 时间转换。它提供一个静态浏览器工作台以及
可脚本化的 `tinkora-id` 和 `tinkora-time` CLI，不引入服务依赖。

## 支持的工作流

1. 生成单个标识符后复制。
2. 生成 1 到 10,000 个标识符，并复制或下载按生成顺序排列的换行结果。
3. 检查 UUID 或规范的大写 ULID，并查看结构化元数据。
4. 将显式指定类型的瞬间转换为 UTC 和 1 到 8 个按输入顺序排列的 IANA 时区。
5. 将一个 IANA 时区中的本地民用日期时间解析为 `UNAMBIGUOUS`、`GAP` 或
   `FOLD`，不静默修改输入，也不擅自选择 fold 候选。
6. 通过精确名称或有界文本筛选查找内置 IANA 时区名称。
7. 通过 `tinkora-id` 执行标识符工作流，通过 `tinkora-time` 执行时间工作流，
   使用人类可读输出或带 schema version 的 JSON 输出。

## 产品边界

版本 `0.2.0` 支持 UUID v4、UUID v7、规范的大写 ULID、显式瞬间转换、IANA
时区对比和本地民用时间解析。不支持托管 API、持久化、UUID v1/v3/v5/v6/v8、
自定义 UUID 布局、同一毫秒内单调生成、调度器、会议规划、日历运算、本地化
输出、闰秒模拟、NTP，也不提供可运行的 MCP server。

`skills/mcp-tools.json` 中的静态 schema 是未来集成使用的机器可读草案；它们不会
启动进程、打开 transport，也不使这些操作可被 MCP client 调用。

## 架构

- `uuid_factory_core` 负责 UUID/ULID 生成、解析、限制和稳定标识符错误。
- `timestamp_zone_core` 负责显式瞬间解析、IANA lookup、内置 tzdb 数据、按顺序
  转换、时区查找、本地时间解析、限制和稳定时间错误。
- `uuid_factory_cli` 提供 `tinkora-id`；`timestamp_zone_cli` 提供
  `tinkora-time`。两个 CLI 在工作过程中都不执行网络或文件访问。
- `uuid_factory_web` 保持为唯一的 WASM package，同时将两个 core 暴露给静态
  Identifiers 和 Time 浏览器模块。

## 标识符契约

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

## 时间契约

### 瞬间转换

调用方必须且只能选择一种输入类型：

- `unix-seconds`：有符号十进制整数。
- `unix-milliseconds`：有符号十进制整数。
- `rfc3339`：包含显式 `Z` 或数字 offset 的 RFC 3339，小数秒最多三位。

core 不根据数值大小推断单位，并拒绝不带 offset 的 RFC 3339 输入。每个成功
转换都包含 `schema_version`、规范的 Unix 秒和毫秒、规范的 UTC RFC 3339、
`tzdb_version`，以及每个请求时区的一个有序结果。时区结果包含规范名称、本地
日期时间、数字 offset、abbreviation 和 DST 状态。重复时区名称会被拒绝。

### 本地民用时间解析

本地输入必须使用不带 offset 的 `YYYY-MM-DDTHH:MM:SS`，并指定一个 IANA 时区。
带版本的结果包含规范时区、本地输入、内置 tzdb 版本和一个判别式解析结果：

- `UNAMBIGUOUS`：一个候选瞬间。
- `GAP`：没有候选瞬间，只包含 gap 前后的 offset。
- `FOLD`：earlier 和 later 两个候选瞬间及其 offset。

core 绝不会将 gap 移动到有效时间，也不会选择 fold 的一侧。

### 数据库与限制

所有时间接口统一使用内置的 IANA tzdb 2026c。契约接受 trim 后最多 128 UTF-8
bytes 的文本输入、最多 64 ASCII bytes 的时区名称，以及 1 到 8 个对比时区。
时区搜索不区分 ASCII 大小写，结果按名称排序且最多返回 50 项。

## CLI 契约

- `tinkora-id generate` 和 `tinkora-id inspect` 提供标识符能力。
- `tinkora-time convert` 必须接受一个显式瞬间 flag、重复的 `--zone` 和可选的
  `--json`。
- `tinkora-time resolve` 必须接受 `--local`、`--zone` 和可选的 `--json`。
- `tinkora-time zones` 接受用于精确查找的 `--name`、用于有界发现的 `--filter`
  和可选的 `--json`；两个查找参数都不传时，返回按名称排序结果的首个有界页面。

成功输出写入 stdout。稳定错误码和简明消息写入 stderr，退出码为 `1`；命令行
用法错误退出码为 `2`。JSON 结果使用 `schema_version: 1`。

## 浏览器契约

静态工作台保留产品 header，并提供 Identifiers 与 Time 的顶层切换。Time 模块
包含 Convert Instant 和 Resolve Local 模式、显式输入类型控件、可搜索且可移除的
时区列表、UTC 主摘要、按顺序排列的对比表、独立的 gap/fold 处理和显式复制操作。
浏览器与 CLI 使用相同的 WASM 契约和内置 tzdb。

## 稳定错误码

标识符错误码为 `INVALID_UUID`、`INVALID_ULID`、`INVALID_IDENTIFIER`、
`BATCH_OUT_OF_RANGE`、`UNSUPPORTED_KIND`、`RANDOM_UNAVAILABLE`、
`CLOCK_UNAVAILABLE` 和 `SERIALIZATION_FAILED`。

时间错误码为 `INVALID_TIMESTAMP`、`INVALID_RFC3339`、
`INVALID_LOCAL_DATETIME`、`INVALID_TIMEZONE`、`DUPLICATE_TIMEZONE`、
`TIMEZONE_LIMIT_EXCEEDED`、`INPUT_TOO_LONG` 和 `SERIALIZATION_FAILED`。
错误码、结果 discriminant 和 JSON field 含义构成兼容性契约；人类可读消息在
`1.0` 之前可以改进。

## 隐私与安全

- 浏览器操作通过 WebAssembly 在本地执行。
- 页面不使用遥测、Cookie、local storage、CDN、远程字体或时区 API。
- OS 或 Web Crypto 随机源不可用时，生成操作会显式失败，绝不回退到弱随机数。
- 标识符和时间输入限制会在解析或分配前执行。
- 剪贴板和文件下载必须由用户显式触发。

UUID v4 的随机性适合生成标识符，但不是访问控制机制。应用仍必须对每项资源进行
授权校验。

## 验证

- Rust 测试覆盖标识符 bit layout 与解析，以及时间解析、公开 IANA transition、
  纽约 gap/fold 精确候选、边界和稳定错误。
- CLI process 测试根据 core 契约验证 stdout、stderr、退出码、时区顺序和带
  schema version 的 JSON。
- Node/WASM 测试调用标识符和时间 export，并验证结构化错误和 IANA tzdb 2026c。
- 浏览器测试在 375、768、1024、1440 像素宽度上运行标识符和 Time 工作流，
  包括键盘操作、accessibility name、复制状态、无横向溢出、无外部请求和零
  runtime error。
- 文档检查验证 UTF-8、本地链接、双语 README/规格入口、公开时间标记、schema
  草案边界和已停用链接。
