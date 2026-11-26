## SSG (Static Site Generation)

With `ssg: true` the build pre-renders each page to HTML, and JavaScript takes it over in the browser.

```json
{ "build": { "ssg": true, "base_path": "/my-site" } }
```

- Pre-renders each page to its own `index.html`
- Static content visible immediately (no blank screen)
- JavaScript hydrates for interactivity: it takes the painted page over in
  place — an element nothing holds stays as painted, never moved or drawn
  again, so the first paint stays the page's largest
- Dynamic content (state, if/for, a `resource`) handled by JS after hydration

