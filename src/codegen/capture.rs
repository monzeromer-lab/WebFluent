//! Which elements a generated function's own closures reach.
//!
//! A static build's page is taken over in place when its script runs: an
//! element the script drew is matched to the element already painted in
//! its place, and the painted one is kept — unless something still holds
//! the drawn one, which must then be the node the reader gets (see
//! `runtime/modules/hydrate.js`). The runtime knows what *it* holds. What
//! it cannot see is a closure the compiler wrote that names an element:
//! `WF.effect(() => { _e7.style.display = … })`, a handler that reaches
//! another element. Those elements are named to it with `WF.live(…)`.
//!
//! This reads a function the compiler just wrote, as the minifier and
//! `jscheck` read JavaScript — a scanner that knows where a string, a
//! template, a regular expression and a comment end — and reports every
//! element variable (`_e<n>`) declared at the function's top level that is
//! named inside a block deeper than that, or inside the body of an arrow
//! function. It errs one way only: a name it is unsure of is reported, and
//! an element reported needlessly is drawn again rather than kept.

use crate::codegen::minify::{
    is_expression_keyword, is_word_byte, regex_end, string_end, template_end,
};
use std::collections::BTreeSet;

/// The element variables declared at the top of `function` that its
/// closures name, in the order they were declared. `function` starts at the
/// function's opening — `function Page_Home(params) {` or `(function() {`
/// — so its body is the first `{`.
pub fn captured_elements(function: &str) -> Vec<String> {
    let bytes = function.as_bytes();
    let mut i = 0;
    let mut braces = 0usize;
    // Open `(` and `[`, for where an arrow's expression body ends.
    let mut parens = 0usize;
    // Arrow functions whose body is an expression: the bracket depths they
    // started at. The body ends at a `,` or `;` at those depths, or at a
    // bracket that closes below them.
    let mut arrows: Vec<(usize, usize)> = Vec::new();
    let mut declared: Vec<String> = Vec::new();
    let mut captured: BTreeSet<String> = BTreeSet::new();
    let mut last: Option<u8> = None;
    let mut last_word = String::new();
    let mut last_was_word = false;
    let mut after_const = false;

    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = function[i + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |end| i + 2 + end + 2);
            }
            b'"' | b'\'' => {
                i = string_end(bytes, i).min(bytes.len());
                last = Some(c);
                last_was_word = true;
                last_word.clear();
                after_const = false;
            }
            b'`' => {
                // A splice inside a template may name an element; any name
                // in one counts, which can only report too much.
                let end = template_end(bytes, i).min(bytes.len());
                for name in element_names(&function[i..end]) {
                    captured.insert(name);
                }
                i = end;
                last = Some(c);
                last_was_word = true;
                last_word.clear();
                after_const = false;
            }
            b'/' if (!last_was_word || is_expression_keyword(&last_word))
                && !matches!(last, Some(b')' | b']')) =>
            {
                i = regex_end(bytes, i).min(bytes.len());
                last = Some(b'/');
                last_was_word = true;
                last_word.clear();
            }
            b'=' if bytes.get(i + 1) == Some(&b'>') => {
                i += 2;
                let mut j = i;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if bytes.get(j) != Some(&b'{') {
                    arrows.push((braces, parens));
                }
                last = Some(b'>');
                last_was_word = false;
                after_const = false;
            }
            b'{' => {
                braces += 1;
                i += 1;
                last = Some(c);
                last_was_word = false;
            }
            b'}' => {
                braces = braces.saturating_sub(1);
                arrows.retain(|&(b, _)| b <= braces);
                i += 1;
                last = Some(c);
                last_was_word = false;
            }
            b'(' | b'[' => {
                parens += 1;
                i += 1;
                last = Some(c);
                last_was_word = false;
            }
            b')' | b']' => {
                parens = parens.saturating_sub(1);
                arrows.retain(|&(b, p)| !(b == braces && p > parens));
                i += 1;
                last = Some(c);
                last_was_word = false;
            }
            b',' | b';' => {
                arrows.retain(|&(b, p)| !(b == braces && p == parens));
                i += 1;
                last = Some(c);
                last_was_word = false;
                after_const = false;
            }
            _ if is_word_byte(c) => {
                let start = i;
                while i < bytes.len() && is_word_byte(bytes[i]) {
                    i += 1;
                }
                let word = &function[start..i];
                // A property (`x._e1`) is not the variable.
                let property = start > 0 && bytes[start - 1] == b'.';
                if !property && is_element_var(word) {
                    if after_const && braces == 1 {
                        declared.push(word.to_string());
                    } else if braces > 1 || !arrows.is_empty() {
                        captured.insert(word.to_string());
                    }
                }
                after_const = word == "const" || word == "let";
                last_word = word.to_string();
                last_was_word = true;
                last = Some(c);
            }
            _ => {
                i += 1;
                last = Some(c);
                last_was_word = false;
                after_const = false;
            }
        }
    }
    declared
        .into_iter()
        .filter(|name| captured.contains(name))
        .collect()
}

fn is_element_var(word: &str) -> bool {
    word.strip_prefix("_e")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// Every `_e<n>` in `text`, wherever it stands.
fn element_names(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if is_word_byte(bytes[i]) {
            let start = i;
            while i < bytes.len() && is_word_byte(bytes[i]) {
                i += 1;
            }
            let word = &text[start..i];
            if is_element_var(word) {
                out.push(word.to_string());
            }
        } else {
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::captured_elements;

    #[test]
    fn what_is_only_built_is_not_reported() {
        let f = r#"function Page_Home(params) {
  const _root = document.createDocumentFragment();
  const _e1 = WF.el("div", { className: "a" });
  const _e2 = WF.el("p", { className: "lead" }, "Hello");
  _e1.appendChild(_e2);
  _e1.classList.add("page");
  WF.when(_e1, () => _open(), () => { const _e3 = WF.el("p", {}, "x"); return _e3; });
  _root.appendChild(_e1);
  return _root;
}"#;
        assert!(
            captured_elements(f).is_empty(),
            "{:?}",
            captured_elements(f)
        );
    }

    #[test]
    fn an_effect_or_a_handler_that_names_an_element_holds_it() {
        let f = r#"function Component_Tabs(_p, _slots) {
  const _e1 = WF.el("div", {});
  const _e2 = WF.el("div", {});
  const _e3 = WF.el("button", {});
  WF.effect(() => { _e1.style.display = _a() === 0 ? 'block' : 'none'; });
  _e3.addEventListener("click", (e) => { _e2.focus(); });
  return _frag;
}"#;
        assert_eq!(captured_elements(f), vec!["_e1", "_e2"]);
    }

    #[test]
    fn an_arrow_whose_body_is_an_expression_holds_what_it_names() {
        let f = r#"function Page_P(params) {
  const _e1 = WF.el("div", {});
  const _e2 = WF.el("div", {});
  const later = () => _e1.scrollTop, other = 1;
  WF.when(_e2, () => _open(), null);
  return _root;
}"#;
        assert_eq!(captured_elements(f), vec!["_e1"]);
    }

    #[test]
    fn strings_comments_and_properties_are_not_names() {
        let f = r#"function Page_P(params) {
  const _e1 = WF.el("div", { title: "() => { _e1 }" });
  // () => { _e1 }
  const x = { y: () => obj._e1 };
  return _root;
}"#;
        assert!(
            captured_elements(f).is_empty(),
            "{:?}",
            captured_elements(f)
        );
    }

    #[test]
    fn a_template_that_names_an_element_counts() {
        let f = "function Page_P(params) {\n  const _e1 = WF.el(\"div\", {});\n  const s = `${_e1.id}`;\n  return _root;\n}";
        assert_eq!(captured_elements(f), vec!["_e1"]);
    }
}
