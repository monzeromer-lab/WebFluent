#!/usr/bin/env python3
"""Write the documentation site's chapter pages from `md-docs/`.

Each `md-docs/NN-slug.md` becomes `site/src/pages/guide/NN-slug.wf`: a page
at `/docs/<route>` framed by `DocsShell`, with a `Heading` per `##`/`###`,
`Markdown` for prose and lists (so inline code, emphasis and links render),
`CodeBlock` for a fenced block, `Terminal` for a shell block and `Table`
for a pipe table. The guide is the source: rerun after it changes,
`python3 scripts/site-from-guide.py`.
"""
import os
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GUIDE = ROOT / "md-docs"
# `WF_GUIDE_OUT` and `WF_BIN` let `tests/docs_parse.rs` run this into a
# scratch directory with the binary it just built, and hold the committed
# pages to the result.
OUT = Path(os.environ.get("WF_GUIDE_OUT") or ROOT / "site" / "src" / "pages" / "guide")
REPO = "https://github.com/monzeromer-lab/WebFluent"

# Each chapter says where it lives and what it is, in a comment under its
# title — its route, its group in the sidebar, the line under the title and
# the description a search result shows:
#
#     <!--
#     route: guide/state
#     group: basics
#     blurb: …
#     description: …
#     -->
#
# so a chapter is added by writing it, and nothing here holds a copy of the
# outline to fall behind it.
META = {}  # number → {route, group, blurb, description, slug, title}
GROUP_ORDER = []
# The chapter drawn from the registry, not from its Markdown.
REFERENCE_ROUTE = "reference"


def read_meta(num: str, path: Path) -> dict:
    text = path.read_text()
    m = re.search(r"<!--\s*\n(.*?)-->", text, re.S)
    if not m:
        raise SystemExit(f"{path.name}: no metadata comment under the title")
    meta = {}
    for line in m.group(1).splitlines():
        if ":" in line:
            key, _, value = line.partition(":")
            meta[key.strip()] = value.strip()
    for key in ("route", "group", "blurb", "description"):
        if key not in meta:
            raise SystemExit(f"{path.name}: the metadata has no `{key}:`")
    title_line = next(l for l in text.split("\n") if l.startswith("# "))
    meta["title"] = re.sub(r"^# (\d+\.\s*)?", "", title_line).strip()
    meta["slug"] = path.name[3:-3]
    return meta


def page_name(route: str) -> str:
    last = route.rstrip("/").split("/")[-1]
    return "".join(w.capitalize() for w in re.split(r"[^a-z0-9]+", last) if w) + "Guide"


TITLES = {}


def wf_string(text: str) -> str:
    """`text` as the inside of a WebFluent string literal."""
    return (
        text.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("{", "\\{")
        .replace("}", "\\}")
        .replace("\t", "    ")
        .replace("\n", "\\n")
    )


def wf_raw(text: str) -> str:
    """`text` as a whole WebFluent raw string: `#"…"#`, with as many hashes
    as it takes for the text not to close it.

    A sample of code is written once, not twice — no backslash before every
    quote and every brace, so what the page shows is what this file holds."""
    text = text.replace("\t", "    ")
    hashes = 1
    while '"' + "#" * hashes in text:
        hashes += 1
    fence = "#" * hashes
    # A raw string cannot end in a quote: nothing would separate it from
    # the delimiter. Those few go back to the escaped spelling.
    if text.endswith('"'):
        return '"' + wf_string(text) + '"'
    return f'{fence}"{text}"{fence}"'[:-1]


def slug(text: str) -> str:
    text = re.sub(r"`", "", text)
    text = re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")
    return text or "section"


def plain(text: str) -> str:
    """Heading text without its Markdown marks."""
    return re.sub(r"[`*_]", "", text).strip()


def rewrite_links(text: str) -> str:
    """Guide-relative links as site routes."""

    def repl(m):
        label, target = m.group(1), m.group(2)
        anchor = ""
        if "#" in target:
            target, anchor = target.split("#", 1)
            anchor = "#" + anchor
        if target == "":
            return f"[{label}]({anchor})"
        chapter = re.match(r"(\d\d)-[a-z0-9-]+\.md$", target)
        if chapter:
            route = META[chapter.group(1)]["route"]
            return f"[{label}](/docs/{route}{anchor})"
        if target == "README.md":
            return f"[{label}](/docs/guide/introduction)"
        if target.startswith("../"):
            return f"[{label}]({REPO}/blob/master/{target[3:]}{anchor})"
        return m.group(0)

    return re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", repl, text)


def parse_table(lines):
    rows = []
    for line in lines:
        # A `\|` is a pipe inside a cell, not a column edge.
        cells = [c.strip() for c in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
        rows.append(cells)
    head = rows[0]
    body = [r for r in rows[2:]]
    return head, body


def cell(text: str, header: bool) -> str:
    text = rewrite_links(text)
    text = text.replace("\\|", "|")
    flag = ".header" if header else ""
    if re.search(r"[`*\[]", text):
        return f'Table.Cell{flag} {{ Markdown("{wf_string(text)}") }}'
    return f'Table.Cell("{wf_string(text)}"){flag}'


def emit_table(head, body, caption, out, ind):
    out.append(f'{ind}Table(caption: "{wf_string(caption)}") {{')
    out.append(f"{ind}    Table.Head {{")
    out.append(f"{ind}        Table.Row {{")
    for c in head:
        out.append(f"{ind}            {cell(c, True)}")
    out.append(f"{ind}        }}")
    out.append(f"{ind}    }}")
    out.append(f"{ind}    Table.Body {{")
    for r in body:
        out.append(f"{ind}        Table.Row {{")
        for c in r + [""] * (len(head) - len(r)):
            out.append(f"{ind}            {cell(c, False)}")
        out.append(f"{ind}        }}")
    out.append(f"{ind}    }}")
    out.append(f"{ind}}}")
    out.append("")


LANG_TITLES = {"wf": "wf", "wfx": "wfx", "json": "json", "rust": "rust", "js": "js",
               "md": "markdown", "css": "css", "toml": "toml", "html": "html", "": ""}


def emit_code(lang, code, out, ind, num="", info=""):
    if lang in ("bash", "sh", "shell"):
        cmds = [l for l in code.split("\n") if l.strip()]
        out.append(f"{ind}Terminal(lines: [")
        out.append(",\n".join(f'{ind}        Line(command: "{wf_string(c)}")' for c in cmds))
        out.append(f"{ind}    ])")
    else:
        title = LANG_TITLES.get(lang, lang)
        # `wf expect T04`: a program that draws that diagnostic, on purpose.
        expect = re.match(r"expect\s+(\w+)", info)
        if expect:
            title = f"{title} · draws {expect.group(1)}"
        out.append(f"{ind}CodeBlock({wf_raw(code)}, title: \"{title}\")")
    out.append("")


def reflow(lines):
    """The guide is hard-wrapped; the renderer keeps a newline as a break,
    so a paragraph's lines are joined, and a list item's continuation
    lines with it."""
    out = []
    for line in lines:
        stripped = line.strip()
        if not stripped:
            out.append("")
            continue
        is_item = re.match(r"^(\s*)([-*]|\d+\.)\s", line)
        if out and out[-1] and not is_item and not out[-1].startswith("#"):
            out[-1] = out[-1] + " " + stripped
        else:
            out.append(line.rstrip())
    return out


def emit_markdown(lines, out, ind):
    text = "\n".join(reflow(lines)).strip("\n")
    if not text.strip():
        return
    text = rewrite_links(text)
    out.append(f'{ind}Markdown("{wf_string(text)}")')
    out.append("")


def convert(num: str, md: str):
    route = META[num]["route"]
    name = page_name(route)
    lines = md.split("\n")
    title = META[num]["title"]
    TITLES[num] = title
    sections = []
    out = []
    ind = "        "
    buf = []
    i = 0
    caption = title
    skipping = False
    while i < len(lines):
        line = lines[i]
        if line.startswith("## "):
            emit_markdown(buf, out, ind)
            buf = []
            text = plain(line[3:])
            if text in ("Next", "Contents"):
                skipping = True
                i += 1
                continue
            skipping = False
            sid = slug(text)
            sections.append((sid, text))
            caption = text
            out.append(f'{ind}Heading("{wf_string(text)}", id: "{sid}").h2')
            out.append("")
            i += 1
            continue
        if skipping:
            i += 1
            continue
        if line.startswith("### "):
            emit_markdown(buf, out, ind)
            buf = []
            text = plain(line[4:])
            caption = text
            out.append(f'{ind}Heading("{wf_string(text)}", id: "{slug(text)}").h3')
            out.append("")
            i += 1
            continue
        if line.startswith("# "):
            i += 1
            continue
        # The metadata comment and markers such as `<!-- tokens -->`.
        if line.strip().startswith("<!--"):
            while i < len(lines) and "-->" not in lines[i]:
                i += 1
            i += 1
            continue
        fence = re.match(r"^(`{3,4})\s*([\w-]*)(?:\s+(.*?))?\s*$", line)
        if fence:
            emit_markdown(buf, out, ind)
            buf = []
            ticks, lang, info = fence.group(1), fence.group(2), fence.group(3) or ""
            i += 1
            code = []
            while i < len(lines) and lines[i].strip() != ticks:
                code.append(lines[i])
                i += 1
            i += 1
            emit_code(lang, "\n".join(code), out, ind, num, info)
            continue
        if line.startswith("|") and i + 1 < len(lines) and re.match(r"^\|[\s:|-]+\|?\s*$", lines[i + 1]):
            emit_markdown(buf, out, ind)
            buf = []
            table = []
            while i < len(lines) and lines[i].startswith("|"):
                table.append(lines[i])
                i += 1
            head, body = parse_table(table)
            emit_table(head, body, caption, out, ind)
            continue
        buf.append(line)
        i += 1
    emit_markdown(buf, out, ind)

    blurb, description = META[num]["blurb"], META[num]["description"]
    n = int(num)
    order = sorted(META)
    at = order.index(num)
    prev_num = order[at - 1] if at > 0 else None
    next_num = order[at + 1] if at + 1 < len(order) else None
    anchors = ", ".join(f'Anchor(id: "{sid}", label: "{wf_string(label)}")' for sid, label in sections)
    header = [
        f'/// Chapter {n} of the guide, written from md-docs/{num}-{{slug}}.md by',
        "/// scripts/site-from-guide.py; edit the guide, not this file.",
        f'page {name}(path: "/docs/{route}", title: "{wf_string(title)}", description: "{wf_string(description)}", layout: DocsShell("{wf_string(title)}", chapter: t("guide.chapter", {{ n: "{n}" }}), blurb: "{wf_string(blurb)}", source: "md-docs/{num}-{META[num]["slug"]}.md", sections: [{anchors}]' + "PREVNEXT" + ")) {",
        "    Stack(gap: .lg) {",
    ]
    return header, out, prev_num, next_num


SIDEBAR_HEAD = """/// The guide's chapter nav, written from the chapters' own metadata by
/// scripts/site-from-guide.py; edit the guide, not this file. 280px, a group
/// per part of the guide under an overline label, each chapter with its
/// number in mono; the one whose `to` matches the route carries the rail,
/// the wash and the bold.
component DocSidebar {
    Sidebar(aria-label: t("nav.chapters")) {
        style {
            width: 280px
            flex-shrink: 0
            padding: 24px 16px
            gap: 18px
            border-inline-end: 1px solid $border
            background: $color-background
            overflow-y: auto
            & .wf-sidebar__item { height: 32px; padding: 0 10px; gap: 10px; border-radius: 4px; font-size: 13px; line-height: 20px; color: $color-text-muted }
            & .wf-sidebar__item:hover { background: $surface-hover; color: $color-text }
            & .wf-sidebar__item.active { padding-inline-start: 7px; border-inline-start: 3px solid $color-primary; border-radius: 0 4px 4px 0; background: $brand-subtle; color: $color-text; font-weight: 600 }
            & .wf-sidebar__item .num { margin: 0; width: 16px; flex-shrink: 0; font-family: $font-family-mono; font-size: 12px; color: $ink-subtle }
            & .wf-sidebar__item.active .num { color: $brand-ink }
            & .wf-sidebar__item p { margin: 0 }
            & .wf-sidebar__divider { display: none }
            & .group-label { margin: 8px 0 4px; padding: 0 10px; font-size: 12px; line-height: 18px; font-weight: 600; letter-spacing: 0.08em; text-transform: uppercase; color: $ink-subtle }
            & .group-label:first-child { margin-top: 0 }
        }
"""


def sidebar() -> str:
    out = [SIDEBAR_HEAD]
    group = None
    for num in sorted(META):
        meta = META[num]
        if meta["group"] != group:
            group = meta["group"]
            out.append("")
            out.append(f'        Text(t("nav.group.{group}"), class: "group-label")')
        prefix = ", active: .prefix" if meta["route"] == REFERENCE_ROUTE else ""
        out.append(
            f'        Sidebar.Item(to: "/docs/{meta["route"]}"{prefix}) {{ Text("{num}", class: "num")  Text(t("ch.{num}")) }}'
        )
    out.append("    }")
    out.append("}")
    return "\n".join(out) + "\n"


def contents_table() -> str:
    """The guide's table of contents, for `md-docs/README.md`."""
    names = {"start": "Start", "basics": "The basics", "building": "Building", "shipping": "Shipping", "reference": "Reference", "help": "Help"}
    rows = ["| # | Chapter | What it covers |", "|---|---|---|"]
    group = None
    for num in sorted(META):
        meta = META[num]
        if meta["group"] != group:
            group = meta["group"]
            rows.append(f"| | **{names.get(group, group)}** | |")
        rows.append(f"| {int(num)} | [{meta['title']}]({num}-{meta['slug']}.md) | {meta['blurb']} |")
    return "\n".join(rows)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for old in OUT.glob("*.wf"):
        old.unlink()
    files = sorted(GUIDE.glob("[0-9][0-9]-*.md"))
    for f in files:
        META[f.name[:2]] = read_meta(f.name[:2], f)
    converted = {}
    for f in files:
        num = f.name[:2]
        # The components reference is drawn from the registry's own data,
        # `site/src/registry.json`, not from its chapter.
        if META[num]["route"] == REFERENCE_ROUTE:
            TITLES[num] = META[num]["title"]
            continue
        converted[num] = (f, convert(num, f.read_text()))
    for num, (f, (header, body, prev_num, next_num)) in converted.items():
        prevnext = ""
        if prev_num:
            prevnext += f', prevTo: "/docs/{META[prev_num]["route"]}", prevLabel: "{int(prev_num)}. {{t("ch.{prev_num}")}}"'
        if next_num:
            prevnext += f', nextTo: "/docs/{META[next_num]["route"]}", nextLabel: "{int(next_num)}. {{t("ch.{next_num}")}}"'
        header[0] = header[0].replace("{slug}", f.name[3:-3])
        header[2] = header[2].replace("PREVNEXT", prevnext)
        text = "\n".join(header + body).rstrip() + "\n    }\n}\n"
        (OUT / f"{f.name[:-3]}.wf").write_text(text)
    print(f"wrote {len(converted)} chapters to {OUT}")

    # The sidebar, beside the pages when a test asks for a scratch copy.
    testing = bool(os.environ.get("WF_GUIDE_OUT"))
    sidebar_path = (OUT / "sidebar" / "DocSidebar.wf") if testing else ROOT / "site" / "src" / "components" / "DocSidebar.wf"
    sidebar_path.parent.mkdir(parents=True, exist_ok=True)
    sidebar_path.write_text(sidebar())

    if not testing:
        # The chapter titles the site shows in English, and the guide's own
        # table of contents. Arabic titles are written by hand; the test
        # suite checks every chapter has one.
        import json
        en_path = ROOT / "site" / "src" / "translations" / "en.json"
        en = json.loads(en_path.read_text())
        for key in [k for k in en if k.startswith("ch.")]:
            del en[key]
        for num in sorted(META):
            en[f"ch.{num}"] = META[num]["title"]
        en_path.write_text(json.dumps(en, ensure_ascii=False, indent=2) + "\n")
        readme = GUIDE / "README.md"
        text = readme.read_text()
        open_mark, close_mark = "<!-- contents -->", "<!-- /contents -->"
        if open_mark in text and close_mark in text:
            a = text.index(open_mark) + len(open_mark)
            b = text.index(close_mark)
            readme.write_text(text[:a] + "\n" + contents_table() + "\n" + text[b:])

    # The site's files are held to the formatter by tests/fmt_corpus.rs.
    import subprocess
    wf = Path(os.environ["WF_BIN"]) if os.environ.get("WF_BIN") else ROOT / "target" / "debug" / "wf"
    subprocess.run([str(wf) if wf.exists() else "wf", "fmt", str(OUT)], check=False)
    subprocess.run([str(wf) if wf.exists() else "wf", "fmt", str(sidebar_path)], check=False)


if __name__ == "__main__":
    main()
