//! Vocabulary lint (studio asks A-1 + A-4): bare words that silently do nothing.
//!
//! The parser discriminates modifiers from arguments purely by vocabulary
//! membership (`is_modifier`), so a misspelled or invented modifier —
//! `Button("Save", huge)` — parses as an *argument expression*, and codegen
//! silently ignores it: not a compile error, not a runtime error, not a visual
//! change. The studio calls this its worst failure mode (finding 0.7): every
//! layer reports success for a change that did not happen.
//!
//! [`lint_vocabulary`] flags every **positional bare-identifier argument that
//! resolves to nothing in scope** — not a state, derived value, prop, action,
//! loop variable, fetch binding, store member, or declared name. Such a word is
//! dead whichever way it was meant: as a modifier it was dropped; as a binding
//! it is an undefined reference at runtime. When the word is close to the
//! modifier vocabulary (`centered`, `outline` — words the LSP itself used to
//! suggest) or to a name in scope (a typo'd state, ask A-4), the hint names the
//! likely intention. `UIElement::arg_spans` is index-aligned with `args`, so
//! each warning points at the exact word.
//!
//! Ships as **warnings** through a separate entry point (the
//! [`lint_accessibility`](super::lint_accessibility) pattern), not as part of
//! [`validate_semantics`](super::validate_semantics): projects that compile
//! today must keep compiling (studio hazard 2). Promotion to a gate error is a
//! later, separate decision once the corpus is clean.

use std::collections::HashSet;

use crate::codegen::builtin::{builtin_to_html, modifier_to_class};
use crate::error::VocabWarning;
use crate::parser::vocabulary::MODIFIER_KEYWORDS;
use crate::parser::{
    Arg, ComponentRef, Declaration, Expr, Program, Statement, StatementKind, UIElement,
};

/// The sheet a default build ships, computed once.
///
/// Structural builds carry fewer rules, so checking against the full sheet is
/// the conservative choice: it reports only classes *no* mode defines.
fn stylesheet() -> &'static str {
    crate::themes::component_css()
}

/// Lint every UI element's positional arguments against the modifier vocabulary
/// and the program's own names. Returns one warning per dead word (empty =
/// clean), in declaration order.
pub fn lint_vocabulary(program: &Program, file: &str) -> Vec<VocabWarning> {
    let globals = global_names(program);
    let mut warnings = Vec::new();

    for decl in &program.declarations {
        let (body, props): (&[Statement], Vec<String>) = match decl {
            Declaration::Page(p) => (&p.body, Vec::new()),
            Declaration::Component(c) => {
                (&c.body, c.props.iter().map(|p| p.name.clone()).collect())
            }
            Declaration::App(a) => (&a.body, Vec::new()),
            // Stores and themes hold no UI.
            Declaration::Store(_) | Declaration::Theme(_) => continue,
        };
        // Hoisted, order-independent scope for the whole declaration: warning on
        // a name that IS declared somewhere would be a false positive, and a
        // false positive at a gate-adjacent lint is worse than a miss.
        let mut scope = globals.clone();
        scope.extend(props);
        hoist_names(body, &mut scope);
        walk(body, file, &scope, &mut warnings);
    }

    warnings
}

/// Names visible everywhere: declared pages/components/stores, and every store
/// member — store state is referenced bare in expressions (codegen resolves it
/// to `store.<name>`), so a bare `cartTotal` may be a store binding.
fn global_names(program: &Program) -> HashSet<String> {
    let mut names = HashSet::new();
    for decl in &program.declarations {
        match decl {
            Declaration::Page(p) => {
                names.insert(p.name.clone());
            }
            Declaration::Component(c) => {
                names.insert(c.name.clone());
            }
            Declaration::Store(s) => {
                names.insert(s.name.clone());
                hoist_names(&s.body, &mut names);
            }
            Declaration::App(_) | Declaration::Theme(_) => {}
        }
    }
    names
}

/// Collect every name `stmts` (recursively) can bind: state, derived, actions,
/// action params, fetch variables, loop variables. Over-inclusive on purpose —
/// this suppresses warnings, never creates them.
fn hoist_names(stmts: &[Statement], names: &mut HashSet<String>) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::State(s) => {
                names.insert(s.name.clone());
            }
            StatementKind::Derived(d) => {
                names.insert(d.name.clone());
            }
            StatementKind::Action(a) => {
                names.insert(a.name.clone());
                for p in &a.params {
                    names.insert(p.name.clone());
                }
                hoist_names(&a.body, names);
            }
            StatementKind::Effect(e) => hoist_names(&e.body, names),
            StatementKind::Fetch(f) => {
                names.insert(f.variable.clone());
                if let Some((err_var, body)) = &f.error_block {
                    names.insert(err_var.clone());
                    hoist_names(body, names);
                }
                if let Some(body) = &f.loading_block {
                    hoist_names(body, names);
                }
                if let Some(body) = &f.success_block {
                    hoist_names(body, names);
                }
            }
            StatementKind::For(f) => {
                names.insert(f.item.clone());
                if let Some(idx) = &f.index {
                    names.insert(idx.clone());
                }
                hoist_names(&f.body, names);
            }
            StatementKind::If(i) => {
                hoist_names(&i.then_body, names);
                for (_, body) in &i.else_if_branches {
                    hoist_names(body, names);
                }
                if let Some(body) = &i.else_body {
                    hoist_names(body, names);
                }
            }
            StatementKind::Show(s) => hoist_names(&s.body, names),
            StatementKind::UIElement(el) => hoist_names(&el.children, names),
            _ => {}
        }
    }
}

/// Walk the statement tree checking each UI element's positional args.
fn walk(stmts: &[Statement], file: &str, scope: &HashSet<String>, out: &mut Vec<VocabWarning>) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(el) => {
                check_element(el, file, scope, out);
                check_dead_variants(el, file, stylesheet(), out);
                walk(&el.children, file, scope, out);
            }
            StatementKind::If(i) => {
                walk(&i.then_body, file, scope, out);
                for (_, body) in &i.else_if_branches {
                    walk(body, file, scope, out);
                }
                if let Some(body) = &i.else_body {
                    walk(body, file, scope, out);
                }
            }
            StatementKind::For(f) => walk(&f.body, file, scope, out),
            StatementKind::Show(s) => walk(&s.body, file, scope, out),
            StatementKind::Fetch(f) => {
                if let Some(body) = &f.loading_block {
                    walk(body, file, scope, out);
                }
                if let Some((_, body)) = &f.error_block {
                    walk(body, file, scope, out);
                }
                if let Some(body) = &f.success_block {
                    walk(body, file, scope, out);
                }
            }
            // Actions/effects hold logic, not UI; everything else holds no
            // statements.
            _ => {}
        }
    }
}

/// V02: a real modifier that produces a class no stylesheet defines.
///
/// The class for a variant is built as `{base}--{modifier}` for *any* pairing,
/// so `Alert(elevated)` emits `.wf-alert--elevated` and `Text(pill)` emits
/// `.wf-text--pill`. Both parse, neither is a typo, and neither changes a pixel:
/// the stylesheet has no such rule. That is the same failure [`lint_vocabulary`]
/// exists to catch — a word the author wrote, every layer reported success for,
/// and nothing happened — arriving by a different route.
///
/// Suppressing the class in the codegen was the alternative, and it is the wrong
/// one: `{base}--{modifier}` is a documented styling hook, so an author may have
/// written the rule themselves. Saying so is right; deciding for them is not.
fn check_dead_variants(el: &UIElement, file: &str, stylesheet: &str, out: &mut Vec<VocabWarning>) {
    let ComponentRef::BuiltIn(name) = &el.component else {
        return;
    };
    let (_, base) = builtin_to_html(name);
    if base.is_empty() {
        return;
    }

    for (i, modifier) in el.modifiers.iter().enumerate() {
        let class = modifier_to_class(base, modifier);
        // Only per-component variants. A shared typography class like
        // `wf-text--bold` applies wherever it lands, and animation classes are
        // driven by keyframes the sheet always carries.
        if class.is_empty() || !class.starts_with(base) || class.starts_with("wf-animate") {
            continue;
        }
        if defines_class(stylesheet, &class) {
            continue;
        }
        let (line, column) = el
            .modifier_spans
            .get(i)
            .map(|s| (s.line as usize, s.col as usize))
            .unwrap_or((el.span.line as usize, el.span.col as usize));
        out.push(VocabWarning {
            rule_id: "V02".to_string(),
            message: format!(
                "`{modifier}` on {name} produces the class `{class}`, which no stylesheet defines"
            ),
            file: file.to_string(),
            line,
            column,
            hint: Some(format!(
                "Either drop it, or add a `.{class}` rule — the engine emits the class either way"
            )),
        });
    }
}

/// Whether `stylesheet` has a rule for `class`, matched at a selector boundary
/// so `.wf-btn` does not find `.wf-btn-group`.
fn defines_class(stylesheet: &str, class: &str) -> bool {
    let needle = format!(".{class}");
    let mut rest = stylesheet;
    while let Some(i) = rest.find(&needle) {
        let after = &rest[i + needle.len()..];
        if after
            .chars()
            .next()
            .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
        {
            return true;
        }
        rest = after;
    }
    false
}

fn check_element(el: &UIElement, file: &str, scope: &HashSet<String>, out: &mut Vec<VocabWarning>) {
    for (i, arg) in el.args.iter().enumerate() {
        // Positional bare identifiers only: that is the position where a typo'd
        // modifier lands, and where the parser's vocabulary check is the sole
        // discriminator between "modifier" and "expression".
        let Arg::Positional(Expr::Identifier(name)) = arg else {
            continue;
        };
        if scope.contains(name) {
            continue;
        }
        // (A word from the vocabulary can't reach here — the parser would have
        // consumed it as a modifier — but the check is cheap and makes the
        // invariant local.)
        if MODIFIER_KEYWORDS.contains(&name.as_str()) {
            continue;
        }
        let span = &el.arg_spans[i];
        let hint = suggest(name, scope);
        out.push(VocabWarning {
            rule_id: "V01".into(),
            message: format!(
                "`{name}` is not a modifier and nothing in scope declares it — it does nothing"
            ),
            file: file.into(),
            line: span.line as usize,
            column: span.col as usize,
            hint,
        });
    }
}

/// The closest modifier or in-scope name within edit distance 2, if any —
/// `outline` → `outlined`, `centered` → `center`, `cuont` → `count`.
fn suggest(word: &str, scope: &HashSet<String>) -> Option<String> {
    let candidates = MODIFIER_KEYWORDS
        .iter()
        .copied()
        .chain(scope.iter().map(String::as_str));
    let best = candidates
        .filter_map(|c| {
            let d = levenshtein(word, c);
            (d <= 2 && d > 0).then_some((d, c))
        })
        .min_by_key(|&(d, c)| (d, c.len()))?;
    Some(format!("did you mean `{}`?", best.1))
}

/// Plain Levenshtein distance; the inputs are single words, so O(n·m) is fine.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let sub = prev[j] + usize::from(ca != cb);
            cur[j + 1] = sub.min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn lint(source: &str) -> Vec<VocabWarning> {
        let tokens = Lexer::new(source, "<test>").tokenize().expect("lex");
        let program = Parser::new(tokens, "<test>").parse().expect("parse");
        lint_vocabulary(&program, "<test>")
    }

    #[test]
    fn an_unknown_bare_word_argument_is_flagged() {
        // The studio's probe case (finding 0.7): compiles clean, does nothing.
        let w = lint(r#"Page Home (path: "/") { Container { Button("Save", huge) } }"#);
        assert_eq!(w.len(), 1, "{w:?}");
        assert_eq!(w[0].rule_id, "V01");
        assert!(w[0].message.contains("`huge`"));
        assert!(w[0].line >= 1 && w[0].column >= 1, "must point at the word");
    }

    #[test]
    fn near_misses_of_the_vocabulary_get_a_suggestion() {
        // The exact words the LSP used to suggest and the compiler dropped.
        let w = lint(r#"Page Home (path: "/") { Text("hi", centered) }"#);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].hint.as_deref(), Some("did you mean `center`?"));

        let w = lint(r#"Page Home (path: "/") { Button("Go", outline) }"#);
        assert_eq!(w[0].hint.as_deref(), Some("did you mean `outlined`?"));
    }

    #[test]
    fn a_typoed_state_name_suggests_the_state() {
        // Ask A-4's case: the mistake lands at argument position, where the
        // span exists.
        let w = lint(r#"Page Home (path: "/") { state count = 0 Text(cuont) }"#);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].hint.as_deref(), Some("did you mean `count`?"));
    }

    #[test]
    fn real_bindings_do_not_warn() {
        let w = lint(
            r#"
            Store Cart { state total = 0 }
            Component Price (amount: Number) { Text(amount) }
            Page Home (path: "/") {
                use Cart
                state items = ["a"]
                derived first = items
                Text(total)
                Price(first)
                for item in items { Text(item) }
            }
            "#,
        );
        assert_eq!(w, Vec::new());
    }

    #[test]
    fn valid_modifiers_still_parse_as_modifiers() {
        // Sanity: the vocabulary refactor must not change what is a modifier.
        let w = lint(r#"Page Home (path: "/") { Button("Save", primary, large) Text("x", bold) }"#);
        assert_eq!(w, Vec::new());
    }

    #[test]
    fn named_arguments_are_not_checked() {
        // Named args reference actions, pages, options — a different namespace
        // with different failure modes; flagging them here would false-positive.
        let w = lint(
            r#"Page Home (path: "/") { state n = 0 action bump() { n = n + 1 } Button("Go", onClick: bump) }"#,
        );
        assert_eq!(w, Vec::new());
    }

    #[test]
    fn fetch_and_loop_bindings_resolve() {
        let w = lint(
            r#"
            Page Home (path: "/") {
                fetch posts from "https://example.com/posts" {
                    loading { Text("...") }
                    error (e) { Text(e) }
                    success { for p in posts { Text(p) } }
                }
            }
            "#,
        );
        assert_eq!(w, Vec::new());
    }

    #[test]
    fn distance_is_bounded_so_arbitrary_words_get_no_suggestion() {
        let w = lint(r#"Page Home (path: "/") { Text("hi", strikethrough) }"#);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].hint, None, "nothing is within distance 2");
    }

    #[test]
    fn levenshtein_basics() {
        assert_eq!(levenshtein("outline", "outlined"), 1);
        assert_eq!(levenshtein("centered", "center"), 2);
        assert_eq!(levenshtein("huge", "large"), 3);
        assert_eq!(levenshtein("same", "same"), 0);
    }
}

#[cfg(test)]
mod dead_variant_tests {
    //! A modifier that is spelled right and still does nothing.
    //!
    //! `lint_vocabulary` already catches invented words. This catches the other
    //! half: a word from the real vocabulary, on a component whose stylesheet
    //! section has no rule for it.
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn warnings(src: &str) -> Vec<VocabWarning> {
        let tokens = Lexer::new(src, "<t>").tokenize().expect("lex");
        let program = Parser::new(tokens, "<t>").parse().expect("parse");
        lint_vocabulary(&program, "<t>")
            .into_iter()
            .filter(|w| w.rule_id == "V02")
            .collect()
    }

    #[test]
    fn a_variant_the_stylesheet_defines_is_silent() {
        assert!(
            warnings(r#"Page P (path: "/") { Button("Save", primary) }"#).is_empty(),
            "`.wf-btn--primary` exists"
        );
        assert!(warnings(r#"Page P (path: "/") { Card(elevated) { Text("x") } }"#).is_empty());
    }

    #[test]
    fn a_variant_with_no_rule_behind_it_is_reported() {
        let w = warnings(r#"Page P (path: "/") { Alert("Careful", elevated) }"#);
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(
            w[0].message.contains("wf-alert--elevated"),
            "{}",
            w[0].message
        );
        assert!(
            w[0].hint.as_ref().unwrap().contains("add a"),
            "{:?}",
            w[0].hint
        );
    }

    /// Typography and animation classes apply wherever they land, so they are
    /// never a per-component gap.
    #[test]
    fn shared_and_animated_classes_are_not_flagged() {
        assert!(
            warnings(r#"Page P (path: "/") { Card(bold) { Text("x") } }"#).is_empty(),
            "`wf-text--bold` styles whatever it is put on"
        );
        assert!(
            warnings(r#"Page P (path: "/") { Card(fadeIn) { Text("x") } }"#).is_empty(),
            "animation classes are carried by keyframes, not per component"
        );
    }

    #[test]
    fn the_warning_points_at_the_modifier_itself() {
        let w = warnings("Page P (path: \"/\") {\n    Alert(\"x\", pill)\n}");
        assert_eq!(w.len(), 1);
        assert_eq!(
            w[0].line, 2,
            "the warning should name the line the word is on"
        );
    }
}
