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

``` toml
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

``` bash
git push origin master
git push github master
```

# v0.2.0 开端

### QuotePort（面向 Quote 聚合）

1. create(QuoteCreate) -> Result<Quote, AppError>
2. get(query: QuoteQuery) -> Result<Quote, AppError>
3. list(query: QuoteQuery) -> Result<Vec<Quote>, AppError>
4. update(QuoteUpdate) -> Result<Quote, AppError>
5. delete(id: i64) -> Result<(), AppError>（可选）

### StoragePort（面向对象存储）

1. upload(path: &str, payload: StoragePayload, content_type: &str) ->
   Result<ObjectKey, AppError>
2. delete(key: &ObjectKey) -> Result<(), AppError>（补偿事务建议必
   备）
3. exists(key: &ObjectKey) -> Result<bool, AppError>（可选）
4. download(key: &ObjectKey) -> Result<Vec<u8>, AppError>（可选，若后
   续要下载）

### Quote Entity
``` rust
- id: i64
- inline: HashMap<Lang, String>
- external: HashMap<Lang, ObjectKey>
- markdown: HashMap<Lang, ObjectKey>
- image: Vec<ObjectKey>
- remark: Option<String>
```
+ QuoteCreate： 不包含 id
+ QuoteUpdate： id + 可选字段
+ QuoteQuery： id + filter + limit + offset
+ struct Lang(String)：在new中做格式校验
+ struct ObjectKey(String)：在new中避免非法路径

# 配置文件
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
endpoint = "http://azvs.lan:9000"
access_key = "root"
secret_key = "rootroot"
bucket = "quote"
region = "us-east-1"

[storage.file]
# todo
# root = "/data/quote"
```