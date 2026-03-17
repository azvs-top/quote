use super::{MultiLangObject, MultiLangText, Quote, QuoteDraft, QuotePatch};
use crate::domain::value::{Lang, ObjectKey};

fn lang(code: &str) -> Lang {
    Lang::new(code).expect("valid lang")
}

fn object_key(path: &str) -> ObjectKey {
    ObjectKey::new(path.to_string()).expect("valid key")
}

#[test]
fn quote_draft_rejects_missing_content() {
    let draft = QuoteDraft::new(
        MultiLangText::new(),
        MultiLangObject::new(),
        MultiLangObject::new(),
        Vec::new(),
        None,
    );

    assert!(draft.is_err());
}

#[test]
fn apply_patch_revalidates_quote() {
    let mut inline = MultiLangText::new();
    inline.insert(lang("en"), "hello".to_string());
    let quote = Quote::new(1, inline, MultiLangObject::new(), MultiLangObject::new(), vec![], None)
        .expect("create quote");

    let patch = QuotePatch::new(
        None,
        true,
        Vec::new(),
        None,
        false,
        Vec::new(),
        None,
        false,
        Vec::new(),
        None,
        false,
        Vec::new(),
        None,
    )
    .expect("create patch");
    let updated = quote.apply(patch);

    assert!(updated.is_err());
}

#[test]
fn apply_patch_updates_external_content() {
    let mut inline = MultiLangText::new();
    inline.insert(lang("en"), "hello".to_string());
    let quote = Quote::new(1, inline, MultiLangObject::new(), MultiLangObject::new(), vec![], None)
        .expect("create quote");

    let mut external = MultiLangObject::new();
    external.insert(lang("zh"), object_key("text/zh/file.txt"));
    let patch = QuotePatch::new(
        None,
        false,
        Vec::new(),
        Some(external),
        false,
        Vec::new(),
        None,
        false,
        Vec::new(),
        None,
        false,
        Vec::new(),
        Some(Some("remark".to_string())),
    )
    .expect("create patch");
    let updated = quote.apply(patch).expect("apply patch");

    assert_eq!(updated.external().len(), 1);
    assert_eq!(updated.remark(), Some("remark"));
}
