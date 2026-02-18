# azvs_quote

`azvs_quote` 是一个基于 DDD 分层的 Quote 管理工具，当前主入口是 CLI（`quote`），支持：
- quote 数据 CRUD
- MinIO 对象上传/下载（external/markdown/image）
- `--format` 模板渲染（`.path` 与 `$path`）
- 图片在终端中的 meta/ascii/view 三种输出模式

当前版本：`0.2.1`

## 常用发布命令
```bash
git push origin master
git push github master
```

## 构建与测试
```bash
cargo build --release
cargo test
```

## 架构视角（DDD）
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
DB[(Postgres / MySQL)]
OBJ[(MinIO / File)]
end

CLI --> APP
APP --> PORTS
PORTS --> QINFRA
PORTS --> SINFRA
QINFRA --> DB
SINFRA --> OBJ
```

## 配置
默认配置路径：
- Linux: `~/.config/azvs/quote.toml`
- macOS: `~/Library/Application Support/azvs/quote.toml`
- Windows: `%APPDATA%\\azvs\\quote.toml`

可通过环境变量覆盖配置文件路径：
- `AZVS_QUOTE_CONFIG=/your/path/quote.toml`

示例配置：
```toml
[database]
backend = "postgres" # postgres | mysql(未实现)

[database.postgres]
url = "postgres://azvs:azvs@127.0.0.1:5432/azvs"
max_connections = 10
min_connections = 0

[storage]
backend = "minio" # minio | file(未实现)

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

`--format` 选择优先级：
1. 命令行 `--format`
2. 命令行 `--format-preset`
3. `quote.toml` 中 `default_get/default_list`
4. 都未提供时输出 JSON

## CLI 命令
`quote get`
- `--id <id>`：按 id 获取；未指定则随机获取
- `--format <tpl>` / `--format-preset <name>`
- `--image-ascii`：仅影响模板中的 `{{$image.<index>}}`
- `--image-view`：仅影响模板中的 `{{$image.<index>}}`，优先终端直出，失败回退

`quote list`
- `--page <n> --limit <n>`
- `--format <tpl>` / `--format-preset <name>`
- `--image-ascii`
- `--image-view`

`quote create`
- `--inline <lang> <text>`（可重复）
- `--external <lang> <file>`（可重复）
- `--markdown <lang> <file>`（可重复）
- `--image <file>`（可重复）
- `--remark <text>`

`quote update`
- `--id <id>` 必填
- 其余参数与 `create` 风格一致
- `--remark <text>` 或 `--clear-remark`
- 默认二次确认；可用 `-y/--yes` 跳过

`quote delete`
- 整条删除：`quote delete --id <id> -y`
- 部分删除：
- `--inline <lang>` / `--all-inline`
- `--external <lang>` / `--all-external`
- `--markdown <lang>` / `--all-markdown`
- `--image <object_key>` / `--all-image`
- 默认二次确认；可用 `-y/--yes` 跳过

`quote download`
- `--id <id>` + 严格三选一目标：
- `--external <lang>` 或 `--markdown <lang>` 或 `--image <index>`
- `--out <path>` 输出文件路径（父目录不存在会自动创建）

## `--format` 模板语法
`{{.path}}`：读取 Quote 字段（不会下载对象）
- `{{.id}}`
- `{{.inline.en}}`
- `{{.external.en}}`（返回对象 key）
- `{{.markdown.zh}}`（返回对象 key）
- `{{.image.0}}`（返回对象 key）

`{{$path}}`：读取扩展对象内容或派生结果（会访问存储）
- `{{$external.en}}`：下载并输出文本
- `{{$markdown.zh}}`：下载并输出 markdown 原文
- `{{$image}}`：输出全部图片 meta 数组（JSON 字符串）
- `{{$image.0}}`：输出单张图片，模式受 `image_mode/--image-ascii/--image-view` 影响

模板字符串支持常见转义：
- `\n` `\t` `\r` `\\` `\"` `\'`

## 使用示例
```bash
quote get --id 1
quote get --format '{{.inline.zh}}\n{{.inline.en}}'
quote get --format '{{$external.en}}'
quote get --format '{{$image.0}}' --image-ascii

quote list --page 1 --limit 20 --format '{{.id}}\t{{.inline.en}}'
quote list --format-preset brief

quote create --inline en "hello" --inline zh "你好" --remark "demo"
quote update --id 1 --markdown zh ./a.md -y
quote delete --id 1 --all-markdown -y

quote download --id 1 --external en --out ./en.txt
quote download --id 1 --markdown zh --out ./zh.md
quote download --id 1 --image 0 --out ./0.bin
```
