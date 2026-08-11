# Developer Primitives Agent Reference

This file documents how an agent can use the released `tinkora-id` CLI. It is
not an installable Agent Skill and does not expose an MCP server.

## Generate Identifiers

```bash
tinkora-id generate --kind uuid-v4 --count 1
tinkora-id generate --kind uuid-v7 --count 10 --json
tinkora-id generate --kind ulid --count 10 --json
```

Kinds are exactly `uuid-v4`, `uuid-v7`, and `ulid`. Count is an integer from 1
through 10,000. Text mode prints one identifier per line. JSON mode returns
`schema_version`, `kind`, `count`, and the ordered `identifiers` array.

## Inspect An Identifier

```bash
tinkora-id inspect 550e8400-e29b-41d4-a716-446655440000 --json
printf '%s\n' '01ARZ3NDEKTSV4RRFFQ69G5FAV' | tinkora-id inspect --json
```

Inspection accepts an argument or one newline-terminated value from standard
input. It returns the canonical value, kind, and optional UUID version, variant,
or embedded millisecond timestamp. Lowercase ULID input is rejected.

## Automation Rules

- Check the exit code before parsing output: `0` is success, `2` is command-line
  usage error, and `1` is an operational error.
- Parse JSON only from standard output. Stable error codes are written to
  standard error.
- Do not treat identifiers as credentials or authorization proof.
- Do not promise monotonic order within the same millisecond.
- Do not describe `skills/mcp-tools.json` as callable. It is a static draft
  schema for possible future integration.
