#!/usr/bin/env python3
"""Write the Coming-from cheat sheets from `md-docs/coming-from/`.

Each page is `md-docs/coming-from/<slug>.md`: front matter naming its route,
its `framework` and the `language` its own code is in; a paragraph; `## The
map`, a table of concept, their idiom and ours; `## Things that work
differently`; and `## A component, ported`. It becomes
`site/src/pages/comingfrom/<Name>.wf`, framed by `ComingFromShell`: a
`CheatRow` per line of the map, the differences and the port in their
places, and the other sheets as tabs.

`WF_COMINGFROM_OUT` writes into that directory instead (for
`cargo xtask site --check`). Rerun with `cargo xtask site`.
"""
import importlib.util
import os
import re
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
PAGES = ROOT / "md-docs" / "coming-from"
_spec = importlib.util.spec_from_file_location("guide", HERE / "guide.py")
guide = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(guide)

OUT = Path(os.environ["WF_COMINGFROM_OUT"]) if os.environ.get("WF_COMINGFROM_OUT") else None
wf = Path(os.environ["WF_BIN"]) if os.environ.get("WF_BIN") else ROOT / "target" / "debug" / "wf"

# The sheets' translations, by the sheet each translates.
TRANSLATIONS = {k: v for k, v in guide.translations().items() if k.startswith("coming-from/")}

# The order the sheets are listed in.
ORDER = ["react", "vue", "svelte", "html-css", "latex"]


def q(text: str) -> str:
    return '"' + guide.wf_string(text) + '"'


def pascal(slug: str) -> str:
    return "".join(w.capitalize() for w in re.split(r"[^a-z0-9]+", slug) if w)


def cell(text: str) -> str:
    """A table cell's code, as the cheat row shows it: one span of code, or
    several joined; prose around it stays."""
    text = text.replace("\\|", "|").strip()
    spans = re.findall(r"`([^`]*)`", text)
    if spans and re.sub(r"`[^`]*`", "", text).strip(" ,…") == "":
        return " … ".join(spans) if "…" in text else ", ".join(spans)
    return text.replace("`", "")


def render(path: Path, sheets: list, other: str = "") -> tuple[str, str]:
    """A sheet, or its translation (`md-docs/ar/coming-from/<slug>.md`,
    `lang: "ar"`), which names the sheet it translates; `other` is the
    address of the page in the other language."""
    meta, body = guide.front_matter(path)
    lang = meta.get("lang", "")
    guide.CURRENT["dir"] = path.parent
    title = next(l[2:].strip() for l in body.splitlines() if l.startswith("# "))
    body = re.sub(r"^# .*\n", "", body.lstrip("\n"), count=1)
    intro, *sections = re.split(r"(?m)^## ", body)
    parts = {}
    for sec in sections:
        heading, _, text = sec.partition("\n")
        parts[heading.strip()] = text.strip()
    rows = []
    for line in parts.get("The map", "").splitlines():
        if not line.startswith("|") or re.match(r"^\|[\s:|-]+\|?\s*$", line):
            continue
        cells = [c.strip() for c in re.split(r"(?<!\\)\|", line.strip().strip("|"))]
        if cells[0] == "Concept":
            continue
        rows.append(cells)
    diff_lines, _ = guide.body(parts.get("Things that work differently", ""), title, "            ")
    port_lines, _ = guide.body(parts.get("A component, ported", ""), title, "            ")
    summary = guide.rewrite_links(" ".join(intro.split()))
    name = "ComingFrom" + pascal(path.stem)
    translated = {Path(m["translates"]).stem: m["route"] for _, m in TRANSLATIONS.values()}
    if lang:
        tabs = ", ".join(f'Place(key: "coming.{s}", to: "{translated.get(s, "/coming-from/" + s)}")' for s in sheets)
    else:
        tabs = ", ".join(f'Place(key: "coming.{s}", to: "/coming-from/{s}")' for s in sheets)
    own = f'lang: "{lang}", translates: {name}, ' if lang else ""
    shell_lang = f', other: "{other}", lang: "{lang}"' if lang else (f', other: "{other}"' if other else "")
    if lang:
        name += lang.capitalize()
    rel = path.relative_to(ROOT / "md-docs").as_posix()
    out = [
        f"// A cheat sheet, written from md-docs/{rel} by",
        "// scripts/docsite/comingfrom.py; edit the sheet, not this file.",
        f"page {name}(path: {q(meta['route'])}, title: {q(title)}, description: {q(meta['description'])}, {own}) {{".replace(", ) {", ") {"),
        f"    ComingFromShell({q(title)}, summary: {q(summary)}, from: {q(meta['framework'])}, sheets: [{tabs}], "
        "sections: [Anchor(id: \"the-map\", label: t(\"coming.map\")), Anchor(id: \"things-that-work-differently\", label: t(\"coming.differences\")), Anchor(id: \"a-component-ported\", label: t(\"coming.port\"))], "
        f"source: \"md-docs/{rel}\"{shell_lang}) {{",
        "        differences {",
        *diff_lines,
        "        }",
        "        port {",
        *port_lines,
        "        }",
    ]
    for concept, theirs, ours in (r[:3] for r in rows if len(r) >= 3):
        out.append(
            f"        CheatRow({q(concept.replace('`', ''))}, theirs: {q(cell(theirs))}, "
            f"ours: {q(cell(ours))}, language: {q(meta.get('language', ''))})"
        )
    out += ["    }", "}"]
    return name, "\n".join(out) + "\n"


def main():
    pages = sorted(PAGES.glob("*.md"), key=lambda p: ORDER.index(p.stem) if p.stem in ORDER else 99)
    sheets = [p.stem for p in pages]
    target = OUT or ROOT / "site" / "src" / "pages" / "comingfrom"
    target.mkdir(parents=True, exist_ok=True)
    for old in target.glob("*.wf"):
        old.unlink()
    for p in pages:
        found = TRANSLATIONS.get(f"coming-from/{p.name}")
        name, text = render(p, sheets, other=found[1]["route"] if found else "")
        (target / f"{name}.wf").write_text(text)
        if found:
            name, text = render(found[0], sheets, other=p.parent.name and guide.front_matter(p)[0]["route"])
            (target / f"{name}.wf").write_text(text)
    subprocess.run([str(wf) if wf.exists() else "wf", "fmt", str(target)], check=False, capture_output=True)
    print(f"wrote {len(pages)} cheat sheets")


if __name__ == "__main__":
    main()
