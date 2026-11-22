# Benchmarks

> WebFluent against Next.js, Astro, SvelteKit, Nuxt and React, measured.

The page: https://webfluent.monzeromer.dev/benchmarks

Every number comes from a script under [`bench/`](https://github.com/monzeromer-lab/WebFluent/tree/master/bench), run on one machine; the raw results sit beside each script.

- Lighthouse for a new `wf init -t static` site: 100 in every category
- Built in with no package: 12 / 12 (9 / 12 with nothing done, three more with `meta.site_url`); the most any other framework builds in: 5 / 12
- Production build: 31 ms, no node_modules (the others: 434 ms–4.5 s)
- PDF, cold: 7.2× faster than Puppeteer and Chrome
- Runtime, geometric mean against vanilla JS: 1.85×
- Tokens for an agent to build the Pokédex app: 15.6M (the twelve others: 3.9M–9.8M)

## Lighthouse, mobile, the median of the runs

| Measure | WebFluent (static) | WebFluent (SPA) | Astro | Next.js | SvelteKit | Nuxt | React + Vite |
|---|---|---|---|---|---|---|---|
| Performance | 100 | 100 | 100 | 98 | 100 | 98 | 100 |
| Accessibility | 100 | 100 | 100 | 100 | 94 | 90 | 94 |
| Best practices | 100 | 100 | 100 | 100 | 100 | 100 | 100 |
| SEO | 100 | 100 | 90 | 100 | 82 | 91 | 75 |
| Largest contentful paint | 1.1 s | 1.4 s | 901 ms | 2.3 s | 1.2 s | 2.1 s | 1.5 s |
| JavaScript sent | 20.4 kB | 26.1 kB | 0.0 kB | 140.2 kB | 35.4 kB | 156.4 kB | 67.3 kB |

## What each framework provides with no third-party package, by its own docs

| Measure | WebFluent | Astro | Next.js | SvelteKit | Nuxt | React + Vite |
|---|---|---|---|---|---|---|
| The page says its language | [✓ Built in](https://webfluent.monzeromer.dev/learn/05-pages-and-routing) | [✓ Built in](https://github.com/withastro/astro/blob/main/examples/basics/src/layouts/Layout.astro) | [✓ Built in](https://github.com/vercel/next.js/blob/canary/packages/create-next-app/templates/app-tw/ts/app/layout.tsx) | [✓ Built in](https://svelte.dev/docs/kit/accessibility) | [✓ One setting](https://nuxt.com/docs/4.x/getting-started/seo-meta) | [✓ Built in](https://github.com/vitejs/vite/blob/main/packages/create-vite/template-react/index.html) |
| A title | [✓ Built in](https://webfluent.monzeromer.dev/learn/05-pages-and-routing) | [✓ Built in](https://github.com/withastro/astro/blob/main/examples/basics/src/layouts/Layout.astro) | [Built-in API](https://nextjs.org/docs/app/api-reference/functions/generate-metadata#title) | [By hand](https://svelte.dev/docs/kit/seo) | [Built-in API](https://nuxt.com/docs/4.x/getting-started/seo-meta) | [✓ Built in](https://react.dev/reference/react-dom/components/title) |
| A description for search results | [✓ Built in](https://webfluent.monzeromer.dev/recipes/search-and-link-previews) | [By hand](https://docs.astro.build/en/basics/layouts/) | [Built-in API](https://nextjs.org/docs/app/api-reference/functions/generate-metadata#description) | [By hand](https://svelte.dev/docs/kit/seo) | [Built-in API](https://nuxt.com/docs/4.x/getting-started/seo-meta) | [By hand](https://react.dev/reference/react-dom/components/meta) |
| The content is in the HTML | [✓ Built in](https://webfluent.monzeromer.dev/learn/05-pages-and-routing) | [✓ Built in](https://docs.astro.build/en/concepts/islands/) | [✓ Built in](https://nextjs.org/docs/app/getting-started/server-and-client-components) | [✓ Built in](https://svelte.dev/docs/kit/seo) | [✓ Built in](https://nuxt.com/docs/4.x/guide/concepts/rendering) | [By hand](https://vite.dev/guide/ssr) |
| A link preview (Open Graph) | [✓ Built in](https://webfluent.monzeromer.dev/recipes/search-and-link-previews) | [By hand](https://docs.astro.build/en/reference/api-reference/#url) | [Built-in API](https://nextjs.org/docs/app/api-reference/functions/generate-metadata#opengraph) | [By hand](https://svelte.dev/docs/kit/seo) | [Built-in API](https://nuxt.com/docs/4.x/getting-started/seo-meta) | [By hand](https://react.dev/reference/react-dom/components/meta) |
| A skip link | [✓ Built in](https://webfluent.monzeromer.dev/about/accessibility) | [By hand](https://docs.astro.build/en/basics/layouts/) | [By hand](https://nextjs.org/docs/architecture/accessibility) | [By hand](https://svelte.dev/docs/kit/accessibility) | [By hand](https://nuxt.com/docs/4.x/api/components/nuxt-route-announcer) | [By hand](https://react.dev/reference/react-dom/components) |
| A content security policy | [✓ Built in](https://webfluent.monzeromer.dev/about/security) | [✓ One setting](https://docs.astro.build/en/reference/configuration-reference/#securitycsp) | [By hand](https://nextjs.org/docs/app/guides/content-security-policy) | [✓ One setting](https://svelte.dev/docs/kit/configuration) | [By hand](https://nuxt.com/docs/4.x/guide/concepts/rendering) | [By hand](https://vite.dev/guide/features#content-security-policy-csp) |
| A rule against framing | [✓ Built in](https://webfluent.monzeromer.dev/about/security) | [By hand](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/frame-ancestors) | [By hand](https://nextjs.org/docs/app/api-reference/config/next-config-js/headers) | [✓ One setting](https://svelte.dev/docs/kit/configuration) | [By hand](https://nuxt.com/docs/4.x/guide/concepts/rendering) | [By hand](https://vite.dev/guide/static-deploy) |
| robots.txt | [✓ Built in](https://webfluent.monzeromer.dev/learn/01-getting-started) | [By hand](https://docs.astro.build/en/guides/integrations-guide/sitemap/) | [Built-in API](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/robots) | [✓ Built in](https://svelte.dev/docs/kit/project-structure) | [✓ Built in](https://github.com/nuxt/starter/tree/v4/public) | [By hand](https://vite.dev/guide/assets#the-public-directory) |
| sitemap.xml | [✓ One setting](https://webfluent.monzeromer.dev/recipes/search-and-link-previews) | [Official package](https://docs.astro.build/en/guides/integrations-guide/sitemap/) | [Built-in API](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/sitemap) | [By hand](https://svelte.dev/docs/kit/seo) | [By hand](https://nuxt.com/modules/sitemap) | [By hand](https://vite.dev/guide/assets#the-public-directory) |
| A canonical link | [✓ One setting](https://webfluent.monzeromer.dev/recipes/search-and-link-previews) | [By hand](https://docs.astro.build/en/reference/configuration-reference/#site) | [Built-in API](https://nextjs.org/docs/app/api-reference/functions/generate-metadata#alternates) | [By hand](https://svelte.dev/docs/kit/seo) | [By hand](https://nuxt.com/docs/4.x/api/composables/use-head) | [By hand](https://react.dev/reference/react-dom/components/link) |
| Structured data (JSON-LD) | [✓ One setting](https://webfluent.monzeromer.dev/recipes/search-and-link-previews) | [By hand](https://docs.astro.build/en/reference/directives-reference/#sethtml) | [By hand](https://nextjs.org/docs/app/guides/json-ld) | [By hand](https://svelte.dev/docs/kit/seo) | [By hand](https://nuxt.com/docs/4.x/getting-started/seo-meta) | [By hand](https://react.dev/reference/react-dom/components/script) |
| Built in or one setting | 12 / 12 | 4 / 12 | 2 / 12 | 5 / 12 | 3 / 12 | 2 / 12 |

## Install, build and dev server, the median of the runs

| Measure | WebFluent | Astro | Next.js | SvelteKit | Nuxt | React + Vite |
|---|---|---|---|---|---|---|
| npm install | 0 ms | 2.3 s | 4.4 s | 1.6 s | 3.2 s | 913 ms |
| Packages installed | 0 | 243 | 646 | 63 | 600 | 35 |
| node_modules on disk | 0 | 272 MiB | 516 MiB | 100 MiB | 188 MiB | 138 MiB |
| Production build | 31 ms | 837 ms | 4.5 s | 2.2 s | 4.0 s | 434 ms |
| Dev server ready | 55 ms | 2.0 s | 2.7 s | 1.2 s | 3.1 s | 385 ms |
| JavaScript shipped (gzip) | 28.8 kB | 0.0 kB | 176.1 kB | 32.4 kB | 61.6 kB | 66.8 kB |

## js-framework-benchmark, keyed, the median in ms (lower is better)

| Measure | WebFluent 7.1.1 | Vanilla JS | Solid 1.9.3 | Svelte 5.42.1 | Vue 3.5.39 | Preact 11.0.0 | React 19.2.0 |
|---|---|---|---|---|---|---|---|
| Create 1,000 rows | 111.3 ms | 58.4 ms | 62.8 ms | 65.3 ms | 71.7 ms | 76.6 ms | 73.3 ms |
| Replace all 1,000 rows | 117.7 ms | 63.0 ms | 69.1 ms | 73.7 ms | 76.3 ms | 82.9 ms | 83.0 ms |
| Update every 10th row | 55.0 ms | 33.4 ms | 34.6 ms | 34.8 ms | 39.1 ms | 59.5 ms | 44.0 ms |
| Select a row | 33.0 ms | 7.5 ms | 9.6 ms | 12.9 ms | 10.3 ms | 37.1 ms | 15.0 ms |
| Swap two rows | 51.9 ms | 37.1 ms | 40.9 ms | 44.1 ms | 41.6 ms | 66.6 ms | 285.7 ms |
| Remove a row | 45.6 ms | 32.6 ms | 33.5 ms | 33.2 ms | 39.4 ms | 47.4 ms | 36.4 ms |
| Create 10,000 rows | 1107.0 ms | 664.0 ms | 690.1 ms | 713.1 ms | 736.5 ms | 788.0 ms | 918.4 ms |
| Append 1,000 rows | 122.2 ms | 71.8 ms | 72.7 ms | 73.7 ms | 79.7 ms | 89.8 ms | 82.8 ms |
| Clear 1,000 rows | 40.6 ms | 22.9 ms | 27.3 ms | 25.3 ms | 32.2 ms | 31.9 ms | 43.6 ms |
| Geometric mean, against vanilla JS | 1.85× | 1.00× | 1.09× | 1.15× | 1.21× | 1.63× | 1.69× |
| First paint | 86.5 ms | 52.0 ms | 56.6 ms | 71.2 ms | 133.7 ms | 59.7 ms | 331.3 ms |
| Size (compressed) | 11.8 kB | 2.5 kB | 4.5 kB | 9.7 kB | 23.3 kB | 5.9 kB | 51.4 kB |
| Memory after 1,000 rows | 4.5 MB | 1.9 MB | 2.7 MB | 2.9 MB | 3.9 MB | 3.3 MB | 4.4 MB |

## One invoice, the median (lower is better)

| Measure | WebFluent | Puppeteer + Chrome | Puppeteer + headless shell |
|---|---|---|---|
| Cold render (a new process) | 77 ms | 557 ms | 374 ms |
| Warm render (in a running process) | 54 ms | 85 ms | 65 ms |
| File size | 40.7 kB | 359.8 kB | 355.4 kB |

## Every measure of the agents' runs, the median

| Framework | Tokens | Cost at list prices | Time | Turns | Lines of source | Passed all tests |
|---|---|---|---|---|---|---|
| Nuxt | 3.9M | $2.82 | 9.0 min | 71 | 1415 | 5/5 |
| Astro | 4.1M | $2.76 | 8.2 min | 73 | 1461 | 5/5 |
| Hono + htmx | 4.3M | $2.67 | 8.6 min | 75 | 1370 | 5/5 |
| Next.js | 5.0M | $3.14 | 10.9 min | 100 | 1539 | 5/5 |
| Vanilla | 5.0M | $2.98 | 10.1 min | 85 | 1492 | 5/5 |
| Qwik | 5.6M | $3.27 | 10.7 min | 89 | 1744 | 5/5 |
| TanStack Start | 7.1M | $3.54 | 10.3 min | 94 | 1697 | 5/5 |
| SolidStart | 7.1M | $3.79 | 11.2 min | 108 | 1630 | 5/5 |
| React Router | 7.6M | $3.81 | 12.5 min | 98 | 1564 | 5/5 |
| SvelteKit | 8.5M | $4.28 | 10.8 min | 103 | 1611 | 5/5 |
| Gracile | 9.0M | $4.65 | 12.1 min | 107 | 1717 | 5/5 |
| Angular | 9.8M | $4.97 | 17.2 min | 117 | 2009 | 5/5 |
| WebFluent 7.1.1 | 15.6M | $6.38 | 15.3 min | 119 | 1602 | 3/3 |

Why the gap: today's models were trained on years of React, Svelte and Next.js code and on none of WebFluent, so an agent learns the language as it goes — reading the skill and checking with the compiler. Every app passed every test; the extra tokens are those turns, not failed builds. Better docs and tools cut the cost from 35M to 15.6M tokens; the rest closes as models are trained on WebFluent, which is why its code is published as an open dataset: https://huggingface.co/datasets/monzeromer-lab/webfluent-code
