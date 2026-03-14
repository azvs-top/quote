use crate::adapter::cli::GetArgs;
use crate::adapter::cli::format::{resolve_effective_format, resolve_image_mode};
use crate::adapter::cli::output::print_quote;
use crate::application::service::quote::{GetQuoteByIdService, GetRandomQuoteService};
use crate::application::service::template::{
    BuildQuoteTemplateFilterService, RenderQuoteTemplateService,
};
use crate::application::{ApplicationState, CliImageMode};

pub(super) async fn handle_get(state: &ApplicationState, args: GetArgs) -> anyhow::Result<()> {
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
        CliImageMode::from(cli_cfg.get_image_mode),
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
