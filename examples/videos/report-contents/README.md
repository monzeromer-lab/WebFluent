# Video 3 — a report with contents, headers and footers

The Halyard quarterly report (from `examples/documents/halyard-report`): a cover, a `TableOfContents` whose page numbers are filled in after layout, a running `Header` that says "page N of M" and a `Footer`, on every page but the cover, and tables whose header row repeats when they break across pages.

    wf check --deny-warnings && wf build    # → build/report.pdf (9 pages)
