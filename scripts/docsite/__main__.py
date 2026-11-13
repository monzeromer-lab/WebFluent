"""Write everything the documentation site takes from the guide and the
compiler, in order: `python3 scripts/docsite` (or `cargo xtask site`).

1. `components.py` — `md-docs/reference/components.md`, from `wf registry --json`
2. `errors.py` — `site/src/data/errors.json`, a page per code
   (`site/src/pages/ErrorPages.wf`) and the list of codes on
   `md-docs/errors/diagnostics.md`, from `wf explain --json` and each code's page
3. `guide.py` — the site's chapter pages, the book's chapter list
   (`site/src/components/Guide.wf`), the chapter titles in `en.json` and the
   guide's contents table
4. `reference.py` — the reference's pages (`site/src/pages/reference/`), and
   `site/src/data/{components,commands,config,tokens}.json` for tools, from
   `wf registry/commands/config schema/theme --json`
5. `data.py` — `site/public/search-index.json`
6. `changelog.py` — `site/src/data/changelog.json` and the page that draws
   it, `site/src/pages/Changelog.wf`, from `RELEASE_NOTES.md`

With `--check`, each writes into a scratch directory instead and the run
fails if anything committed differs from what it would write. `WF_BIN`
names the `wf` to use (by default `target/debug/wf`).
"""
import filecmp
import os
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def run(script: str, env: dict) -> None:
    done = subprocess.run([sys.executable, str(HERE / script)], env={**os.environ, **env})
    if done.returncode != 0:
        raise SystemExit(f"scripts/docsite/{script} failed")


def check_fences() -> list[str]:
    """Every fence in the guide says only what a fence can say."""
    sys.path.insert(0, str(HERE))
    import fences

    problems = []
    for page in sorted((ROOT / "md-docs").rglob("*.md")):
        open_ticks = None
        for n, line in enumerate(page.read_text().splitlines(), 1):
            ticks = len(line) - len(line.lstrip("`"))
            if ticks < 3:
                continue
            if open_ticks is not None:
                if ticks >= open_ticks and not line[ticks:].strip():
                    open_ticks = None
                continue
            open_ticks = ticks
            info = line[ticks:].strip()
            if info:
                try:
                    fences.parse(info)
                except ValueError as e:
                    problems.append(f"{page.relative_to(ROOT)}:{n}: {e}")
    return problems


def main() -> None:
    check = "--check" in sys.argv[1:]
    problems = check_fences()
    if problems:
        raise SystemExit("\n".join(problems))
    if not check:
        run("components.py", {})
        run("errors.py", {})
        run("guide.py", {})
        run("reference.py", {})
        run("data.py", {})
        run("changelog.py", {})
        return

    with tempfile.TemporaryDirectory(prefix="wf-docsite-") as tmp:
        tmp = Path(tmp)
        run("components.py", {"WF_COMPONENTS_OUT": str(tmp / "components.md")})
        run("errors.py", {"WF_ERRORS_OUT": str(tmp / "errors")})
        run("guide.py", {"WF_GUIDE_OUT": str(tmp / "guide")})
        run("reference.py", {"WF_REFERENCE_OUT": str(tmp / "reference")})
        (tmp / "data").mkdir()
        run("data.py", {"WF_SITE_DATA_OUT": str(tmp / "data")})
        (tmp / "changelog").mkdir()
        run("changelog.py", {"WF_CHANGELOG_OUT": str(tmp / "changelog")})
        pairs = [
            (tmp / "components.md", ROOT / "md-docs/reference/components.md"),
            (tmp / "errors/errors.json", ROOT / "site/src/data/errors.json"),
            (tmp / "errors/diagnostics.md", ROOT / "md-docs/errors/diagnostics.md"),
            (tmp / "errors/ErrorPages.wf", ROOT / "site/src/pages/ErrorPages.wf"),
            (tmp / "guide/book/Guide.wf", ROOT / "site/src/components/Guide.wf"),
            (tmp / "data/search-index.json", ROOT / "site/public/search-index.json"),
            (tmp / "changelog/changelog.json", ROOT / "site/src/data/changelog.json"),
            (tmp / "changelog/Changelog.wf", ROOT / "site/src/pages/Changelog.wf"),
        ]
        pairs += [(tmp / "reference" / name, ROOT / "site/src/data" / name)
                  for name in ("components.json", "commands.json", "config.json", "tokens.json")]
        written = ROOT / "site/src/pages/reference"
        fresh = {p.name for p in (tmp / "reference").glob("*.wf")}
        have = {p.name for p in written.glob("*.wf")}
        for name in sorted(fresh ^ have):
            problems.append(f"site/src/pages/reference/{name}: {'missing' if name in fresh else 'not from reference.py'}")
        pairs += [(tmp / "reference" / name, written / name) for name in sorted(fresh & have)]
        committed = ROOT / "site/src/pages/guide"
        fresh = {p.name for p in (tmp / "guide").glob("*.wf")}
        have = {p.name for p in committed.glob("*.wf")}
        for name in sorted(fresh ^ have):
            problems.append(f"site/src/pages/guide/{name}: {'missing' if name in fresh else 'not from the guide'}")
        pairs += [(tmp / "guide" / name, committed / name) for name in sorted(fresh & have)]
        for want, have_path in pairs:
            if not filecmp.cmp(want, have_path, shallow=False):
                problems.append(f"{have_path.relative_to(ROOT)} is behind; run `cargo xtask site`")
    if problems:
        raise SystemExit("\n".join(problems))
    print("the site's generated files are current")


main()
