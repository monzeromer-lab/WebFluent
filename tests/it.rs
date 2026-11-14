//! Every integration test, in one binary: each file of `tests/` is a
//! module of it, so the compiler is linked once rather than once a file.
//! A test is named by its file, `cookbook::the_cookbook_applications_build`,
//! and run as `cargo test --test it -- cookbook::` or
//! `cargo nextest run -E 'test(/^cookbook::/)'`.

mod common;

mod mangled_runtime;

#[path = "agent.rs"]
mod agent;
#[path = "arabic_source.rs"]
mod arabic_source;
#[path = "assets_data.rs"]
mod assets_data;
#[path = "build_and_runtime_agree.rs"]
mod build_and_runtime_agree;
#[path = "build_api.rs"]
mod build_api;
#[path = "build_cache.rs"]
mod build_cache;
#[path = "builtin_attrs.rs"]
mod builtin_attrs;
#[path = "builtin_components.rs"]
mod builtin_components;
#[path = "bundle_size.rs"]
mod bundle_size;
#[path = "check_threads.rs"]
mod check_threads;
#[path = "checks.rs"]
mod checks;
mod choosing;
#[path = "codegen_fixes.rs"]
mod codegen_fixes;
#[path = "codegen_fixes_0b.rs"]
mod codegen_fixes_0b;
mod collections;
#[path = "commands_data.rs"]
mod commands_data;
#[path = "component_root.rs"]
mod component_root;
#[path = "components_data.rs"]
mod components_data;
#[path = "config_edit.rs"]
mod config_edit;
mod content;
#[path = "cookbook.rs"]
mod cookbook;
#[path = "derived_in_a_run.rs"]
mod derived_in_a_run;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "docs_audit_fixes.rs"]
mod docs_audit_fixes;
#[path = "docs_parse.rs"]
mod docs_parse;
#[path = "docs_urls.rs"]
mod docs_urls;
#[path = "e2e_rendered.rs"]
mod e2e_rendered;
#[path = "e2e_sites.rs"]
mod e2e_sites;
#[path = "effect_order.rs"]
mod effect_order;
#[path = "fix_cli.rs"]
mod fix_cli;
#[path = "fixes.rs"]
mod fixes;
#[path = "fmt_corpus.rs"]
mod fmt_corpus;
#[path = "focus.rs"]
mod focus;
mod forms2;
#[path = "frontend_fixes.rs"]
mod frontend_fixes;
#[path = "fs_security.rs"]
mod fs_security;
#[path = "gestures.rs"]
mod gestures;
#[path = "golden.rs"]
mod golden;
#[path = "guide_runs.rs"]
mod guide_runs;
#[path = "handler_values.rs"]
mod handler_values;
#[path = "html_output_hygiene.rs"]
mod html_output_hygiene;
#[path = "in_place.rs"]
mod in_place;
mod interactions;
#[path = "io_errors.rs"]
mod io_errors;
#[path = "js_strings.rs"]
mod js_strings;
#[path = "keep.rs"]
mod keep;
#[path = "keys.rs"]
mod keys;
#[path = "language_3_1.rs"]
mod language_3_1;
#[path = "language_3_2.rs"]
mod language_3_2;
#[path = "language_3_3.rs"]
mod language_3_3;
#[path = "language_4_0.rs"]
mod language_4_0;
#[path = "language_4_2.rs"]
mod language_4_2;
#[path = "layers.rs"]
mod layers;
#[path = "list_folds.rs"]
mod list_folds;
#[path = "markdown.rs"]
mod markdown;
#[path = "migrate_equivalence.rs"]
mod migrate_equivalence;
#[path = "module_layers.rs"]
mod module_layers;
#[path = "motion.rs"]
mod motion;
mod navigation;
#[path = "nodes_corpus.rs"]
mod nodes_corpus;
#[path = "offline.rs"]
mod offline;
#[path = "output_security.rs"]
mod output_security;
#[path = "output_type_static.rs"]
mod output_type_static;
mod overlays;
mod overlays2;
#[path = "page_lang.rs"]
mod page_lang;
#[path = "paged.rs"]
mod paged;
#[path = "peer.rs"]
mod peer;
#[path = "portal.rs"]
mod portal;
#[path = "printer_corpus.rs"]
mod printer_corpus;
#[path = "render_cli.rs"]
mod render_cli;
#[path = "reprint_corpus.rs"]
mod reprint_corpus;
#[path = "resource_limits.rs"]
mod resource_limits;
#[path = "responsive_and_surfaces.rs"]
mod responsive_and_surfaces;
#[path = "runtime_fixes.rs"]
mod runtime_fixes;
#[path = "scope.rs"]
mod scope;
#[path = "serve_and_verify.rs"]
mod serve_and_verify;
#[path = "serve_security.rs"]
mod serve_security;
#[path = "sourcemap.rs"]
mod sourcemap;
#[path = "stale_version.rs"]
mod stale_version;
#[path = "starter_fixes.rs"]
mod starter_fixes;
#[path = "step_script.rs"]
mod step_script;
#[path = "store_method_parity.rs"]
mod store_method_parity;
#[path = "store_resource.rs"]
mod store_resource;
#[path = "style_values.rs"]
mod style_values;
#[path = "template_engine.rs"]
mod template_engine;
#[path = "template_fixes.rs"]
mod template_fixes;
#[path = "theme_boot.rs"]
mod theme_boot;
#[path = "theme_view.rs"]
mod theme_view;
#[path = "tooling_fixes.rs"]
mod tooling_fixes;
#[path = "tools_api.rs"]
mod tools_api;
#[path = "translations.rs"]
mod translations;
#[path = "tutorials.rs"]
mod tutorials;
#[path = "v2_syntax.rs"]
mod v2_syntax;
#[path = "validation.rs"]
mod validation;
#[path = "video_examples.rs"]
mod video_examples;
#[path = "windowed.rs"]
mod windowed;
#[path = "worker.rs"]
mod worker;
