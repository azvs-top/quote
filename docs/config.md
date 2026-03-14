# 配置参考（`quote.toml`）

本文档聚焦 `quote.toml` 的完整配置项，按“默认值、必填条件、组合约束、示例场景”组织。

## 1. 配置加载规则

### 1.1 配置文件路径

优先级：
1. 环境变量 `AZVS_QUOTE_CONFIG`
2. 默认路径

默认路径：
- Linux: `~/.config/azvs/quote.toml`
- macOS: `~/Library/Application Support/azvs/quote.toml`
- Windows: `%APPDATA%\azvs\quote.toml`

### 1.2 环境变量覆盖

环境变量使用 `__` 作为层级分隔符。  
例如：

```bash
DATABASE__BACKEND=postgres
DATABASE__POSTGRES__URL=postgres://user:pass@127.0.0.1:5432/azvs
STORAGE__BACKEND=none
```

## 2. 最小可用配置

`quote.toml` 可以是空文件。默认等价于：

```toml
[database]
backend = "sqlite"

[storage]
backend = "none"
```

说明：
- 默认 sqlite 数据库文件路径是 `~/.config/azvs/quote.db`
- 项目不会自动初始化 sqlite 结构，需你手动建库并执行 SQL
- 默认不支持对象存储，可使用inline模块，配置storage.backend获得完整功能

## 3. 全量键说明

## 3.1 `[database]`

| 键 | 类型 | 默认值 | 必填 | 说明 |
|---|---|---|---|---|
| `backend` | string | `"sqlite"` | 否 | 可选值：`sqlite` / `postgres` / `mysql` |

### 3.1.1 `[database.sqlite]`

| 键 | 类型 | 默认值 | 必填 | 说明 |
|---|---|---|---|---|
| `path` | string | `<config_dir>/quote.db` | 否 | sqlite 文件路径；支持 `~/` 展开；相对路径按 `quote.toml` 所在目录解析 |

### 3.1.2 `[database.postgres]`

当 `database.backend = "postgres"` 时该段必填。

| 键 | 类型 | 默认值 | 必填 | 说明 |
|---|---|---|---|---|
| `url` | string | 无 | 是 | Postgres 连接串 |
| `max_connections` | integer | `10` | 否 | 连接池最大连接数 |
| `min_connections` | integer | `0` | 否 | 连接池最小连接数 |

### 3.1.3 `[database.mysql]`

当 `database.backend = "mysql"` 时该段语义上必填，但当前实现未完成，启动会报“not implemented yet”。

| 键 | 类型 | 默认值 | 必填 | 说明 |
|---|---|---|---|---|
| `url` | string | 无 | 是 | MySQL 连接串 |
| `max_connections` | integer | 无 | 否 | 连接池最大连接数 |
| `min_connections` | integer | 无 | 否 | 连接池最小连接数 |

## 3.2 `[storage]`

| 键 | 类型 | 默认值 | 必填 | 说明 |
|---|---|---|---|---|
| `backend` | string | `"none"` | 否 | 可选值：`none` / `minio` / `file` |

### 3.2.1 `[storage.minio]`

当 `storage.backend = "minio"` 时该段必填。

| 键 | 类型 | 默认值 | 必填 | 说明 |
|---|---|---|---|---|
| `endpoint` | string | 无 | 是 | MinIO/S3 兼容端点 |
| `access_key` | string | 无 | 是 | 访问密钥 |
| `secret_key` | string | 无 | 是 | 密钥 |
| `bucket` | string | 无 | 是 | 桶名 |
| `region` | string | `"us-east-1"` | 否 | 区域 |
| `secure` | bool | `false` | 否 | 是否启用 https |

### 3.2.2 `[storage.file]`

当 `storage.backend = "file"` 时该段语义上必填，但当前实现未完成，启动会报“not implemented yet”。

| 键 | 类型 | 默认值 | 必填 | 说明 |
|---|---|---|---|---|
| `root` | string | 无 | 否 | 文件存储根目录 |

## 3.3 `[cli.format]`

| 键 | 类型 | 默认值 | 必填 | 说明 |
|---|---|---|---|---|
| `default_get` | string | 无 | 否 | `quote get` 默认模板 |
| `default_list` | string | 无 | 否 | `quote list` 默认模板 |
| `get_image_mode` | string | `"meta"` | 否 | `quote get` 的图片渲染模式；可选值：`meta` / `ascii` / `view` |

### 3.3.1 `[cli.format.presets]`

字符串字典，key 为预设名，value 为模板内容。  
示例：

```toml
[cli.format.presets]
brief = "{{.id}}: {{.inline.en}}"
full = "{{}}"
```

## 4. 组合约束（语义校验）

- `database.backend = "postgres"` 时必须提供 `[database.postgres]`
- `database.backend = "mysql"` 时必须提供 `[database.mysql]`（但当前仍未实现）
- `storage.backend = "minio"` 时必须提供 `[storage.minio]`
- `storage.backend = "file"` 时必须提供 `[storage.file]`（但当前仍未实现）

## 5. 场景示例

### 5.1 空配置（默认 sqlite + none）

```toml
# empty
```

### 5.2 sqlite（显式路径）

```toml
[database]
backend = "sqlite"

[database.sqlite]
path = "~/.config/azvs/quote.db"

[storage]
backend = "none"
```

### 5.3 postgres + none

```toml
[database]
backend = "postgres"

[database.postgres]
url = "postgres://azvs:azvs@127.0.0.1:5432/azvs"
max_connections = 10
min_connections = 0

[storage]
backend = "none"
```

### 5.4 postgres + minio

```toml
[database]
backend = "postgres"

[database.postgres]
url = "postgres://azvs:azvs@127.0.0.1:5432/azvs"

[storage]
backend = "minio"

[storage.minio]
endpoint = "https://minio.example.com"
access_key = "username"
secret_key = "password"
bucket = "quote"
region = "us-east-1"
secure = true
```
