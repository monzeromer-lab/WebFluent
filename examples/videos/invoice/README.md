# Video 1 — an invoice, as a PDF

A one-page invoice drawn by the compiler's own PDF engine: the line items come from `src/invoice.json`, the totals are worked out in `src/pages/Invoice.wf`, and the look is `src/invoice.css`. Fonts are the shared ones in `examples/documents/fonts`.

    wf check --deny-warnings && wf build    # → build/invoice.pdf
