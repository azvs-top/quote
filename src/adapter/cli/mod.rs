use crate::application::{ApplicationState, CliImageMode};
use crate::application::quote::QuoteQuery;
use crate::application::service::quote::{
    CreateQuoteService, DeleteQuoteService, GetQuoteByIdService, GetRandomQuoteService,
    ListQuoteService, PartialDeleteQuoteDraft, PartialDeleteQuoteService, QuoteCreateDraft,
    QuoteUpdateDraft, UpdateQuoteService,
};
use crate::application::service::template::{
    BuildQuoteTemplateFilterService, RenderQuoteTemplateService, TemplateImageMode,
};
use crate::application::storage::StoragePayload;
use crate::domain::value::{Lang, ObjectKey};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use viuer::{Config as ViuerConfig, print as print_image};

#[derive(Parser)]
#[command(
    name = "quote",
    version,
    about = "Quote 命令行工具",
    long_about = "管理 quote 的命令行工具，支持 get/list/create/update/delete/download。",
    after_help = r#"示例:
  quote get
  quote get --id 1
  quote get --format '{{.inline.zh}}\n{{.inline.en}}'
  quote list --page 1 --limit 20 --format '{{.id}}\t{{.inline.en}}'
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
struct GetArgs {
    #[arg(long = "id", help = "按 id 获取；未指定时为随机获取")]
    id: Option<i64>,
    #[arg(
        long = "format",
        help = "模板输出，例如 '{{.inline.zh}}\\n{{.inline.en}}'"
    )]
    format: Option<String>,
    #[arg(
        long = "format-preset",
        conflicts_with = "format",
        help = "使用配置文件 [cli.format.presets] 中的模板名称"
    )]
    format_preset: Option<String>,
    #[arg(
        long = "image-ascii",
        default_value_t = false,
        conflicts_with = "image_view",
        help = "模板中 $image 的输出模式使用 ascii"
    )]
    image_ascii: bool,
    #[arg(
        long = "image-view",
        default_value_t = false,
        conflicts_with = "image_ascii",
        help = "模板中 $image 的输出模式使用 view（终端直出优先）"
    )]
    image_view: bool,
}

#[derive(clap::Args)]
#[command(after_help = r#"示例:
  quote list
  quote list --page 2 --limit 5
  quote list --format '{{.id}} {{.inline.en}}'"#)]
struct ListArgs {
    #[arg(long = "page", default_value_t = 1, help = "页码（从 1 开始）")]
    page: i64,
    #[arg(long = "limit", default_value_t = 10, help = "每页数量")]
    limit: i64,
    #[arg(long = "format", help = "模板输出，例如 '{{.id}} {{.inline.en}}'")]
    format: Option<String>,
    #[arg(
        long = "format-preset",
        conflicts_with = "format",
        help = "使用配置文件 [cli.format.presets] 中的模板名称"
    )]
    format_preset: Option<String>,
    #[arg(
        long = "image-ascii",
        default_value_t = false,
        conflicts_with = "image_view",
        help = "模板中 $image 的输出模式使用 ascii"
    )]
    image_ascii: bool,
    #[arg(
        long = "image-view",
        default_value_t = false,
        conflicts_with = "image_ascii",
        help = "模板中 $image 的输出模式使用 view（终端直出优先）"
    )]
    image_view: bool,
}

impl From<CliImageMode> for TemplateImageMode {
    fn from(value: CliImageMode) -> Self {
        match value {
            CliImageMode::Meta => TemplateImageMode::Meta,
            CliImageMode::Ascii => TemplateImageMode::Ascii,
            CliImageMode::View => TemplateImageMode::View,
        }
    }
}

/// 将 CLI 层图片参数映射为统一图片渲染模式。
///
/// 规则：
/// - `--image-view` 优先级最高。
/// - 其次是 `--image-ascii`。
/// - 两者都未指定时回退为默认 `meta`。
///
/// 说明：
/// - 该函数属于 adapter 层参数适配，避免将 cli 参数语义泄漏到 application 层。
fn resolve_image_mode(image_ascii: bool, image_view: bool, default_mode: CliImageMode) -> CliImageMode {
    if image_view {
        return CliImageMode::View;
    }
    if image_ascii {
        return CliImageMode::Ascii;
    }
    default_mode
}

fn resolve_effective_format(
    format: Option<&str>,
    preset: Option<&str>,
    default_format: Option<&str>,
    presets: &HashMap<String, String>,
) -> anyhow::Result<Option<String>> {
    if let Some(raw) = format {
        return Ok(Some(raw.to_string()));
    }
    if let Some(preset_name) = preset {
        let value = presets
            .get(preset_name)
            .ok_or_else(|| anyhow::anyhow!("unknown format preset: {preset_name}"))?;
        return Ok(Some(value.clone()));
    }
    Ok(default_format.map(|v| v.to_string()))
}

#[derive(clap::Args)]
#[command(after_help = r#"示例:
  quote create --inline en "hello" --inline zh "你好"
  quote create --external en ./en.txt --markdown zh ./zh.md
  quote create --image ./a.png --image ./b.jpg --remark "demo""#)]
struct CreateArgs {
    #[arg(
        long = "inline",
        value_names = ["LANG", "TEXT"],
        num_args = 2,
        help = "内联文本（可重复），例如 --inline en \"hello\""
    )]
    inline: Vec<String>,
    #[arg(
        long = "external",
        value_names = ["LANG", "FILE"],
        num_args = 2,
        help = "外部文本文件（可重复），例如 --external en ./en.txt"
    )]
    external: Vec<String>,
    #[arg(
        long = "markdown",
        value_names = ["LANG", "FILE"],
        num_args = 2,
        help = "Markdown 文件（可重复），例如 --markdown zh ./zh.md"
    )]
    markdown: Vec<String>,
    #[arg(long = "image", help = "图片文件路径（可重复）")]
    image: Vec<PathBuf>,
    #[arg(long = "remark", help = "备注")]
    remark: Option<String>,
}

#[derive(clap::Args)]
#[command(after_help = r#"示例:
  quote update --id 1 --inline en "hello" -y
  quote update --id 1 --markdown zh ./zh.md --image ./a.png -y
  quote update --id 1 --remark "new" -y
  quote update --id 1 --clear-remark -y"#)]
struct UpdateArgs {
    #[arg(long = "id", help = "目标 quote id")]
    id: i64,
    #[arg(
        long = "inline",
        value_names = ["LANG", "TEXT"],
        num_args = 2,
        help = "按语言更新内联文本（可重复）"
    )]
    inline: Vec<String>,
    #[arg(
        long = "external",
        value_names = ["LANG", "FILE"],
        num_args = 2,
        help = "按语言更新 external 文件（可重复）"
    )]
    external: Vec<String>,
    #[arg(
        long = "markdown",
        value_names = ["LANG", "FILE"],
        num_args = 2,
        help = "按语言更新 markdown 文件（可重复）"
    )]
    markdown: Vec<String>,
    #[arg(long = "image", help = "追加图片（可重复）")]
    image: Vec<PathBuf>,
    #[arg(long = "remark", conflicts_with = "clear_remark", help = "设置 remark")]
    remark: Option<String>,
    #[arg(long = "clear-remark", default_value_t = false, help = "清空 remark")]
    clear_remark: bool,
    #[arg(
        long = "yes",
        short = 'y',
        default_value_t = false,
        help = "跳过二次确认"
    )]
    yes: bool,
}

#[derive(clap::Args)]
#[command(after_help = r#"示例:
  quote delete --id 1 -y
  quote delete --id 1 --markdown zh -y
  quote delete --id 1 --all-inline -y
  quote delete --id 1 --image object/key.png -y
  quote delete --id 1 --all-image -y"#)]
struct DeleteArgs {
    #[arg(long = "id", help = "目标 quote id")]
    id: i64,
    #[arg(long = "inline", help = "删除指定 inline 语言（可重复）")]
    inline: Vec<String>,
    #[arg(long = "all-inline", default_value_t = false, help = "删除所有 inline")]
    all_inline: bool,
    #[arg(long = "external", help = "删除指定 external 语言（可重复）")]
    external: Vec<String>,
    #[arg(
        long = "all-external",
        default_value_t = false,
        help = "删除所有 external"
    )]
    all_external: bool,
    #[arg(long = "markdown", help = "删除指定 markdown 语言（可重复）")]
    markdown: Vec<String>,
    #[arg(
        long = "all-markdown",
        default_value_t = false,
        help = "删除所有 markdown"
    )]
    all_markdown: bool,
    #[arg(long = "image", help = "按对象 key 删除图片（可重复）")]
    image: Vec<String>,
    #[arg(long = "all-image", default_value_t = false, help = "删除所有图片")]
    all_image: bool,
    #[arg(
        long = "yes",
        short = 'y',
        default_value_t = false,
        help = "跳过二次确认"
    )]
    yes: bool,
}

#[derive(clap::Args)]
#[command(after_help = r#"示例:
  quote download --id 1 --external en --out ./en.txt
  quote download --id 1 --markdown zh --out ./zh.md
  quote download --id 1 --image 0 --out ./0.bin"#)]
struct DownloadArgs {
    #[arg(long = "id", help = "目标 quote id")]
    id: i64,
    #[arg(
        long = "external",
        help = "下载 external 指定语言对象（当前仅支持单个）"
    )]
    external: Option<String>,
    #[arg(
        long = "markdown",
        help = "下载 markdown 指定语言对象（当前仅支持单个）"
    )]
    markdown: Option<String>,
    #[arg(long = "image", help = "下载 image 指定索引对象（当前仅支持单个）")]
    image: Option<usize>,
    #[arg(long = "out", help = "输出文件路径")]
    out: PathBuf,
}

#[derive(Debug, Clone)]
enum DownloadTarget {
    External(Lang),
    Markdown(Lang),
    Image(usize),
}

pub async fn run(state: ApplicationState) -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Get(args) => handle_get(&state, args).await?,
        Command::List(args) => handle_list(&state, args).await?,
        Command::Create(args) => handle_create(&state, args).await?,
        Command::Update(args) => handle_update(&state, args).await?,
        Command::Delete(args) => handle_delete(&state, args).await?,
        Command::Download(args) => handle_download(&state, args).await?,
    }
    Ok(())
}

async fn handle_get(state: &ApplicationState, args: GetArgs) -> anyhow::Result<()> {
    let cli_cfg = &state.config.cli.format;
    let effective_format = resolve_effective_format(
        args.format.as_deref(),
        args.format_preset.as_deref(),
        cli_cfg.default_get.as_deref(),
        &cli_cfg.presets,
    )?;
    let image_mode = resolve_image_mode(
        args.image_ascii,
        args.image_view,
        CliImageMode::from(cli_cfg.image_mode),
    );
    let render_template_service =
        RenderQuoteTemplateService::new(state.storage_port.as_ref(), image_mode.into());

    if let Some(id) = args.id {
        let service = GetQuoteByIdService::new(state.quote_port.as_ref());
        let quote = service.execute(id).await?;
        print_quote(
            &quote,
            effective_format.as_deref(),
            &render_template_service,
            image_mode,
        )
        .await?;
        return Ok(());
    }

    let filter = if let Some(raw) = effective_format.as_deref() {
        BuildQuoteTemplateFilterService::execute(raw)?
    } else {
        None
    };
    let service = GetRandomQuoteService::new(state.quote_port.as_ref());
    let quote = service.execute(filter).await?;
    print_quote(
        &quote,
        effective_format.as_deref(),
        &render_template_service,
        image_mode,
    )
    .await?;
    Ok(())
}

async fn handle_list(state: &ApplicationState, args: ListArgs) -> anyhow::Result<()> {
    let cli_cfg = &state.config.cli.format;
    let effective_format = resolve_effective_format(
        args.format.as_deref(),
        args.format_preset.as_deref(),
        cli_cfg.default_list.as_deref(),
        &cli_cfg.presets,
    )?;
    let image_mode = resolve_image_mode(
        args.image_ascii,
        args.image_view,
        CliImageMode::from(cli_cfg.image_mode),
    );
    let render_template_service =
        RenderQuoteTemplateService::new(state.storage_port.as_ref(), image_mode.into());

    let page = args.page.max(1);
    let limit = args.limit.max(1);
    let offset = (page - 1) * limit;

    let query = QuoteQuery::builder()
        .with_limit(Some(limit))
        .with_offset(Some(offset))
        .build();
    let service = ListQuoteService::new(state.quote_port.as_ref());
    let quotes = service.execute(query).await?;

    print_quotes(
        &quotes,
        effective_format.as_deref(),
        &render_template_service,
        image_mode,
    )
    .await?;
    Ok(())
}

async fn handle_create(state: &ApplicationState, args: CreateArgs) -> anyhow::Result<()> {
    let mut draft = QuoteCreateDraft::default();
    draft.remark = args.remark;

    if args.inline.len() % 2 != 0 {
        anyhow::bail!("--inline expects pairs: LANG TEXT");
    }
    for chunk in args.inline.chunks(2) {
        let lang = Lang::new(chunk[0].clone())?;
        if draft.inline.insert(lang, chunk[1].clone()).is_some() {
            anyhow::bail!("duplicate inline lang: {}", chunk[0]);
        }
    }

    if args.external.len() % 2 != 0 {
        anyhow::bail!("--external expects pairs: LANG FILE");
    }
    for chunk in args.external.chunks(2) {
        let lang = Lang::new(chunk[0].clone())?;
        let path = PathBuf::from(&chunk[1]);
        let bytes = tokio::fs::read(&path).await?;
        let payload = StoragePayload {
            filename: path.file_name().map(|v| v.to_string_lossy().to_string()),
            bytes,
        };
        if draft.external.insert(lang, payload).is_some() {
            anyhow::bail!("duplicate external lang: {}", chunk[0]);
        }
    }

    if args.markdown.len() % 2 != 0 {
        anyhow::bail!("--markdown expects pairs: LANG FILE");
    }
    for chunk in args.markdown.chunks(2) {
        let lang = Lang::new(chunk[0].clone())?;
        let path = PathBuf::from(&chunk[1]);
        let bytes = tokio::fs::read(&path).await?;
        let payload = StoragePayload {
            filename: path.file_name().map(|v| v.to_string_lossy().to_string()),
            bytes,
        };
        if draft.markdown.insert(lang, payload).is_some() {
            anyhow::bail!("duplicate markdown lang: {}", chunk[0]);
        }
    }

    for path in args.image {
        let bytes = tokio::fs::read(&path).await?;
        draft.image.push(StoragePayload {
            filename: path.file_name().map(|v| v.to_string_lossy().to_string()),
            bytes,
        });
    }

    let service = CreateQuoteService::new(state.quote_port.as_ref(), state.storage_port.as_ref());
    let quote = service.execute(draft).await?;
    println!("{}", serde_json::to_string_pretty(&quote)?);
    Ok(())
}

async fn handle_update(state: &ApplicationState, args: UpdateArgs) -> anyhow::Result<()> {
    if !args.yes && !confirm_yes(&format!("update quote id={} ?", args.id))? {
        println!("aborted");
        return Ok(());
    }

    let mut draft = QuoteUpdateDraft {
        id: args.id,
        ..Default::default()
    };

    if args.inline.len() % 2 != 0 {
        anyhow::bail!("--inline expects pairs: LANG TEXT");
    }
    if !args.inline.is_empty() {
        let mut inline = crate::domain::entity::MultiLangText::new();
        for chunk in args.inline.chunks(2) {
            let lang = Lang::new(chunk[0].clone())?;
            if inline.insert(lang, chunk[1].clone()).is_some() {
                anyhow::bail!("duplicate inline lang: {}", chunk[0]);
            }
        }
        draft.inline = Some(inline);
    }

    if args.external.len() % 2 != 0 {
        anyhow::bail!("--external expects pairs: LANG FILE");
    }
    if !args.external.is_empty() {
        let mut external = std::collections::HashMap::new();
        for chunk in args.external.chunks(2) {
            let lang = Lang::new(chunk[0].clone())?;
            let path = PathBuf::from(&chunk[1]);
            let bytes = tokio::fs::read(&path).await?;
            let payload = StoragePayload {
                filename: path.file_name().map(|v| v.to_string_lossy().to_string()),
                bytes,
            };
            if external.insert(lang, payload).is_some() {
                anyhow::bail!("duplicate external lang: {}", chunk[0]);
            }
        }
        draft.external = Some(external);
    }

    if args.markdown.len() % 2 != 0 {
        anyhow::bail!("--markdown expects pairs: LANG FILE");
    }
    if !args.markdown.is_empty() {
        let mut markdown = std::collections::HashMap::new();
        for chunk in args.markdown.chunks(2) {
            let lang = Lang::new(chunk[0].clone())?;
            let path = PathBuf::from(&chunk[1]);
            let bytes = tokio::fs::read(&path).await?;
            let payload = StoragePayload {
                filename: path.file_name().map(|v| v.to_string_lossy().to_string()),
                bytes,
            };
            if markdown.insert(lang, payload).is_some() {
                anyhow::bail!("duplicate markdown lang: {}", chunk[0]);
            }
        }
        draft.markdown = Some(markdown);
    }

    if !args.image.is_empty() {
        let mut images = Vec::with_capacity(args.image.len());
        for path in args.image {
            let bytes = tokio::fs::read(&path).await?;
            images.push(StoragePayload {
                filename: path.file_name().map(|v| v.to_string_lossy().to_string()),
                bytes,
            });
        }
        draft.image = Some(images);
    }

    draft.remark = if args.clear_remark {
        Some(None)
    } else {
        args.remark.map(Some)
    };

    let service = UpdateQuoteService::new(state.quote_port.as_ref(), state.storage_port.as_ref());
    let quote = service.execute(draft).await?;
    println!("{}", serde_json::to_string_pretty(&quote)?);
    Ok(())
}

async fn handle_delete(state: &ApplicationState, args: DeleteArgs) -> anyhow::Result<()> {
    let has_partial = args.all_inline
        || !args.inline.is_empty()
        || args.all_external
        || !args.external.is_empty()
        || args.all_markdown
        || !args.markdown.is_empty()
        || args.all_image
        || !args.image.is_empty();

    let prompt = if has_partial {
        format!("partial delete quote id={} ?", args.id)
    } else {
        format!("delete quote id={} ?", args.id)
    };
    if !args.yes && !confirm_yes(&prompt)? {
        println!("aborted");
        return Ok(());
    }

    if has_partial {
        let mut draft = PartialDeleteQuoteDraft {
            id: args.id,
            clear_inline: args.all_inline,
            clear_external: args.all_external,
            clear_markdown: args.all_markdown,
            clear_image: args.all_image,
            ..Default::default()
        };

        for lang in args.inline {
            draft.inline_langs.push(Lang::new(lang)?);
        }
        for lang in args.external {
            draft.external_langs.push(Lang::new(lang)?);
        }
        for lang in args.markdown {
            draft.markdown_langs.push(Lang::new(lang)?);
        }
        for key in args.image {
            draft.image_keys.push(ObjectKey::new(key)?);
        }

        let service =
            PartialDeleteQuoteService::new(state.quote_port.as_ref(), state.storage_port.as_ref());
        let quote = service.execute(draft).await?;
        println!("{}", serde_json::to_string_pretty(&quote)?);
        return Ok(());
    }

    let service = DeleteQuoteService::new(state.quote_port.as_ref(), state.storage_port.as_ref());
    service.execute(args.id).await?;
    println!("deleted quote id={}", args.id);
    Ok(())
}

async fn handle_download(state: &ApplicationState, args: DownloadArgs) -> anyhow::Result<()> {
    let target = parse_download_target(
        args.external.as_deref(),
        args.markdown.as_deref(),
        args.image,
    )?;

    let service = GetQuoteByIdService::new(state.quote_port.as_ref());
    let quote = service.execute(args.id).await?;
    let key = resolve_download_key(&quote, &target)?;

    let bytes = state.storage_port.download(key).await?;

    if let Some(parent) = args.out.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    tokio::fs::write(&args.out, bytes).await?;

    println!("downloaded {} -> {}", key.as_str(), args.out.display());
    Ok(())
}

fn parse_download_target(
    external: Option<&str>,
    markdown: Option<&str>,
    image: Option<usize>,
) -> anyhow::Result<DownloadTarget> {
    let selected_count =
        usize::from(external.is_some()) + usize::from(markdown.is_some()) + usize::from(image.is_some());
    if selected_count != 1 {
        anyhow::bail!("download requires exactly one target: --external, --markdown, or --image");
    }

    if let Some(lang_raw) = external {
        return Ok(DownloadTarget::External(Lang::new(lang_raw.to_string())?));
    }
    if let Some(lang_raw) = markdown {
        return Ok(DownloadTarget::Markdown(Lang::new(lang_raw.to_string())?));
    }
    if let Some(index) = image {
        return Ok(DownloadTarget::Image(index));
    }

    unreachable!("selected_count ensured one target");
}

fn resolve_download_key<'a>(
    quote: &'a crate::domain::entity::Quote,
    target: &DownloadTarget,
) -> anyhow::Result<&'a ObjectKey> {
    match target {
        DownloadTarget::External(lang) => quote
            .external()
            .get(lang)
            .ok_or_else(|| anyhow::anyhow!("external not found for lang={}", lang.as_str())),
        DownloadTarget::Markdown(lang) => quote
            .markdown()
            .get(lang)
            .ok_or_else(|| anyhow::anyhow!("markdown not found for lang={}", lang.as_str())),
        DownloadTarget::Image(index) => quote
            .image()
            .get(*index)
            .ok_or_else(|| anyhow::anyhow!("image not found for index={index}")),
    }
}

/// 在执行高风险操作（update/delete）前进行再次确认。
///
/// 仅当用户输入 `yes` 或 `y`（不区分大小写）时返回 `true`。
fn confirm_yes(prompt: &str) -> anyhow::Result<bool> {
    print!("{prompt} type 'yes' to continue: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let value = input.trim().to_ascii_lowercase();
    Ok(value == "yes" || value == "y")
}

async fn print_quote(
    quote: &crate::domain::entity::Quote,
    format: Option<&str>,
    render_template_service: &RenderQuoteTemplateService<'_>,
    image_mode: CliImageMode,
) -> anyhow::Result<()> {
    if let Some(raw) = format {
        if matches!(image_mode, CliImageMode::View) {
            if let Some(target) = extract_single_image_target(raw) {
                if try_print_image_view(render_template_service, quote, target).await? {
                    return Ok(());
                }
            }
        }
        let rendered = render_template_service.execute(quote, raw).await?;
        println!("{rendered}");
    } else {
        println!("{}", serde_json::to_string_pretty(quote)?);
    }
    Ok(())
}

async fn print_quotes(
    quotes: &[crate::domain::entity::Quote],
    format: Option<&str>,
    render_template_service: &RenderQuoteTemplateService<'_>,
    image_mode: CliImageMode,
) -> anyhow::Result<()> {
    if let Some(raw) = format {
        for quote in quotes {
            if matches!(image_mode, CliImageMode::View) {
                if let Some(target) = extract_single_image_target(raw) {
                    if try_print_image_view(render_template_service, quote, target).await? {
                        continue;
                    }
                }
            }
            let rendered = render_template_service.execute(quote, raw).await?;
            println!("{rendered}");
        }
    } else {
        println!("{}", serde_json::to_string_pretty(quotes)?);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum ImageTemplateTarget {
    Index(usize),
}

/// 解析“仅包含一个模板表达式”的图片目标。
///
/// 支持格式：`{{$image.<index>}}`。
/// 若模板包含额外文本、不是 `$image` 表达式、或索引非法，则返回 `None`。
fn extract_single_image_target(raw_template: &str) -> Option<ImageTemplateTarget> {
    let expr = raw_template.trim();
    if !expr.starts_with("{{") || !expr.ends_with("}}") {
        return None;
    }
    let inner = expr[2..expr.len() - 2].trim();
    let path = inner.strip_prefix('$')?;
    let mut parts = path.split('.').filter(|v| !v.is_empty());
    let head = parts.next()?;
    if head != "image" {
        return None;
    }
    let index_raw = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    index_raw
        .parse::<usize>()
        .ok()
        .map(ImageTemplateTarget::Index)
}

/// 在 `view` 模式下尝试直接向终端输出图片。
///
/// 行为：
/// - 非 TTY 场景直接返回 `Ok(false)`。
/// - 仅处理单张图片目标（由 `extract_single_image_target` 保证）。
/// - 终端直出失败时返回 `Ok(false)`，由上层回退到文本渲染。
async fn try_print_image_view(
    render_template_service: &RenderQuoteTemplateService<'_>,
    quote: &crate::domain::entity::Quote,
    target: ImageTemplateTarget,
) -> anyhow::Result<bool> {
    if !std::io::stdout().is_terminal() {
        return Ok(false);
    }

    let cfg = ViuerConfig {
        transparent: true,
        ..Default::default()
    };

    let mut printed = false;
    match target {
        ImageTemplateTarget::Index(index) => {
            let Some(bytes) = render_template_service
                .load_image_bytes(quote, index)
                .await?
            else {
                return Ok(false);
            };
            let Ok(img) = image::load_from_memory(&bytes) else {
                return Ok(false);
            };
            if print_image(&img, &cfg).is_ok() {
                printed = true;
            }
        }
    }

    Ok(printed)
}

#[cfg(test)]
mod tests {
    use super::{parse_download_target, resolve_download_key, DownloadTarget};
    use crate::domain::entity::{MultiLangObject, MultiLangText, Quote};
    use crate::domain::value::{Lang, ObjectKey};

    fn build_test_quote() -> Quote {
        let mut inline = MultiLangText::new();
        inline.insert(Lang::new("en").expect("valid"), "hello".to_string());

        let mut external = MultiLangObject::new();
        external.insert(
            Lang::new("en").expect("valid"),
            ObjectKey::new("text/en/ext").expect("valid"),
        );

        let mut markdown = MultiLangObject::new();
        markdown.insert(
            Lang::new("zh").expect("valid"),
            ObjectKey::new("markdown/zh/doc").expect("valid"),
        );

        let image = vec![ObjectKey::new("image/0").expect("valid")];

        Quote::new(1, inline, external, markdown, image, None).expect("valid quote")
    }

    /// 校验：未提供任何下载目标时应返回错误。
    #[test]
    fn parse_download_target_rejects_none() {
        let result = parse_download_target(None, None, None);
        assert!(result.is_err());
    }

    /// 校验：同时提供多个下载目标时应返回错误。
    #[test]
    fn parse_download_target_rejects_multiple() {
        let result = parse_download_target(Some("en"), Some("zh"), None);
        assert!(result.is_err());
    }

    /// 校验：external 目标能被正确解析。
    #[test]
    fn parse_download_target_accepts_external() {
        let result = parse_download_target(Some("en"), None, None).expect("should parse");
        assert!(matches!(result, DownloadTarget::External(_)));
    }

    /// 校验：markdown 目标能被正确解析。
    #[test]
    fn parse_download_target_accepts_markdown() {
        let result = parse_download_target(None, Some("zh"), None).expect("should parse");
        assert!(matches!(result, DownloadTarget::Markdown(_)));
    }

    /// 校验：image 索引目标能被正确解析。
    #[test]
    fn parse_download_target_accepts_image() {
        let result = parse_download_target(None, None, Some(0)).expect("should parse");
        assert!(matches!(result, DownloadTarget::Image(0)));
    }

    /// 校验：external 目标可定位到对应对象 key。
    #[test]
    fn resolve_download_key_for_external() {
        let quote = build_test_quote();
        let target = DownloadTarget::External(Lang::new("en").expect("valid"));
        let key = resolve_download_key(&quote, &target).expect("should resolve");
        assert_eq!(key.as_str(), "text/en/ext");
    }

    /// 校验：markdown 目标可定位到对应对象 key。
    #[test]
    fn resolve_download_key_for_markdown() {
        let quote = build_test_quote();
        let target = DownloadTarget::Markdown(Lang::new("zh").expect("valid"));
        let key = resolve_download_key(&quote, &target).expect("should resolve");
        assert_eq!(key.as_str(), "markdown/zh/doc");
    }

    /// 校验：image 目标可按索引定位到对应对象 key。
    #[test]
    fn resolve_download_key_for_image() {
        let quote = build_test_quote();
        let target = DownloadTarget::Image(0);
        let key = resolve_download_key(&quote, &target).expect("should resolve");
        assert_eq!(key.as_str(), "image/0");
    }

    /// 校验：当目标不存在时应返回错误。
    #[test]
    fn resolve_download_key_returns_err_when_missing() {
        let quote = build_test_quote();
        let target = DownloadTarget::External(Lang::new("ja").expect("valid"));
        let result = resolve_download_key(&quote, &target);
        assert!(result.is_err());
    }
}
