# azvs_quote

一个基于 Rust 的 quote 服务，包含 CLI 和 HTTP 两种入口。

## 运行

```bash
# CLI
cargo run --bin quote -- <args>

# HTTP
cargo run --bin quote-http
```

## 配置文件

默认读取：`~/.config/azvs/quote.toml`

最小示例：

```toml
[storage]
backend = "pgsql"

[storage.pgsql]
url = "postgres://user:pass@127.0.0.1:5432/quote"

[quote]
inline_langs = ["en"]
system_lang = "en"

[http]
addr = "127.0.0.1:3000"
cors_enabled = true
cors_allow_credentials = true
cors_origins = ["http://localhost:2002", "http://localhost:32002", "https://quote.azvs.lan", "https://quote.azvs.top"]
cors_methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"]
cors_headers = ["content-type","authorization"]
```

## 已实现 CLI 命令

1. `quote get`
- `--id <ID>`: 按 ID 获取单条 quote（JSON）。
- `--page <N> --limit <M> [--active <true|false>]`: 分页获取列表（JSON）。
- 不带 `--id/--page`: 随机获取一条（按 `quote.inline_langs` 输出 inline 文本）。

2. `quote add`
- 支持参数收集与校验：
`--lang <LANG> <TEXT>`、`--file <LANG> <FILE>`、`--md <LANG> <FILE>`、`--image <FILE>`
- 当前状态：仅校验并打印草稿，持久化尚未实现。

3. `quote dict get`
- 支持 `--active --page --limit --json`
- 默认表格输出，`--json` 输出 JSON。

4. `quote dict-item get <TYPE>`
- 支持 `--active --page --limit --json`
- 默认表格输出，`--json` 输出 JSON。

## 已实现 HTTP 接口

Base: `http://127.0.0.1:3000`

1. `GET /hello`
- 测试接口，返回服务状态。

2. `GET /quote/random`
- 随机获取一条 quote。
- 查询参数：`active`（可选）。

3. `GET /quote/{id}`
- 按 ID 获取 quote。

4. `GET /quotes`
- 获取 quote 列表。
- 查询参数：`active`、`page`、`limit`（可选）。

## Git 推送（Gitea + GitHub）

手动分别推送：

```bash
git push origin master
git push github master
```