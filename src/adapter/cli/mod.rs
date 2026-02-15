use crate::application::quote::{QuoteFilter, QuoteQuery};
use crate::application::service::quote::{
    CreateQuoteService, DeleteQuoteService, GetQuoteByIdService, GetRandomQuoteService,
    ListQuoteService, PartialDeleteQuoteDraft, PartialDeleteQuoteService, QuoteCreateDraft,
    QuoteUpdateDraft, UpdateQuoteService,
};
use crate::application::storage::StoragePayload;
use crate::application::ApplicationState;
use crate::domain::value::{Lang, ObjectKey};
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::collections::HashSet;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "quote",
    version,
    about = "Quote 命令行工具",
    long_about = "管理 quote 的命令行工具，支持 get/list/create/update/delete。",
    after_help = r#"示例:
  quote get
  quote get --id 1
  quote get --format '{{.inline.zh}}\n{{.inline.en}}'
  quote list --page 1 --limit 20 --format '{{.id}}\t{{.inline.en}}'
  quote create --inline en "hello" --inline zh "你好" --image ./a.png
  quote update --id 1 --markdown zh ./a.md -y
  quote delete --id 1 -y"#
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
}

#[derive(clap::Args)]
#[command(
    after_help = r#"示例:
  quote get
  quote get --id 3
  quote get --format '{{.inline.zh}}\n{{.inline.en}}'"#
)]
struct GetArgs {
    #[arg(long = "id", help = "按 id 获取；未指定时为随机获取")]
    id: Option<i64>,
    #[arg(
        long = "format",
        help = "模板输出，例如 '{{.inline.zh}}\\n{{.inline.en}}'"
    )]
    format: Option<String>,
}

#[derive(clap::Args)]
#[command(
    after_help = r#"示例:
  quote list
  quote list --page 2 --limit 5
  quote list --format '{{.id}} {{.inline.en}}'"#
)]
struct ListArgs {
    #[arg(long = "page", default_value_t = 1, help = "页码（从 1 开始）")]
    page: i64,
    #[arg(long = "limit", default_value_t = 10, help = "每页数量")]
    limit: i64,
    #[arg(long = "format", help = "模板输出，例如 '{{.id}} {{.inline.en}}'")]
    format: Option<String>,
}

#[derive(clap::Args)]
#[command(
    after_help = r#"示例:
  quote create --inline en "hello" --inline zh "你好"
  quote create --external en ./en.txt --markdown zh ./zh.md
  quote create --image ./a.png --image ./b.jpg --remark "demo""#
)]
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
#[command(
    after_help = r#"示例:
  quote update --id 1 --inline en "hello" -y
  quote update --id 1 --markdown zh ./zh.md --image ./a.png -y
  quote update --id 1 --remark "new" -y
  quote update --id 1 --clear-remark -y"#
)]
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
    #[arg(
        long = "remark",
        conflicts_with = "clear_remark",
        help = "设置 remark"
    )]
    remark: Option<String>,
    #[arg(long = "clear-remark", default_value_t = false, help = "清空 remark")]
    clear_remark: bool,
    #[arg(long = "yes", short = 'y', default_value_t = false, help = "跳过二次确认")]
    yes: bool,
}

#[derive(clap::Args)]
#[command(
    after_help = r#"示例:
  quote delete --id 1 -y
  quote delete --id 1 --markdown zh -y
  quote delete --id 1 --all-inline -y
  quote delete --id 1 --image object/key.png -y
  quote delete --id 1 --all-image -y"#
)]
struct DeleteArgs {
    #[arg(long = "id", help = "目标 quote id")]
    id: i64,
    #[arg(long = "inline", help = "删除指定 inline 语言（可重复）")]
    inline: Vec<String>,
    #[arg(long = "all-inline", default_value_t = false, help = "删除所有 inline")]
    all_inline: bool,
    #[arg(long = "external", help = "删除指定 external 语言（可重复）")]
    external: Vec<String>,
    #[arg(long = "all-external", default_value_t = false, help = "删除所有 external")]
    all_external: bool,
    #[arg(long = "markdown", help = "删除指定 markdown 语言（可重复）")]
    markdown: Vec<String>,
    #[arg(long = "all-markdown", default_value_t = false, help = "删除所有 markdown")]
    all_markdown: bool,
    #[arg(long = "image", help = "按对象 key 删除图片（可重复）")]
    image: Vec<String>,
    #[arg(long = "all-image", default_value_t = false, help = "删除所有图片")]
    all_image: bool,
    #[arg(long = "yes", short = 'y', default_value_t = false, help = "跳过二次确认")]
    yes: bool,
}

pub async fn run(state: ApplicationState) -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Get(args) => handle_get(&state, args).await?,
        Command::List(args) => handle_list(&state, args).await?,
        Command::Create(args) => handle_create(&state, args).await?,
        Command::Update(args) => handle_update(&state, args).await?,
        Command::Delete(args) => handle_delete(&state, args).await?,
    }
    Ok(())
}

async fn handle_get(state: &ApplicationState, args: GetArgs) -> anyhow::Result<()> {
    if let Some(id) = args.id {
        let service = GetQuoteByIdService::new(state.quote_port.as_ref());
        let quote = service.execute(id).await?;
        print_quote(&quote, args.format.as_deref())?;
        return Ok(());
    }

    let filter = if let Some(raw) = args.format.as_deref() {
        let tpl = unescape_template(raw);
        validate_template(&tpl)?;
        build_filter_from_template(&tpl)?
    } else {
        None
    };
    let service = GetRandomQuoteService::new(state.quote_port.as_ref());
    let quote = service.execute(filter).await?;
    print_quote(&quote, args.format.as_deref())?;
    Ok(())
}

async fn handle_list(state: &ApplicationState, args: ListArgs) -> anyhow::Result<()> {
    let page = args.page.max(1);
    let limit = args.limit.max(1);
    let offset = (page - 1) * limit;

    let query = QuoteQuery::builder()
        .with_limit(Some(limit))
        .with_offset(Some(offset))
        .build();
    let service = ListQuoteService::new(state.quote_port.as_ref());
    let quotes = service.execute(query).await?;

    print_quotes(&quotes, args.format.as_deref())?;
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

        let service = PartialDeleteQuoteService::new(state.quote_port.as_ref(), state.storage_port.as_ref());
        let quote = service.execute(draft).await?;
        println!("{}", serde_json::to_string_pretty(&quote)?);
        return Ok(());
    }

    let service = DeleteQuoteService::new(state.quote_port.as_ref(), state.storage_port.as_ref());
    service.execute(args.id).await?;
    println!("deleted quote id={}", args.id);
    Ok(())
}

fn confirm_yes(prompt: &str) -> anyhow::Result<bool> {
    print!("{prompt} type 'yes' to continue: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let value = input.trim().to_ascii_lowercase();
    Ok(value == "yes" || value == "y")
}

fn print_quote(quote: &crate::domain::entity::Quote, format: Option<&str>) -> anyhow::Result<()> {
    if let Some(raw) = format {
        let tpl = unescape_template(raw);
        validate_template(&tpl)?;
        let value = serde_json::to_value(quote)?;
        println!("{}", render_template(&tpl, &value));
    } else {
        println!("{}", serde_json::to_string_pretty(quote)?);
    }
    Ok(())
}

fn print_quotes(quotes: &[crate::domain::entity::Quote], format: Option<&str>) -> anyhow::Result<()> {
    if let Some(raw) = format {
        let tpl = unescape_template(raw);
        validate_template(&tpl)?;
        for quote in quotes {
            let value = serde_json::to_value(quote)?;
            println!("{}", render_template(&tpl, &value));
        }
    } else {
        println!("{}", serde_json::to_string_pretty(quotes)?);
    }
    Ok(())
}

fn validate_template(template: &str) -> anyhow::Result<()> {
    if !template.contains("{{") || !template.contains("}}") {
        anyhow::bail!("--format only accepts template strings like '{{.inline.en}}'");
    }
    Ok(())
}

fn render_template(template: &str, root: &Value) -> String {
    let mut out = String::new();
    let mut cursor = 0usize;

    loop {
        let remain = &template[cursor..];
        let Some(start_rel) = remain.find("{{.") else {
            out.push_str(remain);
            break;
        };
        let start = cursor + start_rel;
        out.push_str(&template[cursor..start]);

        let after_start = start + 3;
        let Some(end_rel) = template[after_start..].find("}}") else {
            out.push_str(&template[start..]);
            break;
        };
        let end = after_start + end_rel;

        let key = template[after_start..end].trim();
        out.push_str(&lookup_template_key(root, key));

        cursor = end + 2;
    }

    out
}

fn extract_template_keys(template: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut cursor = 0usize;

    loop {
        let remain = &template[cursor..];
        let Some(start_rel) = remain.find("{{.") else {
            break;
        };
        let start = cursor + start_rel;
        let after_start = start + 3;
        let Some(end_rel) = template[after_start..].find("}}") else {
            break;
        };
        let end = after_start + end_rel;
        let key = template[after_start..end].trim();
        if !key.is_empty() {
            keys.push(key.to_string());
        }
        cursor = end + 2;
    }

    keys
}

fn build_filter_from_template(template: &str) -> anyhow::Result<Option<QuoteFilter>> {
    let keys = extract_template_keys(template);
    if keys.is_empty() {
        return Ok(None);
    }

    let mut inline_all: HashSet<Lang> = HashSet::new();
    let mut external_all: HashSet<Lang> = HashSet::new();
    let mut markdown_all: HashSet<Lang> = HashSet::new();
    let mut image_exists = false;

    for key in keys {
        let mut parts = key.split('.');
        let Some(head) = parts.next() else {
            continue;
        };
        let second = parts.next();

        match head {
            "inline" => {
                if let Some(lang) = second {
                    inline_all.insert(Lang::new(lang.to_string())?);
                }
            }
            "external" => {
                if let Some(lang) = second {
                    external_all.insert(Lang::new(lang.to_string())?);
                }
            }
            "markdown" => {
                if let Some(lang) = second {
                    markdown_all.insert(Lang::new(lang.to_string())?);
                }
            }
            // `{{.image}}` / `{{.image.0}}` 都要求至少有 image。
            "image" => image_exists = true,
            _ => {}
        }
    }

    if inline_all.is_empty() && external_all.is_empty() && markdown_all.is_empty() && !image_exists
    {
        return Ok(None);
    }

    let mut filter = QuoteFilter::default();
    filter.inline_all = inline_all.into_iter().collect();
    filter.external_all = external_all.into_iter().collect();
    filter.markdown_all = markdown_all.into_iter().collect();
    if image_exists {
        filter.image_exists = Some(true);
    }
    Ok(Some(filter))
}

fn lookup_template_key(root: &Value, key: &str) -> String {
    let mut current = root;
    for segment in key.split('.').filter(|s| !s.is_empty()) {
        match current {
            Value::Object(map) => {
                let Some(next) = map.get(segment) else {
                    return String::new();
                };
                current = next;
            }
            Value::Array(arr) => {
                let Ok(idx) = segment.parse::<usize>() else {
                    return String::new();
                };
                let Some(next) = arr.get(idx) else {
                    return String::new();
                };
                current = next;
            }
            _ => return String::new(),
        }
    }

    match current {
        Value::Null => String::new(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => v.clone(),
        _ => current.to_string(),
    }
}

fn unescape_template(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }

        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }

    out
}
