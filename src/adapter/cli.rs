use std::collections::HashMap;
use std::path::PathBuf;
use crate::app::AppState;
use crate::quote::{GetQuoteById, GetQuoteRandom, ListQuotes, QuoteQuery, QuoteQueryFilter};
use clap::{Parser, Subcommand};

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
}

#[derive(Debug, clap::Args)]
struct GetQuoteArgs {
    #[arg(long = "id")]
    pub id: Option<i64>,
    #[arg(long = "page")]
    pub page: Option<usize>,
    #[arg(long = "limit")]
    pub limit: Option<usize>,
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

pub async fn run(state: AppState) -> anyhow::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Get(GetQuoteArgs { id, page, limit }) => {
            if let Some(id) = id {
                // 情况一：存在 id --> 按 id 获取
                // NOTE: 允许返回任何大模块的内容，以JSON形式呈现
                let quote = GetQuoteById::new(state.quote_port.as_ref())
                    .execute(*id)
                    .await?;
                println!("{}", serde_json::to_string_pretty(&quote)?);
                return Ok(());
            } else if let Some(page) = page {
                // 情况二：不存在 id 但存在 page --> 分页获取列表
                // NOTE: 允许返回任何大模块的内容，以JSON形式呈现
                let offset = limit.map(|v| (page - 1) * v);
                let query = QuoteQuery::builder()
                    .with_limit(limit.map(|v| v as i64))
                    .with_offset(offset.map(|v| v as i64))
                    .build();
                let quotes = ListQuotes::new(state.quote_port.as_ref())
                    .execute(query)
                    .await?;
                println!("{}", serde_json::to_string_pretty(&quotes)?);
                return Ok(());
            } else {
                // 情况三：不存在 id 和 page --> 随机获取一条
                // NOTE: 随机获取一条数据，在 CLI 下一定获取的是 inline的内容
                let mut builder = QuoteQuery::builder();

                if !state.config.quote.default_langs.is_empty() {
                    builder = builder.filter(QuoteQueryFilter::HasInlineAllLang(
                        state.config.quote.default_langs.clone(),
                    ));
                }
                let query = builder.build();

                let quote = GetQuoteRandom::new(state.quote_port.as_ref())
                    .execute(query)
                    .await?;
                let texts = quote.get_inline_texts_by_langs(&*state.config.quote.default_langs)?;
                for text in texts {
                    println!("{}", text);
                }
            }
        }

        Commands::Add(args) => handle_add(state, args).await?,
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