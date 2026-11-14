//! The navigation built-ins: the ones drawn as links (`PrevNext`,
//! `BottomNavigation`) are painted as such, a `NavigationMenu`'s entries are
//! buttons and the panels they disclose, and a `Menubar` and a `ContextMenu`
//! are menus the runtime opens. What a real browser does with them is
//! `tests/browser/navigation.mjs`.

use crate::common::{page, spa_generated, ssg_html};
use webfluent::diagnostics::Diagnostic;

/// Every finding for `src`, through the build's pipeline.
fn findings(src: &str) -> Vec<Diagnostic> {
    let (program, mut out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}");
    let source = src.to_string();
    out.extend(
        webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
            program: &program,
            file_of: &|_| "t.wf".to_string(),
            source_of: &|_| Some(source.clone()),
            dir: None,
            config: None,
            declaration_files: &[],
            scripts: &[],
            stylesheets: "",
            incomplete: false,
        })
        .diagnostics,
    );
    out
}

/// The findings with `code`, as `line: message`.
fn with(src: &str, code: &str) -> Vec<String> {
    findings(src)
        .into_iter()
        .filter(|d| d.code == code)
        .map(|d| format!("{}: {}", d.line, d.message))
        .collect()
}

#[test]
fn prev_next_and_a_tab_bar_are_links_in_the_static_paint() {
    let html = ssg_html(&page(
        "PrevNext {\n        PrevNext.Prev(\"Forms\", to: \"/\")\n        PrevNext.Next(\"Stores\", to: \"/\", label: \"Then\")\n    }\n    BottomNavigation {\n        BottomNavigation.Item(\"Home\", to: \"/\", icon: \"home\")\n    }",
    ));
    assert!(
        html.contains("<nav class=\"wf-prev-next\" aria-label=\"Pages\">"),
        "{html}"
    );
    assert!(html.contains("rel=\"prev\" data-dir=\"prev\""), "{html}");
    assert!(html.contains(">Then</p>"), "{html}");
    assert!(
        html.contains("<nav class=\"wf-bottom-nav\" aria-label=\"Main\">"),
        "{html}"
    );
    assert!(html.contains("data-icon=\"home\""), "{html}");
}

#[test]
fn a_navigation_menu_discloses_panels() {
    let src = page(
        "NavigationMenu {\n        NavigationMenu.Item(\"Products\") { Text(\"Inside\") }\n        NavigationMenu.Link(\"Pricing\", to: \"/\")\n    }",
    );
    let html = ssg_html(&src);
    assert!(
        html.contains("<button class=\"wf-nav-menu__trigger\" aria-expanded=\"false\">"),
        "{html}"
    );
    assert!(
        html.contains("<div class=\"wf-nav-menu__panel\" hidden>"),
        "{html}"
    );
    assert!(html.contains(">Pricing</a>"), "{html}");
    assert!(spa_generated(&src).contains("WF.navMenu(n)"));
}

#[test]
fn a_menubar_and_a_context_menu_are_menus() {
    let src = page(
        "state heard = \"\"\n    Menubar(label: \"App\") {\n        Menubar.Menu(\"File\") { Menu.Item { on click { heard = \"new\" }  Text(\"New\") } }\n    }\n    ContextMenu {\n        Text(\"Here\")\n        ContextMenu.Items { Menu.Item { on click { heard = \"x\" }  Text(\"Rename\") } }\n    }\n    Text(heard)",
    );
    let js = spa_generated(&src);
    for call in [
        "WF.menuBar(",
        "role: \"menuitem\"",
        "WF.contextMenu(",
        "role: \"menubar\"",
    ] {
        assert!(js.contains(call), "{call}: {js}");
    }
    let html = ssg_html(&src);
    assert!(
        html.contains("role=\"menubar\" aria-label=\"App\""),
        "{html}"
    );
    assert!(html.contains("tabindex=\"0\">File</button>"), "{html}");
    assert!(html.contains("role=\"menu\" hidden>"), "{html}");
}

#[test]
fn a_menu_item_in_a_menubar_or_a_context_menu_is_inside_a_menu() {
    let found = with(
        &page(
            "Menubar { Menubar.Menu(\"File\") { Menu.Item { Text(\"New\") } } }\n    ContextMenu { Text(\"x\")  ContextMenu.Items { Menu.Divider } }",
        ),
        "C03",
    );
    assert!(found.is_empty(), "{found:?}");
}
