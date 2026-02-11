use crate::app::AppState;
use crate::dict::{DictQuery, ListType};
use crate::quote::{GetQuoteById, GetQuoteRandom, ListQuotes, QuoteQuery, QuoteQueryFilter};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;

#[derive(Default, Debug)]
struct LangDraft {
    inline: Option<String>,
    file: Option<PathBuf>,
    md: Option<PathBuf>,
}
type Draft = HashMap<String, LangDraft>;

#[derive(Parser)]
#[command(name = "quote", version = "0.1.0", about = "Quote CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Get(GetQuoteArgs),
    Add(AddQuoteArgs),
    Dict(DictArgs),
    DictItem(DictItemArgs),
}

#[derive(Debug, clap::Args)]
struct GetQuoteArgs {
    #[arg(long = "id")]
    pub id: Option<i64>,
    #[arg(long = "page")]
    pub page: Option<usize>,
    #[arg(long = "limit")]
    pub limit: Option<usize>,
    #[arg(long = "active")]
    pub active: Option<bool>,
}

#[derive(Debug, clap::Args)]
struct AddQuoteArgs {
    #[arg(long = "lang", value_names = ["LANG", "TEXT"], num_args = 2)]
    pub lang: Vec<String>,

    #[arg(long = "file", value_names = ["LANG", "FILE"], num_args = 2)]
    pub file: Vec<String>,

    #[arg(long = "md", value_names = ["LANG", "FILE"], num_args = 2)]
    pub md: Vec<String>,

    #[arg(long = "image")]
    pub image: Option<PathBuf>,
}

#[derive(Debug, clap::Args)]
struct DictArgs {
    #[command(subcommand)]
    command: DictCommands,
}

#[derive(Debug, Subcommand)]
enum DictCommands {
    Get(GetDictArgs),
}

#[derive(Debug, clap::Args)]
struct GetDictArgs {
    #[arg(long = "active")]
    pub active: Option<bool>,
    #[arg(long = "page")]
    pub page: Option<usize>,
    #[arg(long = "limit")]
    pub limit: Option<usize>,
    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, clap::Args)]
struct DictItemArgs {
    #[command(subcommand)]
    command: DictItemCommands,
}

#[derive(Debug, Subcommand)]
enum DictItemCommands {
    Get(GetDictItemArgs),
}

#[derive(Debug, clap::Args)]
struct GetDictItemArgs {
    #[arg(value_name = "TYPE")]
    pub type_key: String,
    #[arg(long = "active")]
    pub active: Option<bool>,
    #[arg(long = "page")]
    pub page: Option<usize>,
    #[arg(long = "limit")]
    pub limit: Option<usize>,
    #[arg(long = "json")]
    pub json: bool,
}

// #################################################################################################
// ########## 代码逻辑区域
// #################################################################################################

pub async fn run(state: AppState) -> anyhow::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Get(args) => handle_get(state, args).await?,
        Commands::Add(args) => handle_add(state, args).await?,
        Commands::Dict(args) => handle_dict(state, args).await?,
        Commands::DictItem(args) => handle_dict_item(state, args).await?,
    }
    Ok(())
}

async fn handle_get(state: AppState, args: &GetQuoteArgs) -> anyhow::Result<()> {
    if let Some(id) = args.id {
        // 情况一：存在 id --> 按 id 获取
        // NOTE: 允许返回任何大模块的内容，以JSON形式呈现
        let quote = GetQuoteById::new(state.quote_port.as_ref())
            .execute(id)
            .await?;
        println!("{}", serde_json::to_string_pretty(&quote)?);
        return Ok(());
    } else if let Some(page) = args.page {
        // 情况二：不存在 id 但存在 page --> 分页获取列表
        // NOTE: 允许返回任何大模块的内容，以JSON形式呈现
        let offset = args.limit.map(|v| (page - 1) * v);
        let query = QuoteQuery::builder()
            .with_active(args.active)
            .with_limit(args.limit.map(|v| v as i64))
            .with_offset(offset.map(|v| v as i64))
            .build();
        let quotes = ListQuotes::new(state.quote_port.as_ref())
            .execute(query)
            .await?;
        println!("{}", serde_json::to_string_pretty(&quotes)?);
        return Ok(());
    }

    // 情况三：不存在 id 和 page --> 随机获取一条
    // NOTE: 随机获取一条数据，在 CLI 下一定获取的是 inline的内容
    let mut builder = QuoteQuery::builder().active(true);

    if !state.config.quote.inline_langs.is_empty() {
        builder = builder.filter(QuoteQueryFilter::HasInlineAllLang(
            state.config.quote.inline_langs.clone(),
        ));
    }
    let query = builder.build();

    let quote = GetQuoteRandom::new(state.quote_port.as_ref())
        .execute(query)
        .await?;
    let texts = quote.get_inline_texts_by_langs(&state.config.quote.inline_langs)?;
    for text in texts {
        println!("{}", text);
    }

    Ok(())
}

async fn handle_add(state: AppState, args: &AddQuoteArgs) -> anyhow::Result<()> {
    let mut draft: Draft = HashMap::new();

    // --lang en "xxx"
    for chunk in args.lang.chunks(2) {
        let lang = chunk[0].clone();
        let text = chunk[1].clone();

        let entry = draft.entry(lang.clone()).or_default();
        if entry.inline.is_some() {
            anyhow::bail!("Duplicate inline text for lang: {}", lang);
        }
        entry.inline = Some(text);
    }

    // --file en en.txt
    for chunk in args.file.chunks(2) {
        let lang = chunk[0].clone();
        let file = PathBuf::from(&chunk[1]);

        let entry = draft.entry(lang.clone()).or_default();
        if entry.file.is_some() {
            anyhow::bail!("Duplicate file for lang: {}", lang);
        }
        entry.file = Some(file);
    }

    // --md en en.md
    for chunk in args.md.chunks(2) {
        let lang = chunk[0].clone();
        let file = PathBuf::from(&chunk[1]);

        let entry = draft.entry(lang.clone()).or_default();
        if entry.md.is_some() {
            anyhow::bail!("Duplicate markdown for lang: {}", lang);
        }
        entry.md = Some(file);
    }

    if draft.is_empty() {
        anyhow::bail!("No inputs to add");
    }

    // todo!("交付给add_quote_usecase(state, draft).await?;");
    println!("Draft collected:\n{:#?}", draft);
    println!("AddQuote validated, but persistence not implemented yet.");

    Ok(())
}

async fn handle_dict(state: AppState, args: &DictArgs) -> anyhow::Result<()> {
    match &args.command {
        DictCommands::Get(get_args) => {
            let limit = get_args.limit.map(|v| v as i64);
            let offset = get_args
                .page
                .map(|page| (page.saturating_sub(1) * get_args.limit.unwrap_or(10)) as i64);

            let query = DictQuery::builder()
                .with_type_active(get_args.active)
                .with_limit(limit)
                .with_offset(offset)
                .langs(vec![state.config.quote.system_lang.clone()])
                .build();

            let types = ListType::new(state.dict_port.as_ref())
                .execute(query)
                .await?;

            if get_args.json {
                println!("{}", serde_json::to_string_pretty(&types)?);
                return Ok(());
            }

            print_table(
                &["TYPE_KEY", "TYPE_NAME"],
                &types,
                &[
                    Box::new(|row| row.type_key.clone()),
                    Box::new(|row| row.type_name.clone().unwrap_or_default()),
                ],
            )?;
        }
    }
    Ok(())
}

async fn handle_dict_item(state: AppState, args: &DictItemArgs) -> anyhow::Result<()> {
    match &args.command {
        DictItemCommands::Get(get_args) => {
            let limit = get_args.limit.map(|v| v as i64);
            let offset = get_args
                .page
                .map(|page| (page.saturating_sub(1) * get_args.limit.unwrap_or(10)) as i64);

            // 先校验 type 是否存在，避免错误 type 返回空列表造成误解。
            state
                .dict_port
                .get_type(
                    DictQuery::builder()
                        .type_key(get_args.type_key.clone())
                        .langs(vec![state.config.quote.system_lang.clone()])
                        .build(),
                )
                .await?;

            let query = DictQuery::builder()
                .type_key(get_args.type_key.clone())
                .with_item_active(get_args.active)
                .with_limit(limit)
                .with_offset(offset)
                .langs(vec![state.config.quote.system_lang.clone()])
                .build();

            let items = state.dict_port.list_item(query).await?;

            if get_args.json {
                println!("{}", serde_json::to_string_pretty(&items)?);
                return Ok(());
            }

            print_table(
                &["ITEM_KEY", "ITEM_VALUE", "ACTIVE"],
                &items,
                &[
                    Box::new(|row| {
                        if row.is_default {
                            format!("{} (default)", row.item_key)
                        } else {
                            row.item_key.clone()
                        }
                    }),
                    Box::new(|row| row.item_value.clone().unwrap_or_default()),
                    Box::new(|row| row.item_active.to_string()),
                ],
            )?;
        }
    }
    Ok(())
}

/// 以对齐表格形式输出任意行数据。
///
/// # Parameters
/// - `headers`: 列表头，顺序即输出顺序。
/// - `rows`: 行数据集合。
/// - `extractors`: 列提取器；每个提取器把一行映射为该列字符串值。
///
/// # Behavior
/// - `headers.len()` 必须等于 `extractors.len()`，否则返回错误。
/// - 列宽按 Unicode 显示宽度计算，保证中英文混排对齐。
/// - 输出格式为“表头一行 + 数据多行”，列间使用两个空格分隔。
fn print_table<T>(
    headers: &[&str],
    rows: &[T],
    extractors: &[Box<dyn Fn(&T) -> String>],
) -> anyhow::Result<()> {
    if headers.len() != extractors.len() {
        anyhow::bail!("headers/extractors length mismatch");
    }

    let mut widths: Vec<usize> = headers
        .iter()
        .map(|h| UnicodeWidthStr::width(*h))
        .collect();
    let pad_cell = |value: &str, width: usize| -> String {
        let cell_width = UnicodeWidthStr::width(value);
        if cell_width >= width {
            return value.to_string();
        }
        format!("{value}{}", " ".repeat(width - cell_width))
    };

    let matrix: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            extractors
                .iter()
                .enumerate()
                .map(|(i, extractor)| {
                    let value = extractor(row);
                    let w = UnicodeWidthStr::width(value.as_str());
                    widths[i] = widths[i].max(w);
                    value
                })
                .collect()
        })
        .collect();

    for (i, header) in headers.iter().enumerate() {
        print!("{}", pad_cell(header, widths[i]));
        if i < headers.len() - 1 {
            print!("  ");
        }
    }
    println!();

    for row in &matrix {
        for (i, value) in row.iter().enumerate() {
            print!("{}", pad_cell(value, widths[i]));
            if i < row.len() - 1 {
                print!("  ");
            }
        }
        println!();
    }

    Ok(())
}
