"""Write everything the documentation site takes from the guide and the
compiler, in order: `python3 scripts/docsite` (or `cargo xtask site`).

1. `components.py` — `md-docs/reference/components.md`, from `wf registry --json`
2. `guide.py` — the site's chapter pages, the book's chapter list
   (`site/src/components/Guide.wf`), the chapter titles in `en.json` and the
   guide's contents table
3. `data.py` — `site/src/registry.json` and `site/public/search-index.json`

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
        run("guide.py", {})
        run("data.py", {})
        return

    with tempfile.TemporaryDirectory(prefix="wf-docsite-") as tmp:
        tmp = Path(tmp)
        run("components.py", {"WF_COMPONENTS_OUT": str(tmp / "components.md")})
        run("guide.py", {"WF_GUIDE_OUT": str(tmp / "guide")})
        (tmp / "data").mkdir()
        run("data.py", {"WF_SITE_DATA_OUT": str(tmp / "data")})
        pairs = [
            (tmp / "components.md", ROOT / "md-docs/reference/components.md"),
            (tmp / "guide/book/Guide.wf", ROOT / "site/src/components/Guide.wf"),
            (tmp / "data/registry.json", ROOT / "site/src/registry.json"),
            (tmp / "data/search-index.json", ROOT / "site/public/search-index.json"),
        ]
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
