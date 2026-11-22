#!/usr/bin/env python3
"""Write the site's changelog from `RELEASE_NOTES.md`: the releases as
data, `site/src/data/changelog.json`, and the page that draws them,
`site/src/pages/Changelog.wf`, one call per release with its notes written
in place. A `data` file is part of every page's `app.js`; written into the
page, the notes are in that page's chunk alone. The components the page
places are `site/src/components/Changelog.wf`.

Each release is `# WebFluent vX Release Notes`, a summary, then `##`
sections. A section is what comes before its list (`intro`), its top-level
bullets (`items`), and whatever follows the list (`after`), in that order,
so nothing in the notes is dropped or moved. An item is its text, the
bullets nested under it (`points`) and anything after them, since the
site's `Markdown` draws one level of list.

Anchors are stable: a release is `#<version>` as its heading writes it
(`/changelog#6.0.0`), a section `#<version>-<slug>`. A release's date is
its tag's (`v6.0.0`, or `v3.1.0` for `v3.1`); with no tag to read — a
shallow clone — the date already in the committed file is kept.
`WF_CHANGELOG_OUT` names a directory to write both files into instead, for
`--check`.
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import sitepages  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
NOTES = ROOT / "RELEASE_NOTES.md"
COMMITTED = ROOT / "site" / "src" / "data" / "changelog.json"
PAGE = ROOT / "site" / "src" / "pages" / "Changelog.wf"
SCRATCH = os.environ.get("WF_CHANGELOG_OUT")
OUT = Path(SCRATCH) / "changelog.json" if SCRATCH else COMMITTED
PAGE_OUT = Path(SCRATCH) / "Changelog.wf" if SCRATCH else PAGE
REPO = "https://github.com/monzeromer-lab/WebFluent"
MONTHS = ["January", "February", "March", "April", "May", "June", "July",
          "August", "September", "October", "November", "December"]


# Where each page of the guide that release notes link to by its file is
# published now. The guide was written as Markdown under `md-docs/` until
# 7.1, and older notes name those files; the site's pages are the guide
# since, and a page's address is its `path:`.
OLD_PAGES = {
    "md-docs/learn/01-getting-started.md": "/learn/01-getting-started",
    "md-docs/learn/02-tutorial-shelf.md": "/learn/02-tutorial-shelf",
    "md-docs/learn/03-tutorial-todos.md": "/learn/03-tutorial-todos",
    "md-docs/learn/04-language-basics.md": "/learn/04-language-basics",
    "md-docs/learn/05-pages-and-routing.md": "/learn/05-pages-and-routing",
    "md-docs/learn/06-elements.md": "/learn/06-elements",
    "md-docs/learn/07-state-and-reactivity.md": "/learn/07-state-and-reactivity",
    "md-docs/learn/08-events.md": "/learn/08-events",
    "md-docs/learn/09-control-flow.md": "/learn/09-control-flow",
    "md-docs/learn/10-components.md": "/learn/10-components",
    "md-docs/learn/11-stores.md": "/learn/11-stores",
    "md-docs/learn/12-types.md": "/learn/12-types",
    "md-docs/learn/13-styling.md": "/learn/13-styling",
    "md-docs/learn/14-forms.md": "/learn/14-forms",
    "md-docs/learn/15-data.md": "/learn/15-data",
    "md-docs/learn/16-testing.md": "/learn/16-testing",
    "md-docs/learn/17-pdf-and-slides.md": "/learn/17-pdf-and-slides",
    "md-docs/about/introduction.md": "/about/why",
    "md-docs/about/how-reactivity-works.md": "/about/how-reactivity-works",
    "md-docs/about/accessibility.md": "/about/accessibility",
    "md-docs/about/security.md": "/about/security",
    "md-docs/about/performance.md": "/about/what-a-build-ships",
    "md-docs/about/static-and-spa.md": "/about/static-or-single-page",
    "md-docs/about/contributing.md": "/about/contributing",
    "md-docs/reference/expressions.md": "/reference/expressions",
    "md-docs/reference/project-structure.md": "/reference/project",
    "md-docs/reference/components.md": "/reference",
    "md-docs/reference/cli.md": "/reference/cli",
    "md-docs/reference/configuration.md": "/reference/config",
    "md-docs/reference/built-ins.md": "/reference/built-ins",
    "md-docs/reference/design-tokens.md": "/reference/tokens",
    "md-docs/reference/runtime-api.md": "/reference/runtime",
    "md-docs/reference/grammar.md": "/reference/grammar",
    "md-docs/reference/browser-support.md": "/reference/browser-support",
    "md-docs/reference/glossary.md": "/reference/glossary",
    "md-docs/reference/library.md": "/reference/library",
    "md-docs/reference/editors.md": "/reference/editors",
    "md-docs/errors/diagnostics.md": "/errors",
    "md-docs/errors/troubleshooting.md": "/errors/troubleshooting",
    "md-docs/errors/upgrading.md": "/errors/upgrading",
    "md-docs/reference/requests.md": "/reference/requests",
    "md-docs/README.md": "/docs/guide/introduction",
}


def routes() -> dict:
    """Each guide page's file as older notes name it, and where the site has it."""
    return dict(OLD_PAGES)


HEADS = {}


def heads_of(route: str) -> list:
    """The anchors of a page's headings, `##` to `####`, in order: from its
    Markdown twin (`twins.py`), which writes the page's words as it shows
    them."""
    if not HEADS:
        import twins
        for r, (_, _, md, _, _) in twins.build().pages.items():
            HEADS[r] = [sitepages.slug(m) for m in re.findall(r"^#{2,4}\s+(.+?)\s*$", md, flags=re.M)]
    return HEADS.get(route)


ROUTES = {}


def place(path: str, route: str, anchor: str) -> str:
    """Where a link to a section of a guide page lands today. A release's
    notes name the section as it was then: a heading since renamed or moved
    is found by the words it shares with the old anchor, and a built-in of
    the components reference is its own page now. With nothing close, the
    page's top."""
    if not anchor:
        return route
    if path.endswith("reference/components.md") and (ROOT / "site" / "public" / "reference" / f"{anchor}.md").exists():
        return f"/reference/{anchor}"
    heads = heads_of(route)
    if heads is None:
        return f"{route}#{anchor}"
    if anchor in heads:
        return f"{route}#{anchor}"
    # `#buildmedia` was the anchor of `build.media` once.
    flat = {h.replace("-", ""): h for h in heads}
    if anchor.replace("-", "") in flat:
        return f"{route}#{flat[anchor.replace('-', '')]}"
    words = {w for w in anchor.split("-") if len(w) > 3}
    best = max(heads, key=lambda h: len(words & set(h.split("-"))), default="")
    if best and words & set(best.split("-")):
        return f"{route}#{best}"
    return route


def links(text: str) -> str:
    """Links as the changelog page can follow them: a guide page at its
    route on the site, any other file of the repository on GitHub."""

    def repl(m):
        label, target = m.group(1), m.group(2)
        if re.match(r"^[a-z]+:", target) or target.startswith(("/", "#")):
            return m.group(0)
        path, _, anchor = target.partition("#")
        path = path.removeprefix("./")
        if path in ROUTES:
            return f"[{label}]({place(path, ROUTES[path], anchor)})"
        anchor = f"#{anchor}" if anchor else ""
        return f"[{label}]({REPO}/blob/master/{path}{anchor})"

    return re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", repl, text)


def dedent(lines: list[str], by: int) -> list[str]:
    return [l[by:] if l[:by].strip() == "" else l.lstrip() for l in lines]


def text_of(lines: list[str]) -> str:
    """Lines as one Markdown text. A `###` heading in the notes sits under
    a section the page draws as `h3`, so it is drawn a level below."""
    text = links("\n".join(lines).strip("\n"))
    return re.sub(r"^###(?=\s)", "####", text, flags=re.M)


def bullets(lines: list[str]) -> tuple[list[str], list[list[str]], list[str]]:
    """Lines as what comes before a list at their left edge, its bullets
    (each its lines, the marker taken off and the rest dedented), and what
    follows it."""
    before, items, after = [], [], []
    fenced = False
    i = 0
    while i < len(lines):
        line = lines[i]
        if after:
            after.append(line)
        elif line.startswith("- ") and not fenced:
            items.append([line[2:]])
        elif items and (line.startswith((" ", "\t")) or (line.strip() and not items[-1][-1].strip() == "") or fenced):
            items[-1].append(line)
        elif items and line.strip() == "":
            # A blank line inside an item when the next line is indented or
            # another bullet; the list ends at one followed by anything else.
            nxt = next((l for l in lines[i + 1:] if l.strip()), None)
            if nxt is not None and (nxt.startswith((" ", "- "))):
                items[-1].append(line)
            else:
                after.append(line)
        elif items:
            after.append(line)
        else:
            before.append(line)
        if line.strip().startswith("```"):
            fenced = not fenced
        i += 1
    items = [[it[0]] + dedent(it[1:], 2) for it in items]
    for it in items:
        while it and it[-1].strip() == "":
            it.pop()
    return before, items, after


def item(lines: list[str]) -> dict:
    text, points, after = bullets(lines)
    return {
        "text": text_of(text),
        "points": [text_of(p) for p in points],
        "after": text_of(after),
    }


def slug(text: str) -> str:
    return sitepages.slug(sitepages.plain(text))


def tag_date(version: str, kept: dict) -> str:
    for tag in (f"v{version}", f"v{version}.0"):
        done = subprocess.run(
            ["git", "-C", str(ROOT), "log", "-1", "--format=%cs", f"refs/tags/{tag}"],
            capture_output=True, text=True,
        )
        if done.returncode == 0 and done.stdout.strip():
            return done.stdout.strip()
    return kept.get(version, "")


def spoken(date: str) -> str:
    if not date:
        return ""
    y, m, d = date.split("-")
    return f"{int(d)} {MONTHS[int(m) - 1]} {y}"


def releases() -> list[dict]:
    kept = {}
    if COMMITTED.exists():
        kept = {r["version"]: r["date"] for r in json.loads(COMMITTED.read_text())}
    out = []
    current = None
    section = None
    body = []

    def close_section():
        if current is None:
            return
        lines = list(body)
        while lines and lines[-1].strip() in ("", "---"):
            lines.pop()
        if section is None:
            current["summary"] = text_of(lines)
            return
        before, items, after = bullets(lines)
        sid = f"{current['version']}-{slug(section)}"
        taken = {s["id"] for s in current["sections"]}
        n = 2
        while sid in taken:
            sid = f"{current['version']}-{slug(section)}-{n}"
            n += 1
        current["sections"].append({
            "title": section,
            "id": sid,
            "intro": text_of(before),
            "items": [item(i) for i in items],
            "after": text_of(after),
        })

    for line in NOTES.read_text().splitlines():
        head = re.match(r"^# WebFluent v(\S+) Release Notes\s*$", line)
        if head:
            close_section()
            version = head.group(1)
            date = tag_date(version, kept)
            current = {"version": version, "id": version, "date": date, "when": spoken(date),
                       "summary": "", "sections": []}
            out.append(current)
            section, body = None, []
        elif line.startswith("## ") and current is not None:
            close_section()
            section, body = line[3:].strip(), []
        elif current is not None:
            body.append(line)
    close_section()
    for r in out:
        r["count"] = sum(len(s["items"]) for s in r["sections"])
    return out


def lit(text: str) -> str:
    """`text` as a WebFluent string literal, as written: raw, so a sample
    of code in the notes keeps its quotes and braces."""
    return sitepages.wf_raw(text) if text else '""'


def strings(items: list[str], ind: str) -> str:
    if not items:
        return "[]"
    inner = ",\n".join(f"{ind}    {lit(i)}" for i in items)
    return f"[\n{inner}\n{ind}]"


def change(c: dict, ind: str) -> str:
    args = [f"text: {lit(c['text'])}"]
    if c["points"]:
        args.append(f"points: {strings(c['points'], ind)}")
    if c["after"]:
        args.append(f"after: {lit(c['after'])}")
    return "Change(" + ", ".join(args) + ")"


def section(s: dict, ind: str) -> str:
    args = [f"title: {lit(s['title'])}", f'id: "{s["id"]}"']
    if s["intro"]:
        args.append(f"intro: {lit(s['intro'])}")
    if s["items"]:
        inner = ",\n".join(f"{ind}    {change(c, ind + '    ')}" for c in s["items"])
        args.append(f"items: [\n{inner}\n{ind}]")
    if s["after"]:
        args.append(f"after: {lit(s['after'])}")
    return "NotesSection(" + ", ".join(args) + ")"


def release(r: dict, ind: str) -> str:
    args = [f'version: "{r["version"]}"', f'id: "{r["id"]}"']
    for key in ("date", "when"):
        if r[key]:
            args.append(f'{key}: "{r[key]}"')
    if r["summary"]:
        args.append(f"summary: {lit(r['summary'])}")
    if r["sections"]:
        inner = ",\n".join(f"{ind}    {section(s, ind + '    ')}" for s in r["sections"])
        args.append(f"sections: [\n{inner}\n{ind}]")
    args.append(f"count: {r['count']}")
    return "ReleaseNotes(" + ", ".join(args) + ")"


PAGE_HEAD = """/// Every release, newest first, each at its own address
/// (`/changelog#6.0.0`), written from RELEASE_NOTES.md by
/// scripts/docsite/changelog.py; edit the notes, not this file.
page Changelog(path: "/changelog", title: "Changelog — WebFluent", description: "What changed in each WebFluent release, newest first: additions, fixes, editor and library changes, and what to do when you upgrade.") {
    SiteHeader(docs: true)

    Row(align: .start, class: "bp-wrap") {
        style { flex-wrap: nowrap; gap: 0; @md { gap: 32px } @xl { gap: 48px } }

        VersionList(releases: VERSIONS)

        // One per page: the skip link's target. wf-allow(A16)

        Stack(gap: .lg, role: "main", id: "bp-main", tabindex: "-1", class: "bp-main") {
            style { gap: 24px; padding: 32px 0 72px; @md { padding-top: 40px } }

            Breadcrumbs(trail: [Place(key: "nav.reference", to: "/reference")], current: t("footer.changelog"))

            Stack(gap: .sm) {
                style { gap: 12px }
                Row(class: "bp-labels") { KindLabel(.reference) }
                Heading(t("footer.changelog")).h1 { style { margin: 0; font-size: clamp(30px, 5vw, 40px); line-height: 1.08; font-weight: 600; letter-spacing: -0.02em } }
                Markdown(t("changelog.lede"), class: "bp-lede")
                Row(gap: .md) {
                    style { flex-wrap: wrap; font-size: 14px }
                    Link(t("changelog.github"), href: "{REPO}/releases")
                    Link("RELEASE_NOTES.md", href: "{REPO}/blob/master/RELEASE_NOTES.md")
                }
            }
"""

PAGE_TAIL = """        }

        DocsOnThisPage(sections: RAIL, source: "RELEASE_NOTES.md")
    }
}
"""


def page(found: list[dict]) -> str:
    ind = "            "
    versions = ",\n".join(
        f'{ind}    ReleaseNotes(version: "{r["version"]}", id: "{r["id"]}"'
        + (f', date: "{r["date"]}"' if r["date"] else "") + ")"
        for r in found
    )
    rail = ", ".join(f'Anchor(id: "{s["id"]}", label: {lit(s["title"])})' for s in found[0]["sections"])
    # The latest is the newest release that has a date: one still being
    # written (no tag yet) is not released, and says so.
    latest = next((r["version"] for r in found if r["date"]), None)
    calls = "\n".join(
        f"\n{ind}ReleaseEntry({release(r, ind)}" + (", latest: true" if r["version"] == latest else "") + ")"
        for r in found
    )
    head = PAGE_HEAD.replace("VERSIONS", f"[\n{versions}\n        ]")
    return head + calls + "\n" + PAGE_TAIL.replace("RAIL", f"[{rail}]")


def main():
    ROUTES.update(routes())
    found = releases()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(found, ensure_ascii=False, indent=1) + "\n")
    PAGE_OUT.write_text(page(found))
    # The site's files are held to the formatter by tests/fmt_corpus.rs.
    wf = Path(os.environ["WF_BIN"]) if os.environ.get("WF_BIN") else ROOT / "target" / "debug" / "wf"
    subprocess.run([str(wf) if wf.exists() else "wf", "fmt", str(PAGE_OUT)], check=False, capture_output=True)
    print(f"wrote {len(found)} releases to {OUT} and {PAGE_OUT}")


if __name__ == "__main__":
    main()
