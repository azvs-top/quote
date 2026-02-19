use crate::adapter::cli::UpdateArgs;
use crate::adapter::cli::confirm::confirm_yes;
use crate::application::service::quote::{QuoteUpdateDraft, UpdateQuoteService};
use crate::application::storage::StoragePayload;
use crate::application::ApplicationState;
use crate::domain::value::Lang;
use std::path::PathBuf;

pub(super) async fn handle_update(state: &ApplicationState, args: UpdateArgs) -> anyhow::Result<()> {
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
