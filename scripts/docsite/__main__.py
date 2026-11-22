"""Write what the documentation site takes from the compiler, and what it
makes of its own pages, in order: `python3 scripts/docsite` (or
`cargo xtask site`).

The site's pages under `site/src/pages/` are written by hand: the guide's
chapters (`guide/`), the recipes, the cheat sheets, the configuration and
token pages. Nothing here writes them, but for the parts marked in them as
the compiler's: each family's list of codes on the diagnostics chapter, the
baseline's tables on the token page. What is written:

1. `errors.py` — `site/src/data/errors.json`, a page per code
   (`site/src/pages/ErrorPages.wf`) and the lists of codes on the
   diagnostics chapter, from `wf explain --json`
2. `reference.py` — the reference's pages drawn from the compiler
   (`site/src/pages/reference/{Builtins,Cli,FromTheCompiler}.wf`, keeping
   the words written in `Cli.wf`), the token page's tables, and
   `site/src/data/{components,commands,config,tokens}.json` for tools, from
   `wf registry/commands/config schema/theme --json`
3. `data.py` — `site/public/search-index.json`, every page's sections and
   the compiler's codes, commands and built-ins
4. `twins.py` — a Markdown twin of every page (`site/public/**/*.md`),
   `twins.json`, `llms.txt` and `llms-full.txt`, from the pages themselves
5. `changelog.py` — `site/src/data/changelog.json` and the page that draws
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

# `WF_BIN=target/release/wf` is relative to where the run started, and a
# generator may run wf from another folder (`reference.py` from `site/`).
if os.environ.get("WF_BIN") and "/" in os.environ["WF_BIN"]:
    os.environ["WF_BIN"] = str(Path(os.environ["WF_BIN"]).resolve())


def run(script: str, env: dict) -> None:
    done = subprocess.run([sys.executable, str(HERE / script)], env={**os.environ, **env})
    if done.returncode != 0:
        raise SystemExit(f"scripts/docsite/{script} failed")


def main() -> None:
    check = "--check" in sys.argv[1:]
    if not check:
        run("errors.py", {})
        run("reference.py", {})
        run("data.py", {})
        run("twins.py", {})
        run("changelog.py", {})
        return

    problems = []
    with tempfile.TemporaryDirectory(prefix="wf-docsite-") as tmp:
        tmp = Path(tmp)
        run("errors.py", {"WF_ERRORS_OUT": str(tmp / "errors")})
        run("reference.py", {"WF_REFERENCE_OUT": str(tmp / "reference")})
        (tmp / "data").mkdir()
        run("data.py", {"WF_SITE_DATA_OUT": str(tmp / "data")})
        (tmp / "twins").mkdir()
        run("twins.py", {"WF_TWINS_OUT": str(tmp / "twins")})
        (tmp / "changelog").mkdir()
        run("changelog.py", {"WF_CHANGELOG_OUT": str(tmp / "changelog")})
        pairs = [
            (tmp / "errors/errors.json", ROOT / "site/src/data/errors.json"),
            (tmp / "errors/ErrorPages.wf", ROOT / "site/src/pages/ErrorPages.wf"),
            (tmp / "errors/43-diagnostics.wf", ROOT / "site/src/pages/guide/43-diagnostics.wf"),
            (tmp / "data/search-index.json", ROOT / "site/public/search-index.json"),
            (tmp / "changelog/changelog.json", ROOT / "site/src/data/changelog.json"),
            (tmp / "changelog/Changelog.wf", ROOT / "site/src/pages/Changelog.wf"),
        ]
        pairs += [(tmp / "reference" / name, ROOT / "site/src/data" / name)
                  for name in ("components.json", "commands.json", "config.json", "tokens.json")]
        pairs += [(tmp / "reference" / name, ROOT / "site/src/pages/reference" / name)
                  for name in ("Builtins.wf", "Cli.wf", "FromTheCompiler.wf", "Tokens.wf")]
        # The twins: every one written is committed, and nothing else is.
        public = ROOT / "site/public"
        maps = ("llms.txt", "llms-full.txt", "twins.json")
        fresh = {p.relative_to(tmp / "twins") for p in (tmp / "twins").rglob("*") if p.is_file()}
        have = {p.relative_to(public) for p in public.rglob("*.md")} | {Path(n) for n in maps if (public / n).exists()}
        for name in sorted(fresh ^ have):
            problems.append(f"site/public/{name}: {'missing' if name in fresh else 'not from twins.py'}")
        pairs += [(tmp / "twins" / name, public / name) for name in sorted(fresh & have)]
        for want, have_path in pairs:
            if not have_path.exists() or not filecmp.cmp(want, have_path, shallow=False):
                problems.append(f"{have_path.relative_to(ROOT)} is behind; run `cargo xtask site`")
    if problems:
        raise SystemExit("\n".join(problems))
    print("the site's generated files are current")


main()
