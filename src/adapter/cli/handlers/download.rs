use crate::adapter::cli::DownloadArgs;
use crate::application::ApplicationState;
use crate::application::service::quote::GetQuoteByIdService;
use crate::domain::value::{Lang, ObjectKey};

#[derive(Debug, Clone)]
enum DownloadTarget {
    External(Lang),
    Markdown(Lang),
    Image(usize),
}

pub(super) async fn handle_download(
    state: &ApplicationState,
    args: DownloadArgs,
) -> anyhow::Result<()> {
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
    let selected_count = usize::from(external.is_some())
        + usize::from(markdown.is_some())
        + usize::from(image.is_some());
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

#[cfg(test)]
mod tests {
    use super::{DownloadTarget, parse_download_target, resolve_download_key};
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

    #[test]
    fn parse_download_target_rejects_none() {
        let result = parse_download_target(None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn parse_download_target_rejects_multiple() {
        let result = parse_download_target(Some("en"), Some("zh"), None);
        assert!(result.is_err());
    }

    #[test]
    fn parse_download_target_accepts_external() {
        let result = parse_download_target(Some("en"), None, None).expect("should parse");
        assert!(matches!(result, DownloadTarget::External(_)));
    }

    #[test]
    fn parse_download_target_accepts_markdown() {
        let result = parse_download_target(None, Some("zh"), None).expect("should parse");
        assert!(matches!(result, DownloadTarget::Markdown(_)));
    }

    #[test]
    fn parse_download_target_accepts_image() {
        let result = parse_download_target(None, None, Some(0)).expect("should parse");
        assert!(matches!(result, DownloadTarget::Image(0)));
    }

    #[test]
    fn resolve_download_key_for_external() {
        let quote = build_test_quote();
        let target = DownloadTarget::External(Lang::new("en").expect("valid"));
        let key = resolve_download_key(&quote, &target).expect("should resolve");
        assert_eq!(key.as_str(), "text/en/ext");
    }

    #[test]
    fn resolve_download_key_for_markdown() {
        let quote = build_test_quote();
        let target = DownloadTarget::Markdown(Lang::new("zh").expect("valid"));
        let key = resolve_download_key(&quote, &target).expect("should resolve");
        assert_eq!(key.as_str(), "markdown/zh/doc");
    }

    #[test]
    fn resolve_download_key_for_image() {
        let quote = build_test_quote();
        let target = DownloadTarget::Image(0);
        let key = resolve_download_key(&quote, &target).expect("should resolve");
        assert_eq!(key.as_str(), "image/0");
    }

    #[test]
    fn resolve_download_key_returns_err_when_missing() {
        let quote = build_test_quote();
        let target = DownloadTarget::External(Lang::new("ja").expect("valid"));
        let result = resolve_download_key(&quote, &target);
        assert!(result.is_err());
    }
}
