use crate::adapter::cli::ListArgs;
use crate::adapter::cli::format::{resolve_effective_format, resolve_image_mode};
use crate::adapter::cli::output::print_quotes;
use crate::application::quote::QuoteQuery;
use crate::application::service::quote::ListQuoteService;
use crate::application::service::template::RenderQuoteTemplateService;
use crate::application::{ApplicationState, CliImageMode};

pub(super) async fn handle_list(state: &ApplicationState, args: ListArgs) -> anyhow::Result<()> {
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
