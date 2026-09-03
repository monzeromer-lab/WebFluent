use std::fs;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::*;
use webfluent::lexer::Lexer;
use webfluent::linter::{lint_accessibility, lint_contrast, lint_vocabulary, validate_semantics};
use webfluent::parser::{Parser, Program};
use webfluent::themes::resolve_tokens;
use wf_lsp::completion::provide_completions;
use wf_lsp::definition::find_definition;
use wf_lsp::hover::provide_hover;
use wf_lsp::line_index::LineIndex;
use wf_lsp::symbols::build_document_symbols;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize workspace root")
}

fn collect_wf_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.exists() {
        return files;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_wf_files(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("wf") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// Helper: parse source text into an AST Program.
fn try_parse(source: &str, file: &str) -> Result<Program, String> {
    let mut lexer = Lexer::new(source, file);
    let tokens = lexer.tokenize().map_err(|e| format!("{e:?}"))?;
    let mut parser = Parser::new(tokens, file);
    parser.parse().map_err(|e| format!("{e:?}"))
}

// ---------------------------------------------------------------------------
// 1. Deep test of all real files in the documentation site (`site/src/**/*.wf`)
// ---------------------------------------------------------------------------

#[test]
fn test_real_docs_site_all_files() {
    let root = workspace_root();
    let site_dir = root.join("site/src");
    let files = collect_wf_files(&site_dir);

    assert!(
        files.len() >= 15,
        "Expected at least 15 documentation site files, found {}",
        files.len()
    );

    for path in &files {
        let rel_path = path.strip_prefix(&root).unwrap_or(path);
        let filename = rel_path.to_string_lossy().to_string();
        let source = fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
        let uri = Url::from_file_path(path).unwrap();

        // 1. LineIndex invariants
        let index = LineIndex::new(&source);
        for (line_idx, line) in source.lines().enumerate() {
            let pos = Position::new(line_idx as u32, 0);
            if let Some(offset) = index.position_to_offset(&source, pos) {
                let roundtrip_pos = index.offset_to_position(&source, offset);
                assert_eq!(
                    roundtrip_pos.line, line_idx as u32,
                    "Line index mismatch in file {filename} at line {line_idx}"
                );
            }

            // Test line-end coordinate
            let col = line.encode_utf16().count() as u32;
            let end_pos = Position::new(line_idx as u32, col);
            if let Some(offset) = index.position_to_offset(&source, end_pos) {
                let roundtrip_end = index.offset_to_position(&source, offset);
                assert_eq!(
                    roundtrip_end.line, line_idx as u32,
                    "Line end roundtrip mismatch in {filename} at line {line_idx}"
                );
            }
        }

        // 2. Parse and check diagnostics
        let program = match try_parse(&source, &filename) {
            Ok(p) => p,
            Err(e) => panic!("Failed to parse real docs file {filename}: {e}"),
        };

        // 3. Linters must not panic on real code
        let _semantics = validate_semantics(&program, &filename);
        let _vocab = lint_vocabulary(&program, &filename);
        let _a11y = lint_accessibility(&program);
        let resolved = resolve_tokens(&program, &Default::default()).unwrap_or_default();
        let _contrast = lint_contrast(&program, &resolved);

        // 4. Document symbols (hierarchical + flat) must be valid and monotonic
        let sym_res = build_document_symbols(&program, &source, &index, &uri, true);
        match sym_res {
            DocumentSymbolResponse::Nested(nested) => {
                assert!(
                    !nested.is_empty(),
                    "Expected symbols in file {filename}, got empty list"
                );
                for sym in &nested {
                    assert!(
                        sym.range.start.line <= sym.range.end.line,
                        "Inverted line range in symbol {} of {filename}",
                        sym.name
                    );
                    if sym.range.start.line == sym.range.end.line {
                        assert!(
                            sym.range.start.character <= sym.range.end.character,
                            "Inverted char range in symbol {} of {filename}",
                            sym.name
                        );
                    }
                }
            }
            DocumentSymbolResponse::Flat(_) => panic!("Expected nested symbol response"),
        }

        let flat_res = build_document_symbols(&program, &source, &index, &uri, false);
        match flat_res {
            DocumentSymbolResponse::Flat(flat) => {
                assert!(!flat.is_empty(), "Expected flat symbols in file {filename}");
                for sym in &flat {
                    assert_eq!(sym.location.uri, uri);
                }
            }
            DocumentSymbolResponse::Nested(_) => panic!("Expected flat symbol response"),
        }

        // 5. Hover stress test on every line of real code
        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // Find first word in line and query hover
            if let Some(word) = trimmed.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.').next()
                && !word.is_empty() {
                let col_idx = line.find(word).unwrap_or(0);
                let pos = Position::new(line_idx as u32, col_idx as u32);
                let hover_result = provide_hover(&source, pos, Some(&program));
                if let Some(hover) = hover_result
                    && let HoverContents::Markup(m) = hover.contents {
                    assert_eq!(m.kind, MarkupKind::Markdown);
                    assert!(!m.value.is_empty());
                }
            }
        }

        // 6. Completion inside first block
        let open_brace_pos = source.lines().enumerate().find_map(|(idx, line)| {
            line.find('{').map(|col| Position::new(idx as u32, (col + 1) as u32))
        });
        if let Some(pos) = open_brace_pos {
            let completions = provide_completions(&source, pos, Some(&program));
            assert!(
                !completions.is_empty(),
                "Expected completions inside open brace in {filename}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Deep test of all application fixtures (`tests/fixtures/**/*.wf`)
// ---------------------------------------------------------------------------

#[test]
fn test_real_fixtures_all_files() {
    let root = workspace_root();
    let fixtures_dir = root.join("tests/fixtures");
    let files = collect_wf_files(&fixtures_dir);

    assert!(
        files.len() >= 10,
        "Expected at least 10 fixtures files, found {}",
        files.len()
    );

    for path in &files {
        let rel_path = path.strip_prefix(&root).unwrap_or(path);
        let filename = rel_path.to_string_lossy().to_string();
        let source = fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
        let uri = Url::from_file_path(path).unwrap();
        let index = LineIndex::new(&source);

        let program = match try_parse(&source, &filename) {
            Ok(p) => p,
            Err(e) => panic!("Failed to parse fixture {filename}: {e}"),
        };

        // Diagnostics
        let _semantics = validate_semantics(&program, &filename);
        let _vocab = lint_vocabulary(&program, &filename);
        let _a11y = lint_accessibility(&program);
        let resolved = resolve_tokens(&program, &Default::default()).unwrap_or_default();
        let _contrast = lint_contrast(&program, &resolved);

        // Symbols
        let syms = build_document_symbols(&program, &source, &index, &uri, true);
        if let DocumentSymbolResponse::Nested(nested) = syms {
            assert!(!nested.is_empty(), "Expected symbols in {filename}");
        }

        // Definition jumps for all declarations in the file
        for decl in &program.declarations {
            let (name, line) = match decl {
                webfluent::parser::ast::Declaration::Page(p) => (&p.name, p.span.line),
                webfluent::parser::ast::Declaration::Component(c) => (&c.name, c.span.line),
                webfluent::parser::ast::Declaration::Store(s) => (&s.name, s.span.line),
                webfluent::parser::ast::Declaration::Theme(t) => (&t.name, t.span.line),
                webfluent::parser::ast::Declaration::App(_) => continue,
            };

            let pos = Position::new(line - 1, 0);
            if let Some(GotoDefinitionResponse::Scalar(loc)) = find_definition(&program, &source, &index, &uri, pos) {
                assert_eq!(loc.uri, uri);
                assert_eq!(loc.range.start.line, line - 1);
            }

            // Hover on declaration name
            let hover = provide_hover(&source, pos, Some(&program));
            assert!(hover.is_some(), "Expected hover on declaration `{name}` in {filename}");
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Documentation snippet extraction & LSP conformance test
// ---------------------------------------------------------------------------

#[test]
fn test_documentation_markdown_snippets() {
    let root = workspace_root();
    let doc_files = vec![
        root.join("README.md"),
        root.join("spec/SPEC.md"),
        root.join("spec/ACCESSIBILITY_SPEC.md"),
        root.join("spec/ANIMATION_SPEC.md"),
        root.join("spec/I18N_SPEC.md"),
        root.join("spec/SLIDES_SPEC.md"),
        root.join("spec/TEMPLATE_ENGINE_SPEC.md"),
    ];

    let mut total_snippets = 0;
    let mut parsed_snippets = 0;

    for doc_path in doc_files {
        if !doc_path.exists() {
            continue;
        }
        let content = fs::read_to_string(&doc_path).unwrap();
        let doc_name = doc_path.file_name().unwrap().to_string_lossy().to_string();

        let mut in_block = false;
        let mut current_block = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```wf") || (doc_name == "README.md" && trimmed == "```" && !in_block) {
                in_block = true;
                current_block.clear();
                continue;
            } else if trimmed == "```" && in_block {
                in_block = false;
                total_snippets += 1;
                let snippet = current_block.join("\n");

                // Check if snippet is a standalone declaration or needs wrapping
                let standalone = snippet.contains("Page ")
                    || snippet.contains("Component ")
                    || snippet.contains("Store ")
                    || snippet.contains("Theme ")
                    || snippet.contains("App {");

                let full_source = if standalone {
                    snippet.clone()
                } else {
                    format!("Page DocPreview (path: \"/preview\") {{\n{}\n}}", snippet)
                };

                if let Ok(prog) = try_parse(&full_source, &doc_name) {
                    parsed_snippets += 1;
                    let index = LineIndex::new(&full_source);
                    let uri = Url::parse("file:///doc_snippet.wf").unwrap();

                    // Document symbols must not panic
                    let _ = build_document_symbols(&prog, &full_source, &index, &uri, true);

                    // Hover must not panic
                    let _ = provide_hover(&full_source, Position::new(0, 0), Some(&prog));

                    // Completions must produce items
                    let items = provide_completions(&full_source, Position::new(0, 5), Some(&prog));
                    assert!(!items.is_empty());
                }
            } else if in_block {
                current_block.push(line);
            }
        }
    }

    assert!(
        total_snippets >= 20,
        "Expected at least 20 documentation snippets, found {total_snippets}"
    );
    assert!(
        parsed_snippets >= 15,
        "Expected at least 15 valid parsable documentation snippets, parsed {parsed_snippets}"
    );
}

// ---------------------------------------------------------------------------
// 4. Real-world typing simulation & snapshot resilience test
// ---------------------------------------------------------------------------

#[test]
fn test_simulated_interactive_typing_resilience() {
    let base_source = "Page Dashboard (path: \"/dashboard\") {\n    Container {\n        Heading(\"Metrics\", h1)\n    }\n}\n";
    let index = LineIndex::new(base_source);
    let uri = Url::parse("file:///dashboard.wf").unwrap();

    let valid_prog = try_parse(base_source, "dashboard.wf").expect("parse initial");

    // Check initial symbols
    let syms = build_document_symbols(&valid_prog, base_source, &index, &uri, true);
    if let DocumentSymbolResponse::Nested(s) = syms {
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].name, "Dashboard");
    }

    // Simulate keystrokes inside Container:
    // Step 1: User types unclosed component invocation `        Button("Save`
    let dirty_source = "Page Dashboard (path: \"/dashboard\") {\n    Container {\n        Heading(\"Metrics\", h1)\n        Button(\"Save\n    }\n}\n";
    let dirty_index = LineIndex::new(dirty_source);
    let parse_res = try_parse(dirty_source, "dashboard.wf");
    assert!(parse_res.is_err(), "Expected syntax error during typing");

    // The LSP retains last_valid_program: verify hover and symbols still work smoothly!
    let syms_fallback = build_document_symbols(&valid_prog, dirty_source, &dirty_index, &uri, true);
    if let DocumentSymbolResponse::Nested(s) = syms_fallback {
        assert_eq!(s[0].name, "Dashboard");
    }

    // Hover on "Heading" works during syntax error
    let hover = provide_hover(dirty_source, Position::new(2, 10), Some(&valid_prog));
    assert!(hover.is_some());

    // Completions inside parens during typing
    let completions = provide_completions("Page D {\n  Button(\n}\n", Position::new(1, 9), Some(&valid_prog));
    assert!(completions.iter().any(|c| c.label == "primary"));

    // Step 2: User completes typing: `Button("Save", primary)`
    let fixed_source = "Page Dashboard (path: \"/dashboard\") {\n    Container {\n        Heading(\"Metrics\", h1)\n        Button(\"Save\", primary)\n    }\n}\n";
    let fixed_index = LineIndex::new(fixed_source);
    let fixed_prog = try_parse(fixed_source, "dashboard.wf").expect("parse fixed");

    let updated_syms = build_document_symbols(&fixed_prog, fixed_source, &fixed_index, &uri, true);
    if let DocumentSymbolResponse::Nested(s) = updated_syms {
        let container = &s[0].children.as_ref().unwrap();
        // Container has Heading and Button children
        let inner = &container[0].children.as_ref().unwrap();
        assert!(inner.iter().any(|c| c.name == "Button"));
    }
}

// ---------------------------------------------------------------------------
// 5. Multilingual and Unicode UTF-16 coordinate safety
// ---------------------------------------------------------------------------

#[test]
fn test_multilingual_arabic_and_emoji_coordinates() {
    let source = "Page Welcome (path: \"/\") {\n    Text(\"مرحباً بك في WebFluent! 🚀\")\n    Button(\"ابدأ الآن\", primary)\n}\n";
    let index = LineIndex::new(source);
    let uri = Url::parse("file:///welcome.wf").unwrap();

    let prog = try_parse(source, "welcome.wf").expect("parse multilingual");

    // Check symbols
    let syms = build_document_symbols(&prog, source, &index, &uri, true);
    if let DocumentSymbolResponse::Nested(s) = syms {
        assert_eq!(s[0].name, "Welcome");
    }

    // Line 1 contains Arabic text and a 4-byte UTF-8 emoji (🚀) which is 2 code units in UTF-16
    let line1 = source.lines().nth(1).unwrap();
    let utf8_len = line1.len();
    let utf16_len = line1.encode_utf16().count();
    assert_ne!(utf8_len, utf16_len, "Expected byte len != utf16 len for Arabic & Emoji");

    let end_pos = Position::new(1, utf16_len as u32);
    let offset = index.position_to_offset(source, end_pos).unwrap();
    let roundtrip_pos = index.offset_to_position(source, offset);
    assert_eq!(roundtrip_pos.line, 1);
    assert_eq!(roundtrip_pos.character, utf16_len as u32);

    // Hover on 'Text' and 'Button'
    let hover_text = provide_hover(source, Position::new(1, 4), Some(&prog));
    assert!(hover_text.is_some());

    let hover_btn = provide_hover(source, Position::new(2, 4), Some(&prog));
    assert!(hover_btn.is_some());
}
