#!/usr/bin/env python3
"""Write the documentation site's data files.

- `site/src/registry.json`: the components reference, from `wf registry
  --json` — one entry per built-in with its signature, props, flags,
  parts, events and the group it belongs to. The reference pages read it
  as `data registry`.
- `site/src/search-index.json`: every chapter and every section of the
  guide, from `md-docs/`, for the header's search.

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
spec = importlib.util.spec_from_file_location("guide", ROOT / "scripts" / "site-from-guide.py")
guide = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guide)


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
        })
    universal = {
        "props": [prop_row(p) for p in data["universal"]["props"]],
        "events": [{"name": f"on {e['name']}", "meaning": e["summary"]} for e in data["universal"]["events"]],
    }
    return {"groups": groups, "components": out, "universal": universal, "icons": data["icons"], "count": len(out)}


def search_index():
    rows = []
    for f in sorted((ROOT / "md-docs").glob("[0-9][0-9]-*.md")):
        num = f.name[:2]
        route, _ = guide.CHAPTERS[num]
        md = f.read_text()
        title = re.sub(r"^# (\d+\.\s*)?", "", next(l for l in md.split("\n") if l.startswith("# "))).strip()
        rows.append({"chapter": num, "title": title, "section": "", "route": f"/docs/{route}"})
        for h in re.findall(r"^## (.+)$", md, re.M):
            text = guide.plain(h)
            if text in ("Next", "Contents"):
                continue
            rows.append({"chapter": num, "title": title, "section": text, "route": f"/docs/{route}#{guide.slug(text)}"})
    return rows


if __name__ == "__main__":
    (SITE / "registry.json").write_text(json.dumps(registry(), indent=1) + "\n")
    (SITE / "search-index.json").write_text(json.dumps(search_index(), indent=1) + "\n")
    print(f"wrote registry.json and search-index.json to {SITE}")
