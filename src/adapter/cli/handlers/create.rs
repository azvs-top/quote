use crate::adapter::cli::CreateArgs;
use crate::application::service::quote::{CreateQuoteService, QuoteCreateDraft};
use crate::application::storage::StoragePayload;
use crate::application::ApplicationState;
use crate::domain::value::Lang;
use std::path::PathBuf;

pub(super) async fn handle_create(state: &ApplicationState, args: CreateArgs) -> anyhow::Result<()> {
    let mut draft = QuoteCreateDraft::default();
    draft.remark = args.remark;

    if args.inline.len() % 2 != 0 {
        anyhow::bail!("--inline expects pairs: LANG TEXT");
    }
    for chunk in args.inline.chunks(2) {
        let lang = Lang::new(chunk[0].clone())?;
        if draft.inline.insert(lang, chunk[1].clone()).is_some() {
            anyhow::bail!("duplicate inline lang: {}", chunk[0]);
        }
    }

    if args.external.len() % 2 != 0 {
        anyhow::bail!("--external expects pairs: LANG FILE");
    }
    for chunk in args.external.chunks(2) {
        let lang = Lang::new(chunk[0].clone())?;
        let path = PathBuf::from(&chunk[1]);
        let bytes = tokio::fs::read(&path).await?;
        let payload = StoragePayload {
            filename: path.file_name().map(|v| v.to_string_lossy().to_string()),
            bytes,
        };
        if draft.external.insert(lang, payload).is_some() {
            anyhow::bail!("duplicate external lang: {}", chunk[0]);
        }
    }

    if args.markdown.len() % 2 != 0 {
        anyhow::bail!("--markdown expects pairs: LANG FILE");
    }
    for chunk in args.markdown.chunks(2) {
        let lang = Lang::new(chunk[0].clone())?;
        let path = PathBuf::from(&chunk[1]);
        let bytes = tokio::fs::read(&path).await?;
        let payload = StoragePayload {
            filename: path.file_name().map(|v| v.to_string_lossy().to_string()),
            bytes,
        };
        if draft.markdown.insert(lang, payload).is_some() {
            anyhow::bail!("duplicate markdown lang: {}", chunk[0]);
        }
    }

    for path in args.image {
        let bytes = tokio::fs::read(&path).await?;
        draft.image.push(StoragePayload {
            filename: path.file_name().map(|v| v.to_string_lossy().to_string()),
            bytes,
        });
    }

    let service = CreateQuoteService::new(state.quote_port.as_ref(), state.storage_port.as_ref());
    let quote = service.execute(draft).await?;
    println!("{}", serde_json::to_string_pretty(&quote)?);
    Ok(())
}
