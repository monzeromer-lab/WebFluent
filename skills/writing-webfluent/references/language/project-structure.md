## Project Structure

```
project/
├── AGENTS.md                 # This reference — `wf init` writes it, for the version that made the project
├── webfluent.app.json        # Config: theme, build, i18n, meta
├── src/
│   ├── App.wf                # Root: navbar, router, footer
│   ├── pages/                # One .wf per page
│   ├── components/           # Reusable components
│   ├── stores/               # Shared state stores
│   ├── *.css                 # The project's own stylesheets, anywhere under src/
│   ├── *.js                  # The project's own plain browser scripts, anywhere under src/
│   └── translations/         # i18n JSON files (en.json, ar.json)
├── tests/                    # `test "…" { }` declarations that `wf test` runs
├── public/                   # Static assets → copied to build root
└── build/                    # Compiled output
```

Files in `public/` are copied to the **root** of the build output (not nested under `public/`). Example: `public/logo.png` → `build/logo.png`.

