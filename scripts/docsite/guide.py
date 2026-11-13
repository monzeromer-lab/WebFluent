#!/usr/bin/env python3
"""Write the documentation site's chapter pages from `md-docs/`.

Each chapter, `md-docs/<kind>/<name>.md`, becomes
`site/src/pages/guide/NN-<slug>.wf` (NN is its `chapter:`) — but for the
chapters whose page reference.py writes (`PROSE_ROUTES`) — a page at
`/docs/<route>` framed by `DocsShell`, with a `Heading` per `##`/`###`,
`Markdown` for prose and lists (so inline code, emphasis and links render),
`CodeBlock` for a fenced block, `Terminal` for a shell block and `Table`
for a pipe table. It also writes the book's chapter list,
`site/src/components/Guide.wf`. The guide is the source: rerun after it
changes, `cargo xtask site`.
"""
import json
import os
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import fences  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
GUIDE = ROOT / "md-docs"
# `WF_GUIDE_OUT` and `WF_BIN` let `tests/docs_parse.rs` run this into a
# scratch directory with the binary it just built, and hold the committed
# pages to the result.
OUT = Path(os.environ.get("WF_GUIDE_OUT") or ROOT / "site" / "src" / "pages" / "guide")
REPO = "https://github.com/monzeromer-lab/WebFluent"

# Each chapter says where it lives and what it is in its front matter — its
# number, its kind, its route, its group in the sidebar, the line under the
# title and the description a search result shows (a quoted value is a JSON
# string):
#
#     ---
#     chapter: 12
#     kind: "tutorial"
#     route: "guide/state"
#     group: "concepts"
#     blurb: "…"
#     description: "…"
#     ---
#
# so a chapter is added by writing it, and nothing here holds a copy of the
# outline to fall behind it.
META = {}  # number → {route, group, blurb, description, slug, title, path}
KINDS = ("tutorial", "recipe", "about", "reference", "error", "coming-from")
# A page's kind as the site's `PageKind` names it: the label above the
# title, and the place in the header it belongs to.
PAGE_KIND = {"tutorial": "tutorial", "recipe": "howto", "about": "explanation",
             "reference": "reference", "error": "error", "coming-from": "comingfrom"}
GROUP_ORDER = []
# The chapter drawn from the registry, not from its Markdown.
REFERENCE_ROUTE = "/reference"
# Chapters whose page `scripts/docsite/reference.py` writes, from the
# compiler's data: it places each one's prose (`prose()`) under its tables.
PROSE_ROUTES = ("/reference/cli", "/reference/config", "/reference/tokens")
# The guide's sections, in order, as the sidebar and the contents name them.
# A chapter's `group:` is one of these; its English name is the site's
# `nav.group.<key>` (the Arabic one is written by hand).
GROUP_NAMES = {
    "tour": "The tour",
    "language": "The language",
    "build": "Building with it",
    "about": "About",
    "reference": "Reference",
    "help": "Help",
}


def front_matter(path: Path) -> tuple[dict, str]:
    """A page's front matter, and the text after it."""
    import json
    text = path.read_text()
    if not text.startswith("---\n") or "\n---\n" not in text[4:]:
        raise SystemExit(f"{path}: no front matter")
    head, body = text[4:].split("\n---\n", 1)
    meta = {}
    for line in head.splitlines():
        key, sep, value = line.partition(":")
        if not sep:
            continue
        value = value.strip()
        meta[key.strip()] = json.loads(value) if value[:1] in '"[' or value.isdigit() else value
    return meta, body


def read_meta(path: Path) -> dict:
    meta, body = front_matter(path)
    rel = path.relative_to(GUIDE).as_posix()
    for key in ("chapter", "kind", "route", "group", "blurb", "description"):
        if key not in meta:
            raise SystemExit(f"md-docs/{rel}: the front matter has no `{key}:`")
    if meta["group"] not in GROUP_NAMES:
        raise SystemExit(f"md-docs/{rel}: `group: {meta['group']}` is none of {tuple(GROUP_NAMES)}")
    if meta["kind"] not in KINDS:
        raise SystemExit(f"md-docs/{rel}: `kind: {meta['kind']}` is none of {KINDS}")
    title_line = next(l for l in body.split("\n") if l.startswith("# "))
    meta["title"] = re.sub(r"^# (\d+\.\s*)?", "", title_line).strip()
    meta["slug"] = re.sub(r"^\d\d-", "", path.stem)
    meta["path"] = rel
    return meta


def chapter_files() -> list[Path]:
    """Every page of the guide that is a chapter, in chapter order. A page
    with no `chapter:` — a code's page under `errors/` — is drawn by its own
    page family, not as a chapter."""
    pages = [p for p in sorted(GUIDE.rglob("*.md")) if p.parent != GUIDE]
    chapters = [p for p in pages if "chapter" in front_matter(p)[0]]
    return sorted(chapters, key=lambda p: front_matter(p)[0]["chapter"])


def href(route: str) -> str:
    """Where a page's `route:` is on the site: as written when it starts
    with `/` (`/errors/T05`), under `/docs/` otherwise (`guide/state`)."""
    return route if route.startswith("/") else f"/docs/{route}"


ROUTES: dict = {}


def route_of(path: Path):
    """The route of any page of the guide, chapter or not."""
    if not ROUTES:
        for p in GUIDE.rglob("*.md"):
            if p.parent != GUIDE:
                meta = front_matter(p)[0]
                if "route" in meta:
                    ROUTES[p.resolve()] = meta["route"]
    return ROUTES.get(path.resolve())


def num_of(path: Path) -> str:
    return f'{front_matter(path)[0]["chapter"]:02d}'


# The same rule under the name the changelog script uses.
site_path = href


def page_name(route: str) -> str:
    parts = route.strip("/").split("/")
    last = parts[-1]
    name = "".join(w.capitalize() for w in re.split(r"[^a-z0-9]+", last) if w)
    # A chapter of the book is `/learn/07-state-and-reactivity`: its name
    # starts with the part of the site it is in, not with a digit.
    if name[:1].isdigit() or parts[0] in ("learn", "about"):
        return parts[0].capitalize() + name
    return name + "Guide"


def folder(num: str) -> str:
    """The part of the guide a page is in: `learn`, `about`, `reference`,
    `errors`. Pages read on, from one to the next, inside their part."""
    return META[num]["path"].split("/")[0]


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


# The page being converted, so a relative link resolves from its folder.
CURRENT = {"dir": GUIDE}


def rewrite_links(text: str) -> str:
    """Links between the guide's pages as site routes."""

    def repl(m):
        label, target = m.group(1), m.group(2)
        anchor = ""
        if "#" in target:
            target, anchor = target.split("#", 1)
            anchor = "#" + anchor
        if target == "":
            return f"[{label}]({anchor})"
        if re.match(r"^[a-z]+:", target) or target.startswith("/"):
            return m.group(0)
        resolved = (CURRENT["dir"] / target).resolve()
        if resolved == (GUIDE / "README.md").resolve():
            return f"[{label}](/docs/guide/introduction)"
        route = route_of(resolved)
        if route:
            return f"[{label}]({href(route)}{anchor})"
        try:
            inside = resolved.relative_to(ROOT)
        except ValueError:
            return m.group(0)
        if not str(inside).startswith("md-docs"):
            return f"[{label}]({REPO}/blob/master/{inside.as_posix()}{anchor})"
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
        # `wf expect=T04`: a program that draws that diagnostic, on purpose.
        fence = fences.parse(f"{lang} {info}")
        if fence.expect:
            title = f"{title} · draws {fence.expect}"
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


def emit_callout(kind, lines, out, ind):
    text = rewrite_links("\n".join(reflow(lines)).strip("\n"))
    out.append(f'{ind}Callout("{wf_string(text)}", kind: .{kind})')
    out.append("")


def emit_quiz(lines, out, ind, qid):
    """A ```` ```quiz ```` block: the question, then a line per choice —
    `+ ` the right one, `- ` the others — each with ` -- ` and why.

        ```quiz
        Which of these ticks every second?
        - state clock = now(every: 1.seconds) -- A state takes `now` once.
        + derived clock = now(every: 1.seconds) -- A derived value follows it.
        ```
    """
    question, choices = [], []
    for line in lines:
        m = re.match(r"^([+-]) (.*?)(?: -- (.*))?$", line)
        if m:
            choices.append((m.group(1) == "+", m.group(2).strip(), (m.group(3) or "").strip()))
        elif line.strip():
            question.append(line.strip())
    if not question or sum(1 for c in choices if c[0]) != 1:
        raise SystemExit(f"a quiz in {qid} needs a question and exactly one `+` choice")
    rows = ",\n".join(
        f'{ind}    QuizChoice(text: "{wf_string(t)}", right: {"true" if r else "false"}, why: "{wf_string(w)}")'
        for r, t, w in choices
    )
    out.append(f'{ind}Quiz("{wf_string(" ".join(question))}", id: "{wf_string(qid)}", choices: [')
    out.append(rows)
    out.append(f"{ind}])")
    out.append("")


def body(md: str, title: str, ind: str = "        "):
    """A page's Markdown as WebFluent elements: the lines of the body, and
    its `##` sections as `(anchor, label)`. `CURRENT["dir"]` must be the
    page's folder, so its links resolve."""
    lines = md.split("\n")
    sections = []
    out = []
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
            if lang == "quiz":
                emit_quiz(code, out, ind, f"{title}#{len(sections)}")
                continue
            emit_code(lang, "\n".join(code), out, ind, info=info)
            continue
        # A GitHub alert — `> [!TIP]` and the quoted lines under it — is a
        # `Callout`: it renders as an alert on GitHub and as one here.
        alert = re.match(r"^>\s*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s*$", line)
        if alert:
            emit_markdown(buf, out, ind)
            buf = []
            i += 1
            quoted = []
            while i < len(lines) and lines[i].startswith(">"):
                quoted.append(re.sub(r"^>\s?", "", lines[i]))
                i += 1
            emit_callout(alert.group(1).lower(), quoted, out, ind)
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
    return out, sections


def convert(num: str, md: str):
    CURRENT["dir"] = (GUIDE / META[num]["path"]).parent
    route = META[num]["route"]
    name = page_name(route)
    title = META[num]["title"]
    TITLES[num] = title
    out, sections = body(md, title)
    blurb, description = META[num]["blurb"], META[num]["description"]
    n = int(num)
    order = [m for m in sorted(META) if folder(m) == folder(num)]
    at = order.index(num)
    prev_num = order[at - 1] if at > 0 else None
    next_num = order[at + 1] if at + 1 < len(order) else None
    anchors = ", ".join(f'Anchor(id: "{sid}", label: "{wf_string(label)}")' for sid, label in sections)
    header = [
        f'/// Chapter {n} of the guide, written from md-docs/{META[num]["path"]} by',
        "/// scripts/docsite/guide.py; edit the guide, not this file.",
        f'page {name}(path: "{href(route)}", title: "{wf_string(title)}", description: "{wf_string(description)}", layout: DocsShell("{wf_string(title)}", kind: .{PAGE_KIND[META[num]["kind"]]}, chapter: t("nav.group.{META[num]["group"]}"), blurb: "{wf_string(blurb)}", source: "md-docs/{META[num]["path"]}", sections: [{anchors}]' + "PREVNEXT" + ")) {",
        "    Stack(gap: .lg) {",
    ]
    return header, out, prev_num, next_num


def learned(num: str) -> str:
    """What a chapter says its reader learned (`learned:` in its front
    matter), as the shell's list under the page."""
    items = META[num].get("learned") or []
    if not items:
        return ""
    return ", learned: [" + ", ".join(f'"{wf_string(i)}"' for i in items) + "]"


BOOK_HEAD = """/// The guide, in the book sidebar's shape: written from each chapter's
/// front matter by scripts/docsite/guide.py; edit the guide, not this file.
/// The first three chapters are the tour.
const GUIDE: [Chapter] = [
"""


def book() -> str:
    rows = []
    for num in sorted(META):
        meta = META[num]
        if folder(num) != "learn":
            continue
        tour = meta["group"] == "tour"
        rows.append(
            f'    Chapter(n: {int(num)}, title: "{wf_string(meta["title"])}", key: "ch.{num}", '
            f'to: "{href(meta["route"])}", part: "nav.group.{meta["group"]}", '
            f'tour: {"true" if tour else "false"})'
        )
    return BOOK_HEAD + ",\n".join(rows) + "\n]\n"


def part_index(part: str) -> str:
    """`/learn` (the book: how far the reader is, then every chapter by its
    part) or `/about` (the essays), from the pages' front matter."""
    nums = [n for n in sorted(META) if folder(n) == part]
    learn = part == "learn"
    name, path = ("Learn", "/learn") if learn else ("About", "/about")
    title = "Learn WebFluent" if learn else "About WebFluent"
    lede = ("The book: seventeen chapters, read in order. The first three are a tour that ends "
            "with a working site and an app; the rest teach the language, then how to build with it."
            if learn else
            "Essays on why WebFluent works as it does: read once, for the argument, not looked up.")
    desc = ("The WebFluent book, from the first page to PDFs and slides." if learn
            else "Why WebFluent works as it does: essays on its design.")
    out = [
        f"/// `{path}`, written from the front matter of md-docs/{part}/ by",
        "/// scripts/docsite/guide.py; edit the pages, not this file.",
        f'page {name}Index(path: "{path}", title: "{title}", description: "{desc}") {{',
        f'    SiteHeader(section: "{path}")',
        '    Stack(gap: .lg, role: "main", id: "bp-main", tabindex: "-1", class: "bp-main bp-wrap") {',
        "        style { gap: 28px; padding-top: 40px; padding-bottom: 72px; @md { padding-top: 56px } }",
        "        Stack(gap: .sm) {",
        "            style { gap: 14px; max-width: 760px }",
        f'            Row(class: "bp-labels") {{ KindLabel(.{"tutorial" if learn else "explanation"}) }}',
        f'            Heading(t("nav.{"learn" if learn else "about"}")).h1 {{ style {{ margin: 0; font-size: clamp(30px, 5vw, 44px); line-height: 1.08; font-weight: 650; letter-spacing: -0.025em }} }}',
        f'            Text("{wf_string(lede)}", class: "bp-lede")',
        "        }",
    ]
    if learn:
        out.append("        BookProgress(chapters: GUIDE)")
    group = None
    for n in nums:
        meta = META[n]
        if meta["group"] != group:
            if group is not None:
                out += ["            }", "        }"]
            group = meta["group"]
            out += [
                "        Stack(gap: .md) {",
                "            style { gap: 14px }",
                f'            Heading(t("nav.group.{group}"), id: "{group}").h2 {{ style {{ margin: 0; font-size: 22px; font-weight: 650 }} }}',
                "            Grid(columns: { base: 1, md: 2 }, gap: .md) {",
            ]
        label = f'{int(n)}. ' if learn else ""
        out += [
            f'                Link(to: "{href(meta["route"])}", class: "bp-card") {{',
            f'                    Text("{label}{wf_string(meta["title"])}") {{ style {{ font-weight: 650 }} }}',
            f'                    Text("{wf_string(meta["blurb"])}", class: "bp-small")',
            "                }",
        ]
    if group is not None:
        out += ["            }", "        }"]
    out += ["    }", "}", ""]
    return "\n".join(out)


def moved() -> tuple[str, str]:
    """A page at each address a chapter used to have (`from:` in its front
    matter), sending the reader to where it is now; and the same pairs for
    `site/src/old-links.js`, which keeps the `#section` of an old link."""
    pairs = {}
    for n in sorted(META):
        for old in META[n].get("from") or []:
            pairs[old] = href(META[n]["route"])
    out = [
        "/// The guide's old addresses: each sends the reader to the page that is",
        "/// there now. Written by scripts/docsite/guide.py from each page's `from:`.",
        "",
    ]
    for old, to in sorted(pairs.items()):
        name = "Was" + "".join(w.capitalize() for w in re.split(r"[^a-z0-9]+", old) if w)
        out += [
            f'page {name}(path: "{old}", title: "Moved", description: "This page moved.", guard: false, redirect: "{to}", noindex: true) {{',
            '    Heading("Moved").h1',
            f'    Link("Go on", to: "{to}")',
            "}",
            "",
        ]
    js = (
        "// Each old address of a page of the guide, and where it is now: written\n"
        "// by scripts/docsite/guide.py, read by old-links.js, which keeps the\n"
        "// `#section` of an old link.\n"
        f"const OLD_PAGES = {json.dumps(dict(sorted(pairs.items())), indent=1)};\n"
    )
    return "\n".join(out), js


def contents_table() -> str:
    """The guide's table of contents, for `md-docs/README.md`."""
    names = GROUP_NAMES
    rows = ["| # | Chapter | What it covers |", "|---|---|---|"]
    group = None
    for num in sorted(META):
        meta = META[num]
        if meta["group"] != group:
            group = meta["group"]
            rows.append(f"| | **{names.get(group, group)}** | |")
        rows.append(f"| {int(num)} | [{meta['title']}]({meta['path']}) | {meta['blurb']} |")
    return "\n".join(rows)


def prose(route: str) -> tuple[list, list]:
    """The chapter at `route` converted, for a page another script writes:
    its sections as `(id, label)` pairs, and its body's lines."""
    if not META:
        for f in chapter_files():
            META[num_of(f)] = read_meta(f)
    num = next(n for n, m in META.items() if m["route"] == route)
    header, body, _, _ = convert(num, front_matter(GUIDE / META[num]["path"])[1])
    sections = re.findall(r'Anchor\(id: "([^"]+)", label: "((?:[^"\\]|\\.)*)"\)', header[2])
    return sections, body


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for old in OUT.glob("*.wf"):
        old.unlink()
    files = chapter_files()
    for f in files:
        META[num_of(f)] = read_meta(f)
    converted = {}
    for f in files:
        num = num_of(f)
        # The components reference is drawn from the registry's own data
        # (reference.py), not from its chapter; the CLI, configuration and
        # tokens chapters are the prose of pages reference.py writes.
        if META[num]["route"] == REFERENCE_ROUTE or META[num]["route"] in PROSE_ROUTES:
            TITLES[num] = META[num]["title"]
            continue
        converted[num] = (f, convert(num, front_matter(f)[1]))
    for num, (f, (header, body, prev_num, next_num)) in converted.items():
        prevnext = ""
        if prev_num:
            prevnext += f', prevTo: "{href(META[prev_num]["route"])}", prevLabel: t("ch.{prev_num}")'
        if next_num:
            prevnext += f', nextTo: "{href(META[next_num]["route"])}", nextLabel: t("ch.{next_num}")'
        header[2] = header[2].replace("PREVNEXT", prevnext)
        text = "\n".join(header + body).rstrip() + "\n    }\n}\n"
        (OUT / f"{num}-{META[num]['slug']}.wf").write_text(text)
    (OUT / "Learn.wf").write_text(part_index("learn"))
    (OUT / "About.wf").write_text(part_index("about"))
    moved_wf, moved_js = moved()
    (OUT / "Moved.wf").write_text(moved_wf)
    (OUT / "old-links-pages.js" if os.environ.get("WF_GUIDE_OUT") else ROOT / "site" / "src" / "old-links-pages.js").write_text(moved_js)
    print(f"wrote {len(converted)} chapters to {OUT}")

    # The book's chapter list, beside the pages when a test asks for a
    # scratch copy.
    testing = bool(os.environ.get("WF_GUIDE_OUT"))
    book_path = (OUT / "book" / "Guide.wf") if testing else ROOT / "site" / "src" / "components" / "Guide.wf"
    book_path.parent.mkdir(parents=True, exist_ok=True)
    book_path.write_text(book())

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
        for key in [k for k in en if k.startswith("nav.group.") and k[10:] not in GROUP_NAMES]:
            del en[key]
        for key, name in GROUP_NAMES.items():
            en[f"nav.group.{key}"] = name
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
    subprocess.run([str(wf) if wf.exists() else "wf", "fmt", str(book_path)], check=False)


if __name__ == "__main__":
    main()
