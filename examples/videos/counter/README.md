# Video 4 — the smallest interactive page

A `state`, a heading that shows it and two buttons that change it: the whole program is `src/Counter.wf`. The theme (`src/Theme.wf`) and three CSS rules (`src/counter.css`) give it its look.

    wf check --deny-warnings && wf build    # → build/index.html
    wf serve                                # → http://localhost:3000
