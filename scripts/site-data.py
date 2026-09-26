#!/usr/bin/env python3
"""Write the documentation site's data files.

- `site/src/registry.json`: the components reference, from `wf registry
  --json` — one entry per built-in with its signature, props, flags,
  parts, events and the group it belongs to. The reference pages read it
  as `data registry`.
- `site/public/search-index.json`: every chapter, section, component and
  diagnostic code, with the terms each section is about, for the header's
  search. It is a file the site fetches the first time the search box is
  used, not part of every page's bundle.

Rerun after the registry or the guide changes: `python3 scripts/site-data.py`.
"""
import json
import os
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# `WF_SITE_DATA_OUT` and `WF_BIN` let `tests/docs_parse.rs` write these into a
# scratch directory with the binary it just built, and hold the committed
# files to the result.
SITE = Path(os.environ["WF_SITE_DATA_OUT"]) if os.environ.get("WF_SITE_DATA_OUT") else ROOT / "site" / "src"
wf = Path(os.environ["WF_BIN"]) if os.environ.get("WF_BIN") else ROOT / "target" / "debug" / "wf"
if not wf.exists():
    wf = "wf"

import importlib.util
import sys
spec = importlib.util.spec_from_file_location("guide", ROOT / "scripts" / "site-from-guide.py")
guide = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guide)
sys.path.insert(0, str(ROOT / "scripts"))
from component_examples import EXAMPLES  # noqa: E402


def legacy(c):
    s = c["summary"]
    return "original spelling" in s or "original grammar" in s


def prop_row(p, positional=False):
    if p.get("cases"):
        ty = " ".join(f".{c['name']}" for c in p["cases"])
    else:
        ty = p["type"]
    name = p["name"] if positional else f"{p['name']}:"
    return {"name": name, "type": ty + ("  (positional)" if positional else ""), "meaning": p["summary"]}


def signature(c, owner=None):
    name = f"{owner}.{c['name']}" if owner else c["name"]
    pos = c.get("positional")
    args = ([pos["name"]] if pos else []) + [f"{p['name']}: …" for p in c["props"][:3]]
    sig = name + ("(" + ", ".join(args) + ")" if args else "")
    if c["children"] == "elements":
        sig += " { … }"
    return sig


def registry():
    data = json.loads(subprocess.check_output([str(wf), "registry", "--json"]))
    comps = data["components"]
    tops = [c for c in comps if c["owner"] is None and not legacy(c)]
    groups = []
    for c in tops:
        if c["group"] not in groups:
            groups.append(c["group"])
    out = []
    for c in tops:
        pos = c.get("positional")
        props = ([prop_row(pos, True)] if pos else []) + [prop_row(p) for p in c["props"]]
        flags = [f".{f['name']}" for f in c["flags"] if f["prop"] not in ("animate", "speed")]
        parts = []
        for p in comps:
            if p["owner"] != c["name"]:
                continue
            ppos = p.get("positional")
            parts.append({
                "name": f"{c['name']}.{p['name']}",
                "signature": signature(p, c["name"]),
                "summary": p["summary"],
                "props": ([prop_row(ppos, True)] if ppos else []) + [prop_row(q) for q in p["props"]],
                "flags": [f".{f['name']}" for f in p["flags"] if f["prop"] not in ("animate", "speed")],
            })
        siblings = [s["name"] for s in tops if s["group"] == c["group"] and s["name"] != c["name"]]
        out.append({
            "name": c["name"],
            "slug": c["name"].lower(),
            "group": c["group"],
            "summary": c["summary"],
            "plain": c["summary"].replace("`", ""),
            "signature": signature(c),
            "block": c["children"] == "elements",
            "props": props,
            "flags": flags,
            "events": [{"name": f"on {e}", "meaning": ""} for e in c["events"]],
            "parts": parts,
            "attributes": c.get("attributes") or [],
            "nearby": siblings[:4],
            "example": EXAMPLES.get(c["name"], ""),
            "cssClass": c.get("class") or "",
        })
    universal = {
        "props": [prop_row(p) for p in data["universal"]["props"]],
        "events": [{"name": f"on {e['name']}", "meaning": e["summary"]} for e in data["universal"]["events"]],
    }
    return {"groups": groups, "components": out, "universal": universal, "icons": data["icons"], "count": len(out)}


STOP = set("""about after again also always another because before being between both cannot could does doesn during each either every first from have here into itself just like made make many more most much must need never only other over same should since some such than that their them then there these they this those through under until very what when where which while with within without would your yours""".split())


def terms(text: str, cap: int = 24) -> str:
    """The words in a section a reader might search for: what is written as
    code, capitalised names, and longer words — each once, in order, up to
    `cap`. The index is part of the site's bundle, so it stays short."""
    body = re.sub(r"```.*?```", " ", text, flags=re.S)
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
    for f in sorted((ROOT / "md-docs").glob("[0-9][0-9]-*.md")):
        num = f.name[:2]
        meta = guide.read_meta(num, f)
        route = meta["route"]
        md = f.read_text()
        title = meta["title"]
        if route == guide.REFERENCE_ROUTE:
            continue  # its entries come from the registry, below
        rows.append({"chapter": num, "title": title, "section": "", "route": f"/docs/{route}", "words": terms(meta["description"])})
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
                "route": f"/docs/{route}#{guide.slug(text)}",
                "words": terms(parts[k + 2]),
            })
    # Every built-in, by name, with what it takes.
    ref = next(f for f in (ROOT / "md-docs").glob("[0-9][0-9]-components-reference.md"))
    for c in registry()["components"]:
        rows.append({
            "chapter": ref.name[:2],
            "title": "Components reference",
            "section": c["name"],
            "route": f"/docs/reference/{c['slug']}",
            "words": terms(c["summary"] + " " + " ".join(f"`{p['name']}`" for p in c["props"])),
        })
    return rows


if __name__ == "__main__":
    (SITE / "registry.json").write_text(json.dumps(registry(), indent=1) + "\n")
    # Fetched on demand, so it is written compact, into `public/`.
    public = SITE if os.environ.get("WF_SITE_DATA_OUT") else ROOT / "site" / "public"
    public.mkdir(parents=True, exist_ok=True)
    (public / "search-index.json").write_text(json.dumps(search_index(), separators=(",", ":"), ensure_ascii=False) + "\n")
    print(f"wrote {SITE / 'registry.json'} and {public / 'search-index.json'}")
