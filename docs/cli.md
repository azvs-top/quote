# CLI 文档

> 程序会先解析命令参数，再初始化数据库/存储依赖。

## `quote get`

用途：获取一条 quote（按 id 或随机）。

常用参数：
- `--id <id>`：按 id 获取；不传则随机
- `--format <tpl>`：直接指定模板
- `--format-preset <name>`：使用配置中的模板预设
- `--image-ascii` / `--image-view`：控制模板中 `$image` 的渲染模式

示例：

```bash
quote get
quote get --id 1
quote get --format '{{.inline.zh}}\n{{.inline.en}}'
quote get --format-preset full
quote get --format '{{$image.0}}' --image-ascii
```

## `quote list`

用途：分页列出 quote。

常用参数：
- `--page <n>`：页码，默认 `1`
- `--limit <n>`：每页数量，默认 `10`
- `--format <tpl>` / `--format-preset <name>`

说明：
- `quote list` 中 `$image` 固定按 `meta` 输出，不支持 ascii/view 模式切换

示例：

```bash
quote list
quote list --page 2 --limit 5
quote list --limit 20 --format '{{.id}}\t{{.inline.en}}'
quote list --format-preset brief
```

## `quote create`

用途：创建一条新 quote。

常用参数：
- `--inline <lang> <text>`（可重复）
- `--external <lang> <file>`（可重复）
- `--markdown <lang> <file>`（可重复）
- `--image <file>`（可重复）
- `--remark <text>`

示例：

```bash
quote create --inline en "hello" --inline zh "你好"
quote create --external en ./en.txt --markdown zh ./zh.md
quote create --image ./a.png --image ./b.jpg --remark "demo"
```

## `quote update`

用途：按 patch 语义更新已有 quote。

常用参数：
- `--id <id>`：必填
- `--inline/--external/--markdown`：用法与 `create` 一致
- `--image <file>`：追加图片到末尾，不会覆盖已有图片
- `--remark <text>`：设置备注
- `--clear-remark`：清空备注
- `-y, --yes`：跳过二次确认

示例：

```bash
quote update --id 1 --inline en "hello" -y
quote update --id 1 --markdown zh ./a.md --image ./a.png -y
quote update --id 1 --remark "new" -y
quote update --id 1 --clear-remark -y
```

## `quote delete`

用途：删除整条 quote，或删除指定字段内容。

常用参数：
- `--id <id>`：必填，若仅填写 id 则删除整条 quote
- `--inline <lang>` / `--all-inline`
- `--external <lang>` / `--all-external`
- `--markdown <lang>` / `--all-markdown`
- `--image-key <object_key>` 或 `--image-index <index>` / `--all-image`
- `-y, --yes`：跳过二次确认

示例：

```bash
quote delete --id 1 -y
quote delete --id 1 --markdown zh -y
quote delete --id 1 --all-inline -y
quote delete --id 1 --image-index 0 -y
```

## `quote download`

用途：下载 external / markdown / image 对象。

常用参数：
- `--id <id>`：必填
- 目标参数必须三选一：
  - `--external <lang>`
  - `--markdown <lang>`
  - `--image <index>`
- `--out <path>`：输出路径（父目录不存在会自动创建）

示例：

```bash
quote download --id 1 --external en --out ./en.txt
quote download --id 1 --markdown zh --out ./zh.md
quote download --id 1 --image 0 --out ./a.png
```

## 模板语法（`--format`）

字段读取（不访问对象存储）：
- `{{.id}}`
- `{{.inline.en}}`
- `{{.external.en}}`（返回对象 key）
- `{{.markdown.zh}}`（返回对象 key）
- `{{.image.0}}`（返回对象 key）

对象读取（会访问对象存储）：
- `{{$external.en}}`
- `{{$markdown.zh}}`
- `{{$image}}`（输出图片 meta 数组 JSON）
- `{{$image.0}}`（单图输出，受 `cli.format.get_image_mode/--image-ascii/--image-view` 影响）

支持转义：
- `\n` `\t` `\r` `\\` `\"` `\'`

模板优先级：
1. 命令行 `--format`
2. 命令行 `--format-preset`
3. `quote.toml` 中 `cli.format.default_get/default_list`
4. 都没有时输出 JSON
