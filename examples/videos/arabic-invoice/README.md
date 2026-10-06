# Video 2 — the same invoice, in Arabic

The invoice of video 1, right to left: `Document(lang: "ar")` lays out text, rows, grids and table columns from the right, with Noto Sans Arabic for the text and Noto Kufi Arabic for the headings (the shared fonts in `examples/documents/fonts`). Figures and codes sit in left-to-right runs inside the Arabic.

    wf check --deny-warnings && wf build    # → build/invoice-ar.pdf
