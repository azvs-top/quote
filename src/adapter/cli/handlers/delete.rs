use crate::adapter::cli::DeleteArgs;
use crate::adapter::cli::confirm::confirm_yes;
use crate::application::service::quote::{
    DeleteQuoteService, PartialDeleteQuoteDraft, PartialDeleteQuoteService,
};
use crate::application::ApplicationState;
use crate::domain::value::{Lang, ObjectKey};

pub(super) async fn handle_delete(state: &ApplicationState, args: DeleteArgs) -> anyhow::Result<()> {
    let has_partial = args.all_inline
        || !args.inline.is_empty()
        || args.all_external
        || !args.external.is_empty()
        || args.all_markdown
        || !args.markdown.is_empty()
        || args.all_image
        || !args.image_key.is_empty()
        || !args.image_index.is_empty();

    let prompt = if has_partial {
        format!("partial delete quote id={} ?", args.id)
    } else {
        format!("delete quote id={} ?", args.id)
    };
    if !args.yes && !confirm_yes(&prompt)? {
        println!("aborted");
        return Ok(());
    }

    if has_partial {
        let mut draft = PartialDeleteQuoteDraft {
            id: args.id,
            clear_inline: args.all_inline,
            clear_external: args.all_external,
            clear_markdown: args.all_markdown,
            clear_image: args.all_image,
            ..Default::default()
        };

        for lang in args.inline {
            draft.inline_langs.push(Lang::new(lang)?);
        }
        for lang in args.external {
            draft.external_langs.push(Lang::new(lang)?);
        }
        for lang in args.markdown {
            draft.markdown_langs.push(Lang::new(lang)?);
        }
        for key in args.image_key {
            draft.image_keys.push(ObjectKey::new(key)?);
        }
        draft.image_indexes = args.image_index;

        let service =
            PartialDeleteQuoteService::new(state.quote_port.as_ref(), state.storage_port.as_ref());
        let quote = service.execute(draft).await?;
        println!("{}", serde_json::to_string_pretty(&quote)?);
        return Ok(());
    }

    let service = DeleteQuoteService::new(state.quote_port.as_ref(), state.storage_port.as_ref());
    service.execute(args.id).await?;
    println!("deleted quote id={}", args.id);
    Ok(())
}
