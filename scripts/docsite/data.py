#!/usr/bin/env python3
"""Write the documentation site's search index.

`site/public/search-index.json`: every chapter, section, component and
diagnostic code, with the terms each section is about, for the header's
search. It is a file the site fetches the first time the search box is
used, not part of every page's bundle. The reference's own data is
`reference.py`'s.

Rerun after the registry or the guide changes: `cargo xtask site`.
"""
import json
import os
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
# `WF_SITE_DATA_OUT` and `WF_BIN` let `tests/docs_parse.rs` write these into a
# scratch directory with the binary it just built, and hold the committed
# files to the result.
SITE = Path(os.environ["WF_SITE_DATA_OUT"]) if os.environ.get("WF_SITE_DATA_OUT") else ROOT / "site" / "src"
wf = Path(os.environ["WF_BIN"]) if os.environ.get("WF_BIN") else ROOT / "target" / "debug" / "wf"
if not wf.exists():
    wf = "wf"

import importlib.util
import sys
spec = importlib.util.spec_from_file_location("guide", ROOT / "scripts" / "docsite" / "guide.py")
guide = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guide)
sys.path.insert(0, str(ROOT / "scripts" / "docsite"))
import reference  # noqa: E402


def builtins():
    """Every built-in, as the reference pages list it (`reference.py`)."""
    reg = json.loads(subprocess.check_output([str(wf), "registry", "--json"]))
    return reference.components(reg)["items"]


STOP = set("""about after again also always another because before being between both cannot could does doesn during each either every first from have here into itself just like made make many more most much must need never only other over same should since some such than that their them then there these they this those through under until very what when where which while with within without would your yours""".split())


def terms(text: str, cap: int = 24) -> str:
    """The words in a section a reader might search for: what is written as
    code, capitalised names, and longer words — each once, in order, up to
    `cap`. The index is part of the site's bundle, so it stays short."""
    body = re.sub(r"```.*?```", " ", text, flags=re.S)
    # An alert's marker (`> [!TIP]`) is layout, not something to search for.
    body = re.sub(r"\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]", " ", body)
    found = []
    for code in re.findall(r"`([^`]+)`", body):
        for w in re.findall(r"[A-Za-z_$@.][\w.$-]{1,}", code):
            found.append(w.strip(".").lower())
    for w in re.findall(r"[A-Za-z][A-Za-z0-9-]{2,}", re.sub(r"`[^`]*`", " ", body)):
        if (w[0].isupper() or len(w) >= 7) and w.lower() not in STOP:
            found.append(w.lower())
    seen = []
    for w in found:
        if w and w not in seen:
            seen.append(w)
    return " ".join(seen[:cap])


def search_index():
    rows = []
    for f in guide.chapter_files():
        meta = guide.read_meta(f)
        num = f"{meta['chapter']:02d}"
        route = meta["route"]
        md = guide.front_matter(f)[1]
        title = meta["title"]
        if route == guide.REFERENCE_ROUTE:
            continue  # its entries come from the registry, below
        rows.append({"chapter": num, "title": title, "section": "", "route": guide.href(route), "words": terms(meta["description"])})
        # Every `##` and `###` section, with what its text is about.
        parts = re.split(r"^(#{2,3}) (.+)$", md, flags=re.M)
        for k in range(1, len(parts), 3):
            text = guide.plain(parts[k + 1])
            if text in ("Next", "Contents"):
                continue
            rows.append({
                "chapter": num,
                "title": title,
                "section": text,
                "route": f"{guide.href(route)}#{guide.slug(text)}",
                "words": terms(parts[k + 2]),
            })
    # Every recipe, by its title and what its text is about.
    for f in sorted((ROOT / "md-docs" / "recipes").glob("*.md")):
        meta, md = guide.front_matter(f)
        if f.name == "index.md":
            continue
        title = next(l[2:].strip() for l in md.splitlines() if l.startswith("# "))
        rows.append({"chapter": "", "title": "Recipes", "section": title, "route": meta["route"],
                     "words": terms(meta.get("description", "") + " " + md)})
    # Every built-in, by name, with what it takes.
    ref = guide.front_matter(ROOT / "md-docs" / "reference" / "components.md")[0]
    for c in builtins():
        if c["legacy"]:
            continue
        rows.append({
            "chapter": f"{ref['chapter']:02d}",
            "title": "Components reference",
            "section": c["name"],
            "route": f"/reference/{c['slug']}",
            "words": terms(c["summary"] + " " + " ".join(f"`{p['name']}`" for p in c["props"])),
        })
    return rows


if __name__ == "__main__":
    # Fetched on demand, so it is written compact, into `public/`.
    public = SITE if os.environ.get("WF_SITE_DATA_OUT") else ROOT / "site" / "public"
    public.mkdir(parents=True, exist_ok=True)
    (public / "search-index.json").write_text(json.dumps(search_index(), separators=(",", ":"), ensure_ascii=False) + "\n")
    print(f"wrote {public / 'search-index.json'}")
