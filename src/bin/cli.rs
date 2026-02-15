use azvs_quote::adapter::cli;
use azvs_quote::application::ApplicationState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = ApplicationState::new().await?;
    cli::run(state).await
}
