//! Invariants the engine's HTML must hold whatever the source says.
//!
//! These are not about any one component. They are the properties a browser, a
//! crawler, a screen reader and a diff all rely on: no stray characters in the
//! markup, void elements left void, text escaped, tags balanced.

mod common;

use common::*;

/// A page that exercises a wide slice of the component surface at once, so the
/// document-level checks below run against realistic output rather than a single
/// element.
const KITCHEN_SINK: &str = r#"
Page P (path: "/", title: "T") {
    Container {
        Navbar {
            Text("Brand")
            Link("Home", to: "/")
        }
        Heading("Title", h1)
        Text("Body copy", muted)
        Row {
            Card(elevated) {
                Heading("Card", h3)
                Text("Inside")
                Button("Go", primary)
            }
            Card(outlined) {
                Image(src: "/a.png", alt: "a")
                Divider
                Badge("New", success)
            }
        }
        Table {
            Thead { Trow { Tcell("H") } }
            Tbody { Trow { Tcell("C") } }
        }
        List { Text("one") }
        Form {
            Input(email, placeholder: "you@example.com")
            Select { Option("a") Option("b") }
            Checkbox(label: "Agree")
            Button("Submit", primary)
        }
        Alert("Careful", warning)
        Spacer
        Blockquote("Quoted")
        Code("let x = 1")
    }
}
"#;

/// Characters that have no business in generated markup. U+200B in particular is
/// invisible in an editor and in a browser, but it is inside the document: it
/// lands in `textContent`, in copied text, in the bytes a crawler indexes, and it
/// makes byte-exact snapshot comparison impossible.
const FORBIDDEN: &[(char, &str)] = &[
    ('\u{200B}', "ZERO WIDTH SPACE"),
    ('\u{200C}', "ZERO WIDTH NON-JOINER"),
    ('\u{200D}', "ZERO WIDTH JOINER"),
    ('\u{FEFF}', "ZERO WIDTH NO-BREAK SPACE / BOM"),
    ('\u{00A0}', "NO-BREAK SPACE"),
    ('\u{FFFE}', "noncharacter U+FFFE"),
    ('\u{FFFF}', "noncharacter U+FFFF"),
];

#[test]
fn output_contains_no_invisible_or_noncharacter_junk() {
    let mut failures = Vec::new();
    for backend in Backend::ALL {
        let out = raw_output(backend, KITCHEN_SINK);
        for (ch, name) in FORBIDDEN {
            let count = out.matches(*ch).count();
            if count > 0 {
                let at = out.find(*ch).unwrap();
                let start = at.saturating_sub(40);
                let ctx: String = out[start..(at + 10).min(out.len())].replace(*ch, "␣");
                failures.push(format!(
                    "{}: {} occurrences of {} (U+{:04X}); first near: …{}…",
                    backend.name(),
                    count,
                    name,
                    *ch as u32,
                    ctx.trim()
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "junk characters in output:\n{}",
        failures.join("\n")
    );
}

/// Void elements take no children and no closing tag. `<hr>Text</hr>` is not
/// HTML, and a browser will hoist the text out of it.
#[test]
fn void_elements_are_emitted_without_children_or_closing_tags() {
    let cases = [
        "Divider(\"text\")",
        "Image(src: \"/a.png\", alt: \"a\")",
        "Input(placeholder: \"p\")",
    ];

    let mut failures = Vec::new();
    for body in cases {
        let src = page(body);
        for backend in Backend::ALL {
            let out = raw_output(backend, &src);
            for void in VOID_ELEMENTS {
                let close = format!("</{}>", void);
                if out.contains(&close) {
                    failures.push(format!(
                        "`{}` in {} emitted a closing {}",
                        body,
                        backend.name(),
                        close
                    ));
                }
            }
            // The SPA path expresses children as extra `WF.h` arguments after the
            // attribute object; a void element handed one is asking the runtime to
            // append into a node that cannot hold children.
            if backend == Backend::Spa {
                if let Some(e) = root(backend, &src) {
                    if VOID_ELEMENTS.contains(&e.tag.as_str()) && e.raw.contains("}, ") {
                        failures.push(format!(
                            "`{}` in {} gave <{}> a child: {}",
                            body,
                            backend.name(),
                            e.tag,
                            e.raw.trim()
                        ));
                    }
                }
            }
            // The static renderers must not put text inside a void element either.
            if backend != Backend::Spa {
                for void in VOID_ELEMENTS {
                    let open = format!("<{}", void);
                    if let Some(p) = out.find(&open) {
                        let tail = &out[p..];
                        if let Some(gt) = tail.find('>') {
                            let after: String =
                                tail[gt + 1..].chars().take_while(|c| *c != '<').collect();
                            if !after.trim().is_empty() {
                                failures.push(format!(
                                    "`{}` in {} put text after <{}>: {:?}",
                                    body,
                                    backend.name(),
                                    void,
                                    after.trim()
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "void element misuse:\n{}",
        failures.join("\n")
    );
}

/// Text from the source must not be able to close a tag or open a new one.
#[test]
fn text_content_is_escaped() {
    let src = page(r#"Text("<script>alert(1)</script> & \"quoted\"")"#);
    if parse_program(&src).is_err() {
        // The lexer may reject the escapes; fall back to the unquoted case.
        let src = page(r#"Text("<script>alert(1)</script> & done")"#);
        assert_escaped(&src);
        return;
    }
    assert_escaped(&src);
}

fn assert_escaped(src: &str) {
    let mut failures = Vec::new();
    for backend in [Backend::Ssg, Backend::Template] {
        let out = raw_output(backend, src);
        if out.contains("<script>alert(1)</script>") {
            failures.push(format!(
                "{} emitted an unescaped <script> from source text",
                backend.name()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "escaping failure:\n{}",
        failures.join("\n")
    );
}

/// Attribute values must be escaped so a quote in the source cannot end the
/// attribute and inject another.
#[test]
fn attribute_values_are_escaped() {
    let src = page(r#"Image(src: "/a.png", alt: "a > b & c")"#);
    let mut failures = Vec::new();
    for backend in [Backend::Ssg, Backend::Template] {
        let out = raw_output(backend, &src);
        if out.contains("alt=\"a > b & c\"") {
            failures.push(format!(
                "{} emitted a raw `>` and `&` inside an attribute value",
                backend.name()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "attribute escaping failure:\n{}",
        failures.join("\n")
    );
}

/// Every non-void element opened in the static output must be closed, in order.
#[test]
fn static_html_tags_are_balanced() {
    let mut failures = Vec::new();
    for backend in [Backend::Ssg, Backend::Template] {
        let out = raw_output(backend, KITCHEN_SINK);
        if let Err(e) = check_balanced(&out) {
            failures.push(format!("{}: {}", backend.name(), e));
        }
    }
    assert!(
        failures.is_empty(),
        "unbalanced markup:\n{}",
        failures.join("\n")
    );
}

fn check_balanced(html: &str) -> Result<(), String> {
    let mut stack: Vec<String> = Vec::new();
    let mut rest = html;
    while let Some(i) = rest.find('<') {
        let after = &rest[i + 1..];
        if after.starts_with('!') {
            rest = after;
            continue;
        }
        let Some(end) = after.find('>') else { break };
        let inner = &after[..end];
        if let Some(name) = inner.strip_prefix('/') {
            let name = name.trim().to_ascii_lowercase();
            match stack.pop() {
                Some(open) if open == name => {}
                Some(open) => return Err(format!("</{}> closed while <{}> was open", name, open)),
                None => return Err(format!("</{}> with nothing open", name)),
            }
        } else {
            let name = inner
                .split(char::is_whitespace)
                .next()
                .unwrap_or("")
                .trim_end_matches('/')
                .to_ascii_lowercase();
            if !name.is_empty() && !VOID_ELEMENTS.contains(&name.as_str()) && !inner.ends_with('/')
            {
                stack.push(name);
            }
        }
        rest = after;
    }
    if !stack.is_empty() {
        return Err(format!("never closed: {:?}", stack));
    }
    Ok(())
}

/// The generated JS must be free of the placeholder markers the codegen uses
/// internally, and must not leave an unexpanded component comment in a static
/// page — a page that paints `<!--wf-component-->` is a page that paints nothing.
#[test]
fn no_internal_placeholders_leak_into_output() {
    let mut failures = Vec::new();
    for backend in Backend::ALL {
        // The SPA bundle embeds a fixed runtime; only the generated tail is ours.
        let out = match backend {
            Backend::Spa => spa_generated(KITCHEN_SINK),
            _ => raw_output(backend, KITCHEN_SINK),
        };
        for marker in [
            "wf-component",
            "_StyleBlock",
            "undefined",
            "[object Object]",
        ] {
            if out.contains(marker) {
                failures.push(format!("{} leaked `{}`", backend.name(), marker));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "internal markers in output:\n{}",
        failures.join("\n")
    );
}

/// The SPA bundle must be syntactically valid JavaScript. A codegen bug that
/// produces a stray brace or an unterminated string takes the whole page down,
/// and nothing in the build currently checks.
#[test]
fn generated_js_is_syntactically_valid() {
    let js = raw_output(Backend::Spa, KITCHEN_SINK);

    // Balance check over braces/parens/brackets outside string and comment
    // context — a cheap stand-in for a parser, but it catches the failure modes
    // a template-string codegen actually produces.
    let mut depth_curly = 0i64;
    let mut depth_paren = 0i64;
    let mut in_str: Option<u8> = None;
    let mut escape = false;
    let mut in_line_comment = false;
    let mut prev = 0u8;

    for &b in js.as_bytes() {
        if in_line_comment {
            if b == b'\n' {
                in_line_comment = false;
            }
            prev = b;
            continue;
        }
        match in_str {
            Some(q) => {
                if escape {
                    escape = false;
                } else if b == b'\\' {
                    escape = true;
                } else if b == q {
                    in_str = None;
                }
            }
            None => match b {
                b'/' if prev == b'/' => in_line_comment = true,
                b'"' | b'\'' | b'`' => in_str = Some(b),
                b'{' => depth_curly += 1,
                b'}' => depth_curly -= 1,
                b'(' => depth_paren += 1,
                b')' => depth_paren -= 1,
                _ => {}
            },
        }
        prev = b;
    }

    assert!(
        in_str.is_none(),
        "generated JS ends inside an unterminated string literal"
    );
    assert_eq!(
        depth_curly, 0,
        "generated JS has unbalanced braces ({depth_curly:+})"
    );
    assert_eq!(
        depth_paren, 0,
        "generated JS has unbalanced parens ({depth_paren:+})"
    );
}

/// Indentation aside, the static renderers must not emit an element whose class
/// attribute is empty — `class=""` is noise that also defeats class-based
/// selectors written against the output.
#[test]
fn no_empty_class_attributes() {
    let mut failures = Vec::new();
    for backend in Backend::ALL {
        let out = raw_output(backend, KITCHEN_SINK);
        if out.contains("class=\"\"") || out.contains("className: \"\"") {
            failures.push(backend.name().to_string());
        }
    }
    assert!(
        failures.is_empty(),
        "empty class attributes in: {}",
        failures.join(", ")
    );
}
