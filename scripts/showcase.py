#!/usr/bin/env python3
"""Write the documentation site's gallery: every example document, deck and
video, rendered by the compiler, with previews of its first two pages.

    python3 scripts/showcase.py            the gallery
    python3 scripts/showcase.py --hero     and site/art/hero.png (needs Chrome)

For every project under examples/documents/, examples/decks/ and
examples/videos/ (any directory holding a `webfluent.app.json`), it builds a
copy laid out as the repository is — so `../fonts` resolves and the
committed `build/` of the example is left alone — and writes:

  site/public/showcase/<output>         the PDF (or video) the build wrote
  site/public/showcase/<stem>-1.webp    the first page, 640px wide
  site/public/showcase/<stem>-2.webp    the second
  site/src/showcase.json                one entry per example, which the
                                        gallery page reads as `data showcase`

The page count is the one `wf build` reports; the title is the one written
into the PDF (a `Document(title:)`), or the page's `title:` when the PDF
carries only the project's name.

Needs: the compiler (`target/release/wf`, or `WF_BIN`), `pdftoppm` and
`pdfinfo` (poppler-utils) and `magick` (ImageMagick 7). A video example also
needs `ffmpeg`. `--hero` needs Chrome or Chromium (`WF_CHROME` names one).
"""
import functools
import http.server
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EXAMPLES = ROOT / "examples"
OUT = ROOT / "site" / "public" / "showcase"
DATA = ROOT / "site" / "src" / "showcase.json"
TREE = "https://github.com/monzeromer-lab/WebFluent/tree/master/"
# The groups, in the order the gallery shows them, and what each one holds.
# examples/videos/ holds the projects the launch videos are recorded from:
# each is a document or a deck by its own output, and one that only writes a
# web page (the counter) or repeats an example already shown is left out.
GROUPS = [("documents", "document"), ("decks", "deck"), ("videos", None)]
VIDEO = (".mp4", ".webm", ".mov")
PREVIEW_WIDTH = 640


def wf_binary():
    if os.environ.get("WF_BIN"):
        return os.environ["WF_BIN"]
    for build in ("release", "debug"):
        exe = ROOT / "target" / build / "wf"
        if exe.exists():
            return str(exe)
    found = shutil.which("wf")
    if not found:
        sys.exit("showcase: no compiler; run `cargo build --release` or set WF_BIN")
    return found


def need(*tools):
    missing = [t for t in tools if not shutil.which(t)]
    if missing:
        sys.exit(f"showcase: needs {', '.join(missing)} on PATH")


def run(cmd, **kw):
    done = subprocess.run(cmd, capture_output=True, text=True, **kw)
    if done.returncode != 0:
        sys.exit(f"showcase: {cmd[0]} failed:\n{done.stdout}{done.stderr}")
    return done


def copy_examples(tmp):
    """The groups that exist, copied without their build output, so a build
    writes into the copy and the paths between groups still resolve."""
    ignore = shutil.ignore_patterns("build", ".wf-cache", ".wf-sizes.json")
    for group, _ in GROUPS:
        src = EXAMPLES / group
        if src.is_dir():
            shutil.copytree(src, tmp / group, ignore=ignore)


def projects():
    for group, kind in GROUPS:
        base = EXAMPLES / group
        if not base.is_dir():
            continue
        for config in sorted(base.glob("*/webfluent.app.json")):
            yield group, kind, config.parent


def consts(src_dir):
    """`const NAME = "…"` across a project's sources, to resolve a title's
    splices the way the compiler would."""
    table = {}
    for f in sorted(src_dir.rglob("*.wf*")):
        for m in re.finditer(r'^const\s+(\w+)(?:\s*:\s*\w+)?\s*=\s*"([^"{}]*)"', f.read_text(), re.M):
            table[m.group(1)] = m.group(2)
    return table


def page_meta(src_dir):
    """The `title:` and `description:` of the project's last page — a page
    quoted inside a raw string in a deck comes before the real one."""
    title = description = ""
    table = consts(src_dir)
    for f in sorted(src_dir.rglob("*.wf*")):
        for m in re.finditer(r"^page\s+\w+\((.*)$", f.read_text(), re.M):
            head = m.group(1)
            t = re.search(r'title:\s*"((?:[^"\\]|\\.)*)"', head)
            d = re.search(r'description:\s*"((?:[^"\\]|\\.)*)"', head)
            title = t.group(1) if t else title
            description = d.group(1) if d else description

    def resolve(s):
        return re.sub(r"\{(\w+)\}", lambda m: table.get(m.group(1), m.group(0)), s)

    return resolve(title), resolve(description)


def human_size(n):
    if n < 1000 * 1000:
        return f"{max(1, round(n / 1000))} kB"
    return f"{n / 1000 / 1000:.1f} MB"


def preview(src, dest):
    """Scale to the preview width and write it as WebP — a fifth of the PNG's
    bytes for a page of text, which matters on a phone — with no metadata,
    so a page that did not change writes the same bytes."""
    run(["magick", str(src), "-resize", f"{PREVIEW_WIDTH}x", "-strip",
         "-quality", "82", "-define", "webp:method=6", str(dest)])


def pdf_previews(pdf, stem, tmp, pages):
    out = []
    for n in range(1, min(2, pages) + 1):
        raw = tmp / f"{stem}-raw-{n}"
        run(["pdftoppm", "-png", "-singlefile", "-f", str(n), "-l", str(n),
             "-scale-to-x", str(PREVIEW_WIDTH * 2), "-scale-to-y", "-1", str(pdf), str(raw)])
        dest = OUT / f"{stem}-{n}.webp"
        preview(Path(f"{raw}.png"), dest)
        out.append(dest)
    return out


def video_previews(video, stem, tmp):
    need("ffmpeg", "ffprobe")
    length = float(run(["ffprobe", "-v", "error", "-show_entries", "format=duration",
                        "-of", "default=nw=1:nk=1", str(video)]).stdout.strip() or 0)
    out = []
    for n, at in ((1, 0.0), (2, length / 2)):
        raw = tmp / f"{stem}-raw-{n}.png"
        run(["ffmpeg", "-v", "error", "-y", "-ss", f"{at:.2f}", "-i", str(video),
             "-frames:v", "1", str(raw)])
        dest = OUT / f"{stem}-{n}.webp"
        preview(raw, dest)
        out.append(dest)
    return out


def pdfinfo(pdf):
    info = {}
    for line in run(["pdfinfo", "-enc", "UTF-8", str(pdf)]).stdout.splitlines():
        key, _, value = line.partition(":")
        info[key.strip()] = value.strip()
    return info


def image_size(path):
    w, h = run(["magick", "identify", "-format", "%w %h", str(path)]).stdout.split()
    return int(w), int(h)


def build(wf, project, must_build=True):
    """Build one copied project; what it wrote and how many pages it says.
    A launch-video project may be meant not to build (the typo video): then
    it is `(None, None)`, and any other example that fails stops the run."""
    done = subprocess.run([wf, "build", "-d", str(project)], capture_output=True, text=True)
    said = done.stdout + done.stderr
    if done.returncode != 0:
        if not must_build:
            return None, "fails"
        sys.exit(f"showcase: {project.name} did not build:\n{said}")
    pages = None
    m = re.search(r"(?:PDF|Slides): \d+ bytes, (\d+) (?:page|slide)", said)
    if m:
        pages = int(m.group(1))
    m = re.search(r"Output: (.+)$", said, re.M)
    output = None
    if m:
        named = (project / m.group(1).strip()).resolve()
        if named.is_file():
            output = named
    if output is None:
        # A build that writes something else (a video): the newest file it made.
        made = [p for p in (project / "build").rglob("*") if p.suffix in (".pdf",) + VIDEO]
        if not made:
            return None, None
        output = max(made, key=lambda p: p.stat().st_mtime)
    return output, pages


def main():
    hero = "--hero" in sys.argv[1:]
    need("pdftoppm", "pdfinfo", "magick")
    wf = wf_binary()
    work = Path(tempfile.mkdtemp(prefix="wf-showcase-"))
    try:
        copy_examples(work)
        if OUT.exists():
            shutil.rmtree(OUT)
        OUT.mkdir(parents=True)
        entries = []
        for group, kind, project in projects():
            rel = project.relative_to(ROOT).as_posix()
            config = json.loads((project / "webfluent.app.json").read_text())
            output, pages = build(wf, work / group / project.name, must_build=kind is not None)
            if output is None and pages == "fails":
                print(f"  skipped   {rel:<38} it is meant not to build")
                continue
            if output is None:
                if kind is None:
                    print(f"  skipped   {rel:<38} it writes a web page, not a PDF")
                    continue
                sys.exit(f"showcase: {project.name} built, but wrote no PDF or video")
            if kind is None:
                kind = "deck" if config.get("build", {}).get("output_type") == "slides" else "document"
            published = OUT / output.name
            if published.exists():
                published = OUT / f"{project.name}-{output.name}"
            shutil.copyfile(output, published)
            stem = published.stem
            title, description = page_meta(project / "src")
            if output.suffix == ".pdf":
                info = pdfinfo(published)
                pages = pages or int(info.get("Pages", "0"))
                written = info.get("Title", "")
                if written and written != config.get("name"):
                    title = written
                previews = pdf_previews(published, stem, work, pages)
            else:
                previews = video_previews(published, stem, work)
            width, height = image_size(previews[0])
            size = published.stat().st_size
            entry = {
                "slug": project.name,
                "title": title or config.get("name", project.name),
                "description": description,
                "kind": kind,
                "lang": config.get("meta", {}).get("lang", "en"),
                "pages": pages or 0,
                "bytes": size,
                "size": human_size(size),
                "file": f"/showcase/{published.name}",
                "previews": [f"/showcase/{p.name}" for p in previews],
                "width": width,
                "height": height,
                "source": TREE + rel,
            }
            if any(e["title"] == entry["title"] for e in entries):
                print(f"  skipped   {rel:<38} the same as an example already shown")
                published.unlink()
                for p in previews:
                    p.unlink()
                continue
            entries.append(entry)
            print(f"  {kind:<9} {rel:<38} {entry['pages']:>3} pages  {entry['size']:>7}  {entry['title']}")
        DATA.write_text(json.dumps(entries, ensure_ascii=False, indent=2) + "\n")
        print(f"  {len(entries)} example(s) → {OUT.relative_to(ROOT)}, {DATA.relative_to(ROOT)}")
        if hero:
            make_hero(wf, work, entries)
    finally:
        shutil.rmtree(work, ignore_errors=True)


# ── The README's hero: one source, three outputs ──────────────────────────


def chrome():
    for name in (os.environ.get("WF_CHROME"), "google-chrome", "google-chrome-stable",
                 "chromium", "chromium-browser"):
        if name and shutil.which(name):
            return shutil.which(name)
    sys.exit("showcase: --hero needs Chrome or Chromium (or WF_CHROME)")


def screenshot(browser, url, dest, width, height):
    run([browser, "--headless=new", "--disable-gpu", "--hide-scrollbars",
         "--force-device-scale-factor=1", f"--window-size={width},{height}",
         "--virtual-time-budget=4000", f"--screenshot={dest}", url])


def make_hero(wf, work, entries):
    """The Halyard report as a web page (its project built as a site), page
    1 of the PDF the same project builds, and slide 1 of the review deck made
    from the same numbers — laid out by site/art/hero.src.html and
    photographed at 1600x900 as site/art/hero.png."""
    browser = chrome()
    art = ROOT / "site" / "art"
    shots = work / "hero"
    shots.mkdir()
    by_slug = {e["slug"]: e for e in entries}
    report, deck = by_slug["halyard-report"], by_slug["halyard-review"]

    # The report, built as a site: the same source, `output_type` switched.
    web = work / "documents" / "halyard-report"
    config = json.loads((web / "webfluent.app.json").read_text())
    config["build"].update(output_type="spa", ssg=True, output="./web")
    (web / "webfluent.app.json").write_text(json.dumps(config))
    run([wf, "build", "-d", str(web)])
    # Its pages address their files from the root, so it is served, not opened.
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(web / "web"))
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        screenshot(browser, f"http://127.0.0.1:{server.server_port}/", shots / "full.png", 1160, 1400)
    finally:
        server.shutdown()
    # From the cover's headline down: the cover is a full screen tall, and
    # its top is mostly sky.
    run(["magick", str(shots / "full.png"), "-crop", "1160x1000+0+270", "+repage", str(shots / "web.png")])

    run(["pdftoppm", "-png", "-singlefile", "-f", "1", "-l", "1", "-scale-to-x", "708",
         "-scale-to-y", "-1", str(ROOT / "site" / "public" / report["file"].lstrip("/")), str(shots / "pdf")])
    run(["pdftoppm", "-png", "-singlefile", "-f", "1", "-l", "1", "-scale-to-x", "940",
         "-scale-to-y", "-1", str(ROOT / "site" / "public" / deck["file"].lstrip("/")), str(shots / "slide")])
    fonts = EXAMPLES / "documents" / "fonts"
    for font in ("Inter.ttf", "JetBrainsMono.ttf"):
        shutil.copyfile(fonts / font, shots / font)

    html = (art / "hero.src.html").read_text()
    html = html.replace("{{PDF_PAGES}}", str(report["pages"])).replace("{{SLIDES}}", str(deck["pages"]))
    (shots / "hero.html").write_text(html)
    screenshot(browser, (shots / "hero.html").as_uri(), shots / "hero.png", 1600, 900)
    run(["magick", str(shots / "hero.png"), "-strip", "-define", "png:exclude-chunks=date,time",
         "-define", "png:compression-level=9", str(art / "hero.png")])
    print(f"  hero → {(art / 'hero.png').relative_to(ROOT)}")


if __name__ == "__main__":
    main()
