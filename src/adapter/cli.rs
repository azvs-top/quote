use crate::app::AppState;
use crate::quote::GetQuoteById;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "quote", version = "0.1.0", about = "Quote CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Get(GetQuoteArgs),
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

pub async fn run_cli(state: AppState) -> anyhow::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Get(GetQuoteArgs { id, page, limit }) => {
            match id {
                Some(id) => {
                    // 按 id 获取一条 quote
                    let quote = GetQuoteById::new(state.quote_port.as_ref())
                        .execute(*id)
                        .await?;
                    println!("{:#?}", quote);
                }
                None => {
                    todo!()
                }
            }
        }
    }
    Ok(())
}
