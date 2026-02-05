use azvs_quote::adapter::cli;
use azvs_quote::app::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState::new().await?;
    cli::run(state).await
}