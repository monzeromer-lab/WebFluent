## Configuration Reference (webfluent.app.json)

```json
{
    "name": "My App",
    "version": "1.0.0",
    "author": "Name",
    "theme": {
        "name": "Brand",
        "dark": null,
        "tokens": {},
        "builtin": "full"
    },
    "build": {
        "output": "./build",
        "minify": true,
        "ssg": false,
        "base_path": "",
        "clean_urls": null,
        "csp": false,
        "split": true,
        "compress": true,
        "sourcemap": false,
        "runtime": "auto",
        "budget": {},
        "media": { "formats": ["webp"], "widths": [480, 960, 1440, 1920], "quality": 78, "pipeline": true },
        "output_type": "spa",
        "elements": [],
        "pdf": {
            "page_size": "A4",
            "margins": { "top": 72, "bottom": 72, "left": 72, "right": 72 },
            "default_font": "Helvetica",
            "default_font_size": 12,
            "output_filename": null,
            "fonts": [],
            "system_fonts": true
        },
        "slides": {
            "size": "16:9",
            "width": null,
            "height": null,
            "default_font": "Helvetica",
            "default_font_size": 24,
            "margin": 60,
            "show_slide_numbers": false,
            "footer_text": null,
            "background_color": null,
            "chrome_color": null,
            "output_filename": null,
            "fonts": [],
            "system_fonts": true
        }
    },
    "dev": { "port": 3000, "hot_reload": true },
    "motion": { "duration": "180ms", "easing": "$ease-standard" },
    "meta": {
        "title": "",
        "description": "",
        "favicon": "",
        "touch_icon": "",
        "lang": "en",
        "site_url": "",
        "site_name": "",
        "owner": "organization",
        "same_as": [],
        "job_title": "",
        "owner_details": {},
        "image": "",
        "image_alt": "",
        "sitemap": true,
        "fonts": [],
        "preload": [],
        "stylesheets": [],
        "scripts": [],
        "connect": [],
        "img": [],
        "frame": [],
        "wasm": false,
        "style_elements": false,
        "integrity": {}
    },
    "env": {},
    "public_env": [],
    "lints": {},
    "apis": {},
    "offline": {
        "precache": ["/"],
        "fallback": null,
        "cache": {},
        "sync": false
    },
    "i18n": {
        "default_locale": "en",
        "locales": ["en"],
        "dir": "src/translations",
        "literals": false
    }
}
```

- `theme.dark` names the theme for `prefers-color-scheme: dark` (see Themes).
- `theme.builtin` is `"full"` (the built-ins' own look) or `"structural"`
  (their layout and behaviour only — for a design system that styles
  everything itself).
- `build.output_type` is `"spa"`, `"static"`, `"pdf"`, `"slides"` or
  `"elements"`.
- `apis` is what `wf api` keeps: each service's `spec`, its `out` folder, and
  the operations `skip` leaves out.
- `i18n.literals: true` makes the default locale's text its own key (see
  i18n); `wf config schema` lists every key with its default.

