#!/usr/bin/env python3
"""Write the error pages' data, and the list of codes on the errors index.

- `site/src/data/errors.json`: one entry per code, from `wf explain --json`
  and the code's page, `md-docs/errors/CODE.md`, split into what the error
  template draws — the program that draws the code, what the compiler
  prints for it, why the rule exists, and the fix.
- `site/src/pages/ErrorPages.wf`: a page per code, its text written into
  the page itself, so each page's words travel in its own chunk rather
  than in every page's `app.js` (a `data` file is inlined there); and
  `ERROR_INDEX`, the sidebar's list of codes, which every error page shares.
- `md-docs/errors/diagnostics.md`: the list of every code by family,
  between `<!-- codes … -->` and `<!-- /codes -->`.

`WF_ERRORS_OUT` writes both into that directory instead (for
`cargo xtask site --check`); `WF_BIN` names the `wf`. Rerun with
`cargo xtask site`.
"""
import importlib.util
import json
import os
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ERRORS = ROOT / "md-docs" / "errors"
INDEX = ERRORS / "diagnostics.md"
OUT = Path(os.environ["WF_ERRORS_OUT"]) if os.environ.get("WF_ERRORS_OUT") else None
wf = Path(os.environ["WF_BIN"]) if os.environ.get("WF_BIN") else ROOT / "target" / "debug" / "wf"
if not wf.exists():
    wf = "wf"

_spec = importlib.util.spec_from_file_location("guide", Path(__file__).resolve().parent / "guide.py")
guide = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(guide)

OPEN = "<!-- codes: written by scripts/docsite/errors.py -->"
CLOSE = "<!-- /codes -->"


def front_matter(text: str) -> tuple[dict, str]:
    head, body = text[4:].split("\n---\n", 1)
    meta = {}
    for line in head.splitlines():
        key, _, value = line.partition(":")
        value = value.strip()
        meta[key.strip()] = json.loads(value) if value[:1] == '"' else value
    return meta, body


def blocks(body: str):
    """The page's fenced blocks and the prose between them, in order:
    `("prose", text)` or `(info, code)`."""
    out, prose, lines, i = [], [], body.split("\n"), 0
    while i < len(lines):
        line = lines[i]
        ticks = len(line) - len(line.lstrip("`"))
        if ticks >= 3:
            out.append(("prose", "\n".join(prose)))
            prose = []
            info = line[ticks:].strip()
            i += 1
            code = []
            while i < len(lines) and not (lines[i].startswith("`" * ticks) and not lines[i][ticks:].strip()):
                code.append(lines[i])
                i += 1
            out.append((info, "\n".join(code)))
        else:
            prose.append(line)
        i += 1
    out.append(("prose", "\n".join(prose)))
    return out


def split(code: str, body: str) -> dict:
    """A page's body as the template's parts."""
    body = re.sub(r"^# .*\n", "", body.lstrip("\n"), count=1)
    broken, printed, rest = "", "", []
    for info, text in blocks(body):
        if info == "prose":
            rest.append(text)
        elif info.startswith("wf") and "expect=" in info and not broken:
            broken = text
        elif info == "text" and broken and not printed:
            printed = text
        else:
            rest.append(f"```{info}\n{text}\n```")
    prose = "\n\n".join(p.strip("\n") for p in rest if p.strip())
    why, fix = prose, ""
    m = re.search(r"\*\*Fix:\*\*\s*", prose)
    if m:
        why, fix = prose[: m.start()].strip(), prose[m.end():].strip()
    return {"broken": broken, "printed": printed, "why": why, "fix": fix}


def related(code: str, text: str, titles: dict) -> list:
    """The other codes a page names, as links to their pages."""
    seen = []
    for other in re.findall(r"\b([A-Z]\d{2,3})\b", text):
        if other != code and other in titles and other not in seen:
            seen.append(other)
    return [{"kind": c, "title": titles[c], "to": f"/errors/{c}"} for c in seen[:6]]


def link_paths(markdown: str) -> str:
    """Links between the error pages as the site's routes."""
    markdown = re.sub(r"\]\(([A-Z]\d{2,3})\.md\)", r"](/errors/\1)", markdown)
    return re.sub(r"\]\(diagnostics\.md(#[^)]*)?\)", r"](/errors\1)", markdown)


def main():
    explained = json.loads(subprocess.check_output([str(wf), "explain", "--json"]))
    titles = {e["code"]: e["title"].replace("`", "") for e in explained}
    raw_titles = {e["code"]: e["title"] for e in explained}
    entries, families = [], []
    for e in explained:
        page = ERRORS / f"{e['code']}.md"
        meta, body = front_matter(page.read_text())
        parts = split(e["code"], body)
        group = meta["family"]
        if not families or families[-1][0] != group:
            families.append((group, []))
        families[-1][1].append(e["code"])
        entries.append({
            "code": e["code"],
            "title": e["title"],
            "plain": titles[e["code"]],
            "severity": e["severity"],
            "family": e["family"],
            "group": group,
            "summary": e["summary"],
            "lowerable": e["lowerable"],
            "description": meta.get("description", e["summary"]),
            "printed": parts["printed"],
            "broken": parts["broken"],
            "why": link_paths(parts["why"]),
            "fix": link_paths(parts["fix"]),
            "related": related(e["code"], parts["why"] + " " + parts["fix"], titles),
        })

    # The index's list: each family's heading and the line under it, as the
    # page has them, and every code in the compiler's order.
    index = INDEX.read_text()
    a, b = index.index(OPEN) + len(OPEN), index.index(CLOSE)
    intros = dict(re.findall(r"^### (.+)\n\n(?!- \[)(.+(?:\n.+)*)\n", index[a:b], re.M))
    listing = []
    for group, codes in families:
        listing += ["", f"### {group}", ""]
        if intros.get(group):
            listing += [intros[group], ""]
        listing += [f"- [{c}]({c}.md) — {raw_titles[c]}" for c in codes]
    new_index = index[:a] + "\n".join(listing) + "\n\n" + index[b:]

    data = json.dumps(entries, indent=1, ensure_ascii=False) + "\n"
    pages = page_file(entries)
    if OUT:
        OUT.mkdir(parents=True, exist_ok=True)
        (OUT / "errors.json").write_text(data)
        (OUT / "diagnostics.md").write_text(new_index)
        (OUT / "ErrorPages.wf").write_text(pages)
        fmt(OUT / "ErrorPages.wf")
    else:
        target = ROOT / "site" / "src" / "data" / "errors.json"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(data)
        INDEX.write_text(new_index)
        (ROOT / "site" / "src" / "pages" / "ErrorPages.wf").write_text(pages)
        fmt(ROOT / "site" / "src" / "pages" / "ErrorPages.wf")
    print(f"wrote {len(entries)} error pages' data")


def fmt(path: Path) -> None:
    subprocess.run([str(wf), "fmt", str(path)], check=False, capture_output=True)


def q(text: str) -> str:
    return '"' + guide.wf_string(text) + '"'


def page_file(entries: list) -> str:
    """Every code's page, its words written in."""
    out = [
        "/// A page for every code the compiler reports, at `/errors/CODE`: written",
        "/// by scripts/docsite/errors.py from `wf explain --json` and each code's",
        "/// page, `md-docs/errors/CODE.md`; edit those, not this file. The index,",
        "/// `/errors`, is the guide's diagnostics chapter.",
        "",
        "/// The error pages' sidebar: every code, by family.",
        "const ERROR_INDEX: [IndexEntry] = [",
        ",\n".join(
            f'    IndexEntry(label: {q(e["code"] + " " + e["plain"])}, to: "/errors/{e["code"]}", group: {q(e["group"])})'
            for e in entries
        ),
        "]",
        "",
    ]
    for e in entries:
        anchors = [("printed", "error.printed")] if e["printed"] else []
        if e["why"]:
            anchors.append(("why", "error.why"))
        anchors.append(("fix", "error.fix"))
        sections = ", ".join(f'Anchor(id: "{a}", label: t("{k}"))' for a, k in anchors)
        related = ", ".join(
            f'RelatedLink(kind: {q(r["kind"])}, title: {q(r["title"])}, to: {q(r["to"])})' for r in e["related"]
        )
        title = f'{e["code"]} — {e["plain"]}'
        out.append(
            f'page Error{e["code"]}(path: "/errors/{e["code"]}", title: {q(title)}, '
            f'description: {q(e["description"])}) {{'
        )
        out.append(
            f'    ErrorShell("{e["code"]}", title: {q(e["plain"])}, severity: "{e["severity"]}", '
            f'family: {q(e["group"])}, summary: {q(e["summary"])}, printed: {guide.wf_raw(e["printed"])}, '
            f'lowerable: {"true" if e["lowerable"] else "false"}, why: {"true" if e["why"] else "false"}, '
            f'example: false, families: ERROR_INDEX, related: [{related}], sections: [{sections}], '
            f'source: "md-docs/errors/{e["code"]}.md") {{'
        )
        out.append("        fixed {")
        out.append("            Stack(gap: .sm) {")
        if e["broken"]:
            out.append('                Text(t("error.broken"), class: "bp-label", data-tone: "error") { style { align-self: flex-start } }')
            out.append(f'                CodeBlock({guide.wf_raw(e["broken"])}, title: "wf")')
        out.append(f'                Markdown({q(e["fix"])})')
        out.append("            }")
        out.append("        }")
        if e["why"]:
            out.append(f'        Markdown({q(e["why"])})')
        out.append("    }")
        out.append("}")
        out.append("")
    return "\n".join(out)


if __name__ == "__main__":
    main()
