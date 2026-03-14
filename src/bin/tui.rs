use azvs_quote::adapter::tui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tui::run().await
}
