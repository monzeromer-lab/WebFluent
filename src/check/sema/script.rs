//! A script's names as the person calling them reads them.

/// How a name a script declares reads to the person calling it:
/// `money(n: Number, currency?: String) -> String`, its types as the
/// checker reads its JSDoc.
pub fn signature(n: &crate::syntax::js_scan::Name) -> String {
    use crate::check::sema::types::jsdoc_type;
    let doc = n
        .doc
        .as_deref()
        .map(crate::check::jsdoc::parse)
        .unwrap_or_default();
    let params = |params: &[crate::syntax::js_scan::Param]| {
        params
            .iter()
            .map(|p| {
                let documented = doc.param(&p.name);
                let ty = documented
                    .and_then(|d| d.ty.as_deref())
                    .map(|t| jsdoc_type(t).to_string())
                    .unwrap_or_else(|| "Any".to_string());
                let optional = p.optional || documented.is_some_and(|d| d.optional);
                format!(
                    "{}{}{}: {ty}",
                    if p.rest { "..." } else { "" },
                    p.name,
                    if optional { "?" } else { "" }
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let returns = doc
        .returns
        .as_deref()
        .map(|t| format!(" -> {}", jsdoc_type(t)))
        .unwrap_or_default();
    match &n.kind {
        crate::syntax::js_scan::NameKind::Function {
            params: p,
            is_async,
        } => format!(
            "{}{}({}){returns}",
            if *is_async { "async " } else { "" },
            n.name,
            params(p)
        ),
        crate::syntax::js_scan::NameKind::Class { params: p, .. } => {
            format!("class {}({})", n.name, params(p))
        }
        crate::syntax::js_scan::NameKind::Value => n.name.clone(),
    }
}
