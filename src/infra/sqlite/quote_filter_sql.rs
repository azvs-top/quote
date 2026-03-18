use crate::application::ApplicationError;
use crate::domain::quote::QuoteFilter;
use sqlx::{QueryBuilder, Sqlite};

/// 将 `QuoteFilter` 编译为 SQLite SQL 条件片段。
///
/// 约定：
/// - 输出的是可嵌入 `WHERE (...)` 的布尔表达式片段。
/// - 当过滤器为空时输出 `TRUE`，表示不施加约束。
pub fn push_filter_expr(
    qb: &mut QueryBuilder<'_, Sqlite>,
    filter: &QuoteFilter,
) -> Result<(), ApplicationError> {
    if filter.is_empty() {
        qb.push("TRUE");
        return Ok(());
    }

    let mut has_clause = false;

    if !filter.inline_all.is_empty() {
        if has_clause {
            qb.push(" AND ");
        }
        push_lang_all(qb, "inline", &filter.inline_all);
        has_clause = true;
    }
    if !filter.inline_any.is_empty() {
        if has_clause {
            qb.push(" AND ");
        }
        push_lang_any(qb, "inline", &filter.inline_any);
        has_clause = true;
    }
    if !filter.external_all.is_empty() {
        if has_clause {
            qb.push(" AND ");
        }
        push_lang_all(qb, "external", &filter.external_all);
        has_clause = true;
    }
    if !filter.external_any.is_empty() {
        if has_clause {
            qb.push(" AND ");
        }
        push_lang_any(qb, "external", &filter.external_any);
        has_clause = true;
    }
    if !filter.markdown_all.is_empty() {
        if has_clause {
            qb.push(" AND ");
        }
        push_lang_all(qb, "markdown", &filter.markdown_all);
        has_clause = true;
    }
    if !filter.markdown_any.is_empty() {
        if has_clause {
            qb.push(" AND ");
        }
        push_lang_any(qb, "markdown", &filter.markdown_any);
        has_clause = true;
    }
    if let Some(image_exists) = filter.image_exists {
        if has_clause {
            qb.push(" AND ");
        }
        if image_exists {
            qb.push("json_array_length(image) > 0");
        } else {
            qb.push("json_array_length(image) = 0");
        }
        has_clause = true;
    }

    if !filter.all_of.is_empty() {
        if has_clause {
            qb.push(" AND ");
        }
        qb.push("(");
        for (idx, child) in filter.all_of.iter().enumerate() {
            if idx > 0 {
                qb.push(" AND ");
            }
            qb.push("(");
            push_filter_expr(qb, child)?;
            qb.push(")");
        }
        qb.push(")");
        has_clause = true;
    }

    if !filter.any_of.is_empty() {
        if has_clause {
            qb.push(" AND ");
        }
        qb.push("(");
        for (idx, child) in filter.any_of.iter().enumerate() {
            if idx > 0 {
                qb.push(" OR ");
            }
            qb.push("(");
            push_filter_expr(qb, child)?;
            qb.push(")");
        }
        qb.push(")");
        has_clause = true;
    }

    if let Some(not) = &filter.not {
        if has_clause {
            qb.push(" AND ");
        }
        qb.push("NOT (");
        push_filter_expr(qb, not)?;
        qb.push(")");
        has_clause = true;
    }

    if !has_clause {
        qb.push("TRUE");
    }

    Ok(())
}

fn push_lang_all(
    qb: &mut QueryBuilder<'_, Sqlite>,
    col: &str,
    langs: &[crate::domain::value::Lang],
) {
    // SQLite 通过 json_extract 判定指定语言 key 是否存在。
    // all 语义：同一列必须同时包含所有给定语言 key。
    qb.push("(");
    for (idx, lang) in langs.iter().enumerate() {
        if idx > 0 {
            qb.push(" AND ");
        }
        push_lang_exists_expr(qb, col, lang);
    }
    qb.push(")");
}

fn push_lang_any(
    qb: &mut QueryBuilder<'_, Sqlite>,
    col: &str,
    langs: &[crate::domain::value::Lang],
) {
    // any 语义：同一列包含任意一个给定语言 key 即可。
    qb.push("(");
    for (idx, lang) in langs.iter().enumerate() {
        if idx > 0 {
            qb.push(" OR ");
        }
        push_lang_exists_expr(qb, col, lang);
    }
    qb.push(")");
}

fn push_lang_exists_expr(
    qb: &mut QueryBuilder<'_, Sqlite>,
    col: &str,
    lang: &crate::domain::value::Lang,
) {
    // 统一用一个表达式形式，便于未来加表达式索引时保持稳定。
    qb.push("json_extract(");
    qb.push(col);
    qb.push(", '$.");
    qb.push(lang.as_str());
    qb.push("') IS NOT NULL");
}
