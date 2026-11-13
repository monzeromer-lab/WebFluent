#!/usr/bin/env python3
"""Write the site's changelog, `site/src/data/changelog.json`, from
`RELEASE_NOTES.md`: one entry per release, newest first, read by
`site/src/pages/Changelog.wf`.

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
`WF_CHANGELOG_OUT` writes elsewhere, for `--check`.
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import guide  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
NOTES = ROOT / "RELEASE_NOTES.md"
COMMITTED = ROOT / "site" / "src" / "data" / "changelog.json"
OUT = Path(os.environ.get("WF_CHANGELOG_OUT") or COMMITTED)
REPO = "https://github.com/monzeromer-lab/WebFluent"
MONTHS = ["January", "February", "March", "April", "May", "June", "July",
          "August", "September", "October", "November", "December"]


def routes() -> dict:
    """Each guide page's path under the repository, and where the site has it."""
    found = {}
    for f in guide.chapter_files():
        meta = guide.read_meta(f)
        found[f"md-docs/{meta['path']}"] = guide.site_path(meta["route"])
    found["md-docs/README.md"] = "/docs/guide/introduction"
    return found


ROUTES = {}


def links(text: str) -> str:
    """Links as the changelog page can follow them: a guide page at its
    route on the site, any other file of the repository on GitHub."""

    def repl(m):
        label, target = m.group(1), m.group(2)
        if re.match(r"^[a-z]+:", target) or target.startswith(("/", "#")):
            return m.group(0)
        path, _, anchor = target.partition("#")
        anchor = f"#{anchor}" if anchor else ""
        path = path.removeprefix("./")
        if path in ROUTES:
            return f"[{label}]({ROUTES[path]}{anchor})"
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
    return guide.slug(guide.plain(text))


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


def main():
    ROUTES.update(routes())
    found = releases()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(found, ensure_ascii=False, indent=1) + "\n")
    print(f"wrote {len(found)} releases to {OUT}")


if __name__ == "__main__":
    main()
