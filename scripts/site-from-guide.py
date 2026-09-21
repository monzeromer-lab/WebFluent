#!/usr/bin/env python3
"""Write the documentation site's chapter pages from `md-docs/`.

Each `md-docs/NN-slug.md` becomes `site/src/pages/guide/NN-slug.wf`: a page
at `/docs/<route>` framed by `DocsShell`, with a `Heading` per `##`/`###`,
`Markdown` for prose and lists (so inline code, emphasis and links render),
`CodeBlock` for a fenced block, `Terminal` for a shell block and `Table`
for a pipe table. The guide is the source: rerun after it changes,
`python3 scripts/site-from-guide.py`.
"""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GUIDE = ROOT / "md-docs"
OUT = ROOT / "site" / "src" / "pages" / "guide"
REPO = "https://github.com/monzeromer-lab/WebFluent"

# Chapter number → route, page name, nav group.
CHAPTERS = {
    "01": ("guide/getting-started", "GettingStarted"),
    "02": ("guide/language-basics", "LanguageBasics"),
    "03": ("guide/pages-and-routing", "PagesAndRouting"),
    "04": ("guide/elements", "Elements"),
    "05": ("guide/state", "StateAndReactivity"),
    "06": ("guide/events-and-forms", "EventsAndForms"),
    "07": ("guide/control-flow", "ControlFlow"),
    "08": ("guide/components", "ComponentsChapter"),
    "09": ("guide/stores", "Stores"),
    "10": ("guide/types", "Types"),
    "11": ("guide/expressions", "Expressions"),
    "12": ("guide/styling", "Styling"),
    "13": ("guide/motion", "Motion"),
    "14": ("guide/data", "DataChapter"),
    "15": ("guide/i18n", "I18nChapter"),
    "16": ("guide/content", "Content"),
    "17": ("guide/outputs", "Outputs"),
    "18": ("guide/tooling", "Tooling"),
    "19": ("reference", "Reference"),
    "20": ("cookbook", "Cookbook"),
}

# The one-line description under each chapter's title, and the sentence the
# search result and the link preview show.
BLURBS = {
    "01": ("Install the compiler, make a project, and see what a build writes.",
           "Install wf, run a first project, and learn the project layout and the config file."),
    "02": ("A file is a list of declarations. This chapter covers the ten kinds, the naming rules, comments, strings and the two layouts.",
           "Declarations, naming, bodies, the braced and the indented layout, and how strings are written."),
    "03": ("A page is a route and what it shows. Pages own their paths; the app shell frames them; a layout wraps them.",
           "Page attributes, routes and parameters, layouts, the app shell, per-page head tags and the 404 page."),
    "04": ("Everything on screen is an element: a call with one positional value, named props, flags, and a block.",
           "The element call shape: positional, named props, flags and enum cases, the block and its order, attributes and parts."),
    "05": ("Reactivity here is fine-grained: a state is a signal, and only the thing that read it updates.",
           "state, derived, effect, actions and let, persist, timers, element handles and the browser's values."),
    "06": ("Handlers, keys, two-way binding, forms with a handle, pending actions, and the overlays a page opens.",
           "Event handlers, keyboard shortcuts, bind:, Form handles, pending actions, emitted events, theme switching and overlays."),
    "07": ("Five statements decide what shows: if, if let, for, show and match. Each creates and removes elements as its condition changes.",
           "if, if let, for … by, show and match, in render blocks and in actions."),
    "08": ("A component is a reusable element you declare: typed props, events, slots, parts and a body, checked like a built-in.",
           "Declaring components: props, enum props, events, slots and scoped slots, parts, local state and stores."),
    "09": ("A store is state shared by every page and component that uses it, alive for the life of the app.",
           "Stores: declaration, use, actions, persistence, loading data, and when to reach for one."),
    "10": ("A gradual, structural type checker that changes no output and names every mistake with a hint.",
           "The type language, records and enums, shapes, inference and narrowing, and every check the compiler makes."),
    "11": ("The full expression language: literals, interpolation, operators, values that choose, calls, formatting and await.",
           "Literals, interpolation, operators, if and match as values, lambdas, methods, format and ago, regexes, tokens, await, const and env."),
    "12": ("Every built-in ships styled; a theme changes the whole site; a style block is real CSS scoped to its element.",
           "Design tokens and themes, style blocks with splices and nesting, transitions, classes, dark mode and layout."),
    "13": ("Motion is declared, not scripted: how an element enters and leaves, how a list staggers, how pages change.",
           "Enter and exit animations, keyed lists and stagger, keyframes, transitions, route transitions and reduced motion."),
    "14": ("Three ways to get data onto a page: resource, await fetch, and data files read at build time.",
           "resource and match, reactive URLs, await fetch in actions, data files, static paths, const and env."),
    "15": ("One JSON file per locale, t() to read it, and a switch that flips the whole site.",
           "Translation files, t() with placeholders and plurals, switching the locale, right-to-left and locale-aware formatting."),
    "16": ("Markdown, .md pages, static builds, and what the build does for search engines and link previews.",
           "The Markdown element, .md pages with front matter, static builds, search and sharing tags, code and media."),
    "17": ("One source, several outputs: a single-page app, a static site, a PDF, a slide deck, or a template rendered with data.",
           "SPA, static site, PDF documents, slide decks, and rendering templates with data from the CLI, Rust and Node."),
    "18": ("One binary does everything: scaffold, build, serve, format, test, render and describe. A language server brings the checks into the editor.",
           "Every wf command, the dev server, diagnostics, the formatter, tests, the gallery and registry, and editor support."),
    "19": ("Every built-in with its props, cases, flags, events, slots and parts — generated from the compiler's own registry.",
           "The components reference: every built-in element, its props, flags, events, slots, parts and attribute families."),
    "20": ("Three complete applications you can paste into a fresh project, and recipes for the things every site needs.",
           "Three complete applications — todos, a static blog, a guarded dashboard — and recipes for search, pagination, forms and more."),
}

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
            route = CHAPTERS[chapter.group(1)][0]
            return f"[{label}](/docs/{route}{anchor})"
        if target == "README.md":
            return f"[{label}](/docs/guide/getting-started)"
        if target.startswith("../"):
            return f"[{label}]({REPO}/blob/master/{target[3:]}{anchor})"
        return m.group(0)

    return re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", repl, text)


def parse_table(lines):
    rows = []
    for line in lines:
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
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


def emit_code(lang, code, out, ind, num=""):
    if lang in ("bash", "sh", "shell"):
        cmds = [l for l in code.split("\n") if l.strip()]
        out.append(f"{ind}Terminal(lines: [")
        out.append(",\n".join(f'{ind}        Line(command: "{wf_string(c)}")' for c in cmds))
        out.append(f"{ind}    ])")
    else:
        title = LANG_TITLES.get(lang, lang)
        # The reference's untitled blocks are each element's call.
        if not title and num == "19":
            title = "call"
        out.append(f'{ind}CodeBlock("{wf_string(code)}", title: "{title}")')
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
    route, page_name = CHAPTERS[num]
    lines = md.split("\n")
    title_line = next(l for l in lines if l.startswith("# "))
    title = re.sub(r"^# (\d+\.\s*)?", "", title_line).strip()
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
        fence = re.match(r"^(`{3,4})\s*(\w*)\s*$", line)
        if fence:
            emit_markdown(buf, out, ind)
            buf = []
            ticks, lang = fence.group(1), fence.group(2)
            i += 1
            code = []
            while i < len(lines) and lines[i].strip() != ticks:
                code.append(lines[i])
                i += 1
            i += 1
            emit_code(lang, "\n".join(code), out, ind, num)
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

    blurb, description = BLURBS[num]
    n = int(num)
    prev_num = f"{n - 1:02}" if n > 1 else None
    next_num = f"{n + 1:02}" if n < 20 else None
    anchors = ", ".join(f'Anchor(id: "{sid}", label: "{wf_string(label)}")' for sid, label in sections)
    header = [
        f'/// Chapter {n} of the guide, written from md-docs/{num}-{{slug}}.md by',
        "/// scripts/site-from-guide.py; edit the guide, not this file.",
        f'page {page_name}(path: "/docs/{route}", title: "{wf_string(title)}", description: "{wf_string(description)}", layout: DocsShell("{wf_string(title)}", chapter: "Chapter {n}", blurb: "{wf_string(blurb)}", sections: [{anchors}]' + "PREVNEXT" + ")) {",
        "    Stack(gap: .lg) {",
    ]
    return header, out, prev_num, next_num


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for old in OUT.glob("*.wf"):
        old.unlink()
    files = sorted(GUIDE.glob("[0-9][0-9]-*.md"))
    converted = {}
    for f in files:
        num = f.name[:2]
        # The components reference is drawn from the registry's own data,
        # `site/src/registry.json`, not from its chapter.
        if num == "19":
            TITLES[num] = "Components reference"
            continue
        converted[num] = (f, convert(num, f.read_text()))
    for num, (f, (header, body, prev_num, next_num)) in converted.items():
        prevnext = ""
        if prev_num:
            prevnext += f', prevTo: "/docs/{CHAPTERS[prev_num][0]}", prevLabel: "{int(prev_num)}. {wf_string(TITLES[prev_num])}"'
        if next_num:
            prevnext += f', nextTo: "/docs/{CHAPTERS[next_num][0]}", nextLabel: "{int(next_num)}. {wf_string(TITLES[next_num])}"'
        header[0] = header[0].replace("{slug}", f.name[3:-3])
        header[2] = header[2].replace("PREVNEXT", prevnext)
        text = "\n".join(header + body).rstrip() + "\n    }\n}\n"
        (OUT / f"{f.name[:-3]}.wf").write_text(text)
    print(f"wrote {len(converted)} chapters to {OUT.relative_to(ROOT)}")
    # The site's files are held to the formatter by tests/fmt_corpus.rs.
    import subprocess
    wf = ROOT / "target" / "debug" / "wf"
    subprocess.run([str(wf) if wf.exists() else "wf", "fmt", str(OUT)], check=False)


if __name__ == "__main__":
    main()
