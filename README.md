# azvs_quote

## 开发手册

```bash
git push origin master
git push github master
```

```bash
cargo build --release
```

### 架构视角（DDD架构）
```mermaid
flowchart TB

subgraph Adapters
CLI[CLI]
end

subgraph Application
APP[Application Service]
end

subgraph Domain
PORTS[QuotePort / StoragePort]
end

subgraph Infrastructure
QINFRA[Quote Repository Impl]
SINFRA[Storage Service Impl]
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

### 启动装配流程（v0.2.0）
```mermaid
flowchart TD
    A["bin/cli main()"] --> B["ApplicationState::new().await"]
    B --> C["ApplicationConfig::load()"]
    C --> D["validate_semantics()"]

    D --> E{"database.backend"}
    E -->|"postgres"| F["PgPoolOptions::connect(url)"]
    F --> G["PostgresQuoteRepo::new(pool)"]
    E -->|"mysql"| E1["返回未实现错误"]

    D --> H{"storage.backend"}
    H -->|"minio"| I["MinioStorageRepo::new(config).await"]
    H -->|"file"| H1["返回未实现错误"]

    G --> J["ApplicationState { quote_port, storage_port, config }"]
    I --> J

    J --> K["adapter::cli::run(state)"]
    K --> L{"clap 子命令"}
    L -->|"get/list/create/update/delete"| M["application services"]
    M --> N["QuotePort trait"]
    M --> O["StoragePort trait"]
```

## 配置文件(v0.2.0)
> 默认读取：`~/.config/azvs/quote.toml`
```toml
[database]
backend = "postgres" # postgres | mysql

[database.postgres]
url = "postgres://azvs:azvs@azvs.lan:5432/azvs"
max_connections = 10
min_connections = 0

[database.mysql]
# todo
# url = "mysql://..."

[storage]
backend = "minio" # minio | file

[storage.minio]
endpoint = "https://minio.azvs.com"
access_key = "username"
secret_key = "password"
bucket = "quote"
region = "us-east-1"

[storage.file]
# todo
# root = "/data/quote"
```

## Quote-CLI
+ `quote get`
  + `--id <id>` 按 id 获取，不带 id 则随机获取。
    + 随机模式下，指定模板，按照模板是否存在过滤。
  + `--format '{{.id}}'` 模板输出。
+ `quote list`
  + `--page\--limit` 分页。
  + `--format '{{...}}'` 模板输出。
+ `quote create` 除 remark 外，其余参数均可使用多次。（相同语言模块下的相同语言会被覆盖）
  + `--inline <lang> <text>`
  + `--external <lang> <file>`
  + `--markdown <lang> <file>`
  + `--image <file>`
  + `--remark <text>`
+ `quote update`
  + `--id <id>` 必填
  + 参数风格与 create 一致
  + `--remark / --clear-remark`
  + 必须确认：--yes/-y 或交互输入 yes
  + 同一模块，多语言字段按语言粒度覆盖
  + image 为追加语义
+ `quote delete`
  + 整条删除：仅传 `--id <id>`
  + 部分删除:
    + `--inline <lang> / --all-inline`
    + `--external <lang> / --all-external`
    + `--markdown <lang> / --all-markdown`
    + `--image <object_key> / --all-image`
    + 必须确认：--yes/-y 或交互输入 yes
