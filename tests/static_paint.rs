//! What the runtime used to draw alone, painted by the build where it can
//! read what is shown: a page's headings listed in its `OnThisPage`, the
//! rows of a `DataTable` and a `Tree`, a rating's stars, a ring's fill, an
//! avatar group's count, and the controls that hold a value.

use crate::common::{page, ssg_html};

#[test]
fn on_this_page_lists_the_headings_and_gives_them_ids() {
    let html = ssg_html(&page(
        "Heading(\"Guide\").h1\n    OnThisPage(levels: 2)\n    Heading(\"First steps\").h2\n    Heading(\"Details & more\").h3\n    Heading(\"First steps\").h2",
    ));
    for want in [
        "<h2 class=\"wf-heading\" id=\"first-steps\">",
        "<h3 class=\"wf-heading\" id=\"details-more\">",
        "<h2 class=\"wf-heading\" id=\"first-steps-2\">",
        "<a href=\"#first-steps\" class=\"wf-on-this-page__link\" data-level=\"2\">First steps</a>",
        "<a href=\"#details-more\" class=\"wf-on-this-page__link\" data-level=\"3\">Details &amp; more</a>",
    ] {
        assert!(html.contains(want), "{want}: {html}");
    }
}

#[test]
fn a_table_and_a_tree_of_known_rows_are_painted() {
    let html = ssg_html(&page(
        "state rows = [{ id: \"1\", name: \"web\" }, { id: \"2\", name: \"api\" }]\n    state files = [{ path: \"src\", children: [{ path: \"src/a.wf\", children: [] }] }]\n    DataTable(rows: rows, key: r => r.id, label: \"Services\") {\n        DataTable.Column(\"Name\", value: r => r.name).sortable\n        DataTable.Column(\"Id\") { cell(r) { Badge(r.id) } }\n    }\n    Tree(items: files, children: f => f.children, key: f => f.path, label: \"Files\", open: [\"src\"]) { row(f) { Text(f.path) } }",
    ));
    for want in [
        "<caption class=\"wf-visually-hidden\">Services</caption>",
        "<th scope=\"col\" aria-sort=\"none\"><button type=\"button\" class=\"wf-data-table__sort\">Name</button></th>",
        "<td>web</td>",
        "<span class=\"wf-badge\">2</span>",
        "<ul class=\"wf-tree\" role=\"tree\" aria-label=\"Files\">",
        "aria-level=\"1\" aria-setsize=\"1\" aria-posinset=\"1\" aria-expanded=\"true\"",
        "aria-level=\"2\"",
        ">src/a.wf</p>",
    ] {
        assert!(html.contains(want), "{want}: {html}");
    }
}

#[test]
fn a_rating_a_ring_and_an_avatar_group_are_painted() {
    let html = ssg_html(&page(
        "state stars = 3.5\n    state done = 40\n    Rating(bind: stars, label: \"Score\").half\n    CircularProgress(value: done, label: \"Upload\")\n    AvatarGroup(max: 2) {\n        for n in [\"A\", \"B\", \"C\", \"D\"] { Avatar(initials: n) }\n    }\n    Button(\"x\") { on click { done = 50  stars = 4 } }",
    ));
    for want in [
        "role=\"img\" aria-label=\"Score: 3.5 out of 5\"",
        "data-fill=\"half\"",
        "aria-valuenow=\"40\"",
        "stroke-dasharray=\"40 100\"",
        "aria-label=\"2 more\">+2</span>",
    ] {
        assert!(html.contains(want), "{want}: {html}");
    }
}

#[test]
fn the_controls_that_hold_a_value_are_painted_with_it() {
    let html = ssg_html(&page(
        "state name = \"Atlas\"\n    state code = \"12\"\n    state tags: [String] = [\"rust\"]\n    Editable(bind: name, label: \"Name\")\n    InputOTP(bind: code, length: 4, label: \"Code\")\n    TagInput(bind: tags, label: \"Tags\")",
    ));
    for want in [
        "aria-label=\"Name: Atlas, edit\">Atlas</button>",
        "aria-label=\"Digit 2 of 4\" value=\"2\">",
        "aria-label=\"Digit 3 of 4\">",
        "<span class=\"wf-chip__label\">rust</span>",
    ] {
        assert!(html.contains(want), "{want}: {html}");
    }
}
