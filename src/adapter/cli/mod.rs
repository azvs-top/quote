mod confirm;
mod format;
mod handlers;
mod output;
use crate::application::ApplicationState;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // 先完成 CLI 参数校验，再初始化外部依赖，避免缺参时提前连接数据库/对象存储。
    let state = ApplicationState::new().await?;
    match cli.command {
        Command::Get(args) => handlers::handle_get(&state, args).await?,
        Command::List(args) => handlers::handle_list(&state, args).await?,
        Command::Create(args) => handlers::handle_create(&state, args).await?,
        Command::Update(args) => handlers::handle_update(&state, args).await?,
        Command::Delete(args) => handlers::handle_delete(&state, args).await?,
        Command::Download(args) => handlers::handle_download(&state, args).await?,
    }
    Ok(())
}

#[derive(Parser)]
#[command(
    name = "quote",
    version,
    about = "Quote 命令行工具",
    long_about = "管理 quote 的命令行工具，支持 get/list/create/update/delete/download。",
    after_help = r#"示例:
  quote get
  quote get --id 1 --format '{{.inline.zh}}\n{{.inline.en}}'
  quote list --page 1 --limit 5 --format-preset full
  quote create --inline en "hello" --inline zh "你好" --image ./a.png
  quote update --id 1 --markdown zh ./a.md -y
  quote delete --id 1 -y
  quote download --id 1 --external en --out ./en.txt"#
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 获取一条 quote（可按 id，或随机）
    Get(GetArgs),
    /// 列出 quote（分页）
    List(ListArgs),
    /// 创建 quote
    Create(CreateArgs),
    /// 更新 quote（patch 语义）
    Update(UpdateArgs),
    /// 删除 quote（整条或部分字段）
    Delete(DeleteArgs),
    /// 下载 quote 关联对象（external/markdown/image）
    Download(DownloadArgs),
}

#[derive(clap::Args)]
#[command(after_help = r#"示例:
  quote get
  quote get --id 3
  quote get --format '{{.inline.zh}}\n{{.inline.en}}'"#)]
pub(super) struct GetArgs {
    #[arg(long = "id", help = "按 id 获取；未指定时为随机获取")]
    pub(super) id: Option<i64>,
    #[arg(
        long = "format",
        help = "模板输出，例如 '{{.inline.zh}}\\n{{.inline.en}}'"
    )]
    pub(super) format: Option<String>,
    #[arg(
        long = "format-preset",
        conflicts_with = "format",
        help = "使用配置文件 [cli.format.presets] 中的模板名称"
    )]
    pub(super) format_preset: Option<String>,
    #[arg(
        long = "image-ascii",
        default_value_t = false,
        conflicts_with = "image_view",
        help = "模板中 $image 的输出模式使用 ascii"
    )]
    pub(super) image_ascii: bool,
    #[arg(
        long = "image-view",
        default_value_t = false,
        conflicts_with = "image_ascii",
        help = "模板中 $image 的输出模式使用 view（终端直出优先）"
    )]
    pub(super) image_view: bool,
}

#[derive(clap::Args)]
#[command(after_help = r#"示例:
  quote list
  quote list --page 2 --limit 5
  quote list --format '{{.id}} {{.inline.en}}'"#)]
pub(super) struct ListArgs {
    #[arg(long = "page", default_value_t = 1, help = "页码（从 1 开始）")]
    pub(super) page: i64,
    #[arg(long = "limit", default_value_t = 10, help = "每页数量")]
    pub(super) limit: i64,
    #[arg(long = "format", help = "模板输出，例如 '{{.id}} {{.inline.en}}'")]
    pub(super) format: Option<String>,
    #[arg(
        long = "format-preset",
        conflicts_with = "format",
        help = "使用配置文件 [cli.format.presets] 中的模板名称"
    )]
    pub(super) format_preset: Option<String>,
    #[arg(
        long = "image-ascii",
        default_value_t = false,
        conflicts_with = "image_view",
        help = "模板中 $image 的输出模式使用 ascii"
    )]
    pub(super) image_ascii: bool,
    #[arg(
        long = "image-view",
        default_value_t = false,
        conflicts_with = "image_ascii",
        help = "模板中 $image 的输出模式使用 view（终端直出优先）"
    )]
    pub(super) image_view: bool,
}

#[derive(clap::Args)]
#[command(after_help = r#"示例:
  quote create --inline en "hello" --inline zh "你好"
  quote create --external en ./en.txt --markdown zh ./zh.md
  quote create --image ./a.png --image ./b.jpg --remark "demo""#)]
pub(super) struct CreateArgs {
    #[arg(
        long = "inline",
        value_names = ["LANG", "TEXT"],
        num_args = 2,
        help = "内联文本（可重复），例如 --inline en \"hello\""
    )]
    pub(super) inline: Vec<String>,
    #[arg(
        long = "external",
        value_names = ["LANG", "FILE"],
        num_args = 2,
        help = "外部文本文件（可重复），例如 --external en ./en.txt"
    )]
    pub(super) external: Vec<String>,
    #[arg(
        long = "markdown",
        value_names = ["LANG", "FILE"],
        num_args = 2,
        help = "Markdown 文件（可重复），例如 --markdown zh ./zh.md"
    )]
    pub(super) markdown: Vec<String>,
    #[arg(long = "image", help = "图片文件路径（可重复）")]
    pub(super) image: Vec<PathBuf>,
    #[arg(long = "remark", help = "备注")]
    pub(super) remark: Option<String>,
}

#[derive(clap::Args)]
#[command(after_help = r#"示例:
  quote update --id 1 --inline en "hello" -y
  quote update --id 1 --markdown zh ./zh.md --image ./a.png -y
  quote update --id 1 --remark "new" -y
  quote update --id 1 --clear-remark -y"#)]
pub(super) struct UpdateArgs {
    #[arg(long = "id", help = "目标 quote id")]
    pub(super) id: i64,
    #[arg(
        long = "inline",
        value_names = ["LANG", "TEXT"],
        num_args = 2,
        help = "按语言更新内联文本（可重复）"
    )]
    pub(super) inline: Vec<String>,
    #[arg(
        long = "external",
        value_names = ["LANG", "FILE"],
        num_args = 2,
        help = "按语言更新 external 文件（可重复）"
    )]
    pub(super) external: Vec<String>,
    #[arg(
        long = "markdown",
        value_names = ["LANG", "FILE"],
        num_args = 2,
        help = "按语言更新 markdown 文件（可重复）"
    )]
    pub(super) markdown: Vec<String>,
    #[arg(long = "image", help = "追加图片（可重复）")]
    pub(super) image: Vec<PathBuf>,
    #[arg(long = "remark", conflicts_with = "clear_remark", help = "设置 remark")]
    pub(super) remark: Option<String>,
    #[arg(long = "clear-remark", default_value_t = false, help = "清空 remark")]
    pub(super) clear_remark: bool,
    #[arg(
        long = "yes",
        short = 'y',
        default_value_t = false,
        help = "跳过二次确认"
    )]
    pub(super) yes: bool,
}

#[derive(clap::Args)]
#[command(after_help = r#"示例:
  quote delete --id 1 -y
  quote delete --id 1 --markdown zh -y
  quote delete --id 1 --all-inline -y
  quote delete --id 1 --image-key object/key.png -y
  quote delete --id 1 --image-index 0 -y
  quote delete --id 1 --all-image -y"#)]
pub(super) struct DeleteArgs {
    #[arg(long = "id", help = "目标 quote id")]
    pub(super) id: i64,
    #[arg(long = "inline", help = "删除指定 inline 语言（可重复）")]
    pub(super) inline: Vec<String>,
    #[arg(long = "all-inline", default_value_t = false, help = "删除所有 inline")]
    pub(super) all_inline: bool,
    #[arg(long = "external", help = "删除指定 external 语言（可重复）")]
    pub(super) external: Vec<String>,
    #[arg(
        long = "all-external",
        default_value_t = false,
        help = "删除所有 external"
    )]
    pub(super) all_external: bool,
    #[arg(long = "markdown", help = "删除指定 markdown 语言（可重复）")]
    pub(super) markdown: Vec<String>,
    #[arg(
        long = "all-markdown",
        default_value_t = false,
        help = "删除所有 markdown"
    )]
    pub(super) all_markdown: bool,
    #[arg(
        long = "image-key",
        conflicts_with = "image_index",
        help = "按对象 key 删除图片（可重复）"
    )]
    pub(super) image_key: Vec<String>,
    #[arg(
        long = "image-index",
        conflicts_with = "image_key",
        help = "按图片索引删除（可重复）"
    )]
    pub(super) image_index: Vec<usize>,
    #[arg(long = "all-image", default_value_t = false, help = "删除所有图片")]
    pub(super) all_image: bool,
    #[arg(
        long = "yes",
        short = 'y',
        default_value_t = false,
        help = "跳过二次确认"
    )]
    pub(super) yes: bool,
}

#[derive(clap::Args)]
#[command(after_help = r#"示例:
  quote download --id 1 --external en --out ./en.txt
  quote download --id 1 --markdown zh --out ./zh.md
  quote download --id 1 --image 0 --out ./0.bin"#)]
pub(super) struct DownloadArgs {
    #[arg(long = "id", help = "目标 quote id")]
    pub(super) id: i64,
    #[arg(
        long = "external",
        help = "下载 external 指定语言对象（当前仅支持单个）"
    )]
    pub(super) external: Option<String>,
    #[arg(
        long = "markdown",
        help = "下载 markdown 指定语言对象（当前仅支持单个）"
    )]
    pub(super) markdown: Option<String>,
    #[arg(long = "image", help = "下载 image 指定索引对象（当前仅支持单个）")]
    pub(super) image: Option<usize>,
    #[arg(long = "out", help = "输出文件路径")]
    pub(super) out: PathBuf,
}
