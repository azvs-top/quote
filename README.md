# azvs_quote

`azvs_quote` 是一个以 CLI 为主的 Quote 管理工具，采用 DDD 分层，支持：
- Quote 数据 CRUD
- 对象存储（external/markdown/image）上传下载
- `--format` 模板渲染（`.path` / `$path`）
- 图片 `meta` / `ascii` / `view` 三种输出模式

当前仓库版本：`0.2.3`

## 快速笔记（Git 提交）

```bash
git push origin master
git push github master
```

## 快速开始（30 秒）

```bash
cargo build --release
./target/release/quote list --limit 3
```

默认行为：
- 数据库 backend 默认是 `sqlite`
- 数据库文件默认是 `~/.config/azvs/quote.db`
- 不会自动初始化数据库；需手动创建库和表
- 存储 backend 默认是 `none`（不依赖 MinIO）

## 配置

默认配置文件路径：
- Linux: `~/.config/azvs/quote.toml`
- macOS: `~/Library/Application Support/azvs/quote.toml`
- Windows: `%APPDATA%\\azvs\\quote.toml`

可通过环境变量覆盖：
- `AZVS_QUOTE_CONFIG=/your/path/quote.toml`

### 最小配置

`quote.toml` 可以为空文件，程序可启动（使用默认配置）。
但 sqlite 需要你先手动准备好 `~/.config/azvs/quote.db`（或自定义 `database.sqlite.path`）并初始化表结构。

如果你希望显式写最小配置，推荐：

```toml
[database]
backend = "sqlite"

[storage]
backend = "none"
```

### 完整配置示例

```toml
[database]
backend = "sqlite" # sqlite(默认) | postgres | mysql(未实现)

[database.sqlite]
path = "~/.config/azvs/quote.db" # 可选；不写则默认 <config_dir>/quote.db

[database.postgres]
url = "postgres://azvs:azvs@127.0.0.1:5432/azvs"
max_connections = 10
min_connections = 0

[storage]
backend = "none" # none(默认) | minio | file(未实现)

[storage.minio]
endpoint = "https://minio.example.com"
access_key = "username"
secret_key = "password"
bucket = "quote"
region = "us-east-1"
secure = true

[cli.format]
default_get = "{{.inline.en}}\n{{.inline.zh}}"
default_list = "{{.id}}\t{{.inline.en}}"
image_mode = "meta" # meta | ascii | view

[cli.format.presets]
brief = "{{.id}}: {{.inline.en}}"
full = "{{}}"
```

## CLI 命令

> 先解析命令参数，再初始化依赖（数据库/存储）。

### `quote get`
- `--id <id>`：按 id 获取；不传则随机获取
- `--format <tpl>` / `--format-preset <name>`
- `--image-ascii` / `--image-view`

### `quote list`
- `--page <n>`（默认 1）
- `--limit <n>`（默认 10）
- `--format <tpl>` / `--format-preset <name>`
- `--image-ascii` / `--image-view`

### `quote create`
- `--inline <lang> <text>`（可重复）
- `--external <lang> <file>`（可重复）
- `--markdown <lang> <file>`（可重复）
- `--image <file>`（可重复）
- `--remark <text>`

### `quote update`
- `--id <id>`（必填）
- 其余参数与 `create` 类似（patch 语义）
- `--remark <text>` 或 `--clear-remark`
- 默认二次确认；`-y/--yes` 跳过

### `quote delete`
- 整条删除：`quote delete --id <id> -y`
- 部分删除：
- `--inline <lang>` / `--all-inline`
- `--external <lang>` / `--all-external`
- `--markdown <lang>` / `--all-markdown`
- `--image-key <object_key>` 或 `--image-index <index>` / `--all-image`

### `quote download`
- `--id <id>` + 严格三选一目标：
- `--external <lang>` 或 `--markdown <lang>` 或 `--image <index>`
- `--out <path>`（父目录不存在会自动创建）

## 模板语法（`--format`）

`{{.path}}`：读取 Quote 字段，不访问对象存储。

常见示例：
- `{{.id}}`
- `{{.inline.en}}`
- `{{.external.en}}`（返回对象 key）
- `{{.markdown.zh}}`（返回对象 key）
- `{{.image.0}}`（返回对象 key）

`{{$path}}`：读取扩展对象内容（会访问对象存储）。

常见示例：
- `{{$external.en}}`
- `{{$markdown.zh}}`
- `{{$image}}`（输出图片 meta 数组 JSON）
- `{{$image.0}}`（单图输出，受 `image_mode/--image-ascii/--image-view` 影响）

支持转义：
- `\n` `\t` `\r` `\\` `\"` `\'`

模板优先级：
1. 命令行 `--format`
2. 命令行 `--format-preset`
3. `quote.toml` 的 `default_get/default_list`
4. 都没有时输出 JSON

## 示例

```bash
quote get --id 1
quote get --format '{{.inline.zh}}\n{{.inline.en}}'
quote get --format '{{$image.0}}' --image-ascii

quote list --limit 20 --format '{{.id}}\t{{.inline.en}}'
quote list --format-preset brief

quote create --inline en "hello" --inline zh "你好" --remark "demo"
quote update --id 1 --markdown zh ./a.md -y
quote delete --id 1 --all-markdown -y

quote download --id 1 --external en --out ./en.txt
```

## 构建与测试

```bash
cargo build --release
cargo test
```

## 架构概览

```mermaid
flowchart TB

subgraph Adapters
CLI[CLI]
end

subgraph Application
APP[Application Services]
end

subgraph Domain
PORTS[QuotePort / StoragePort]
end

subgraph Infrastructure
QINFRA[Quote Repository Impl]
SINFRA[Storage Repository Impl]
end

subgraph External
DB[(Sqlite / Postgres)]
OBJ[(MinIO / None)]
end

CLI --> APP
APP --> PORTS
PORTS --> QINFRA
PORTS --> SINFRA
QINFRA --> DB
SINFRA --> OBJ
```

## 已知待办

- 使用 `sqlx migrate` 替代手工 SQL 初始化管理
