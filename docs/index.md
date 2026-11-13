# WebFluent

> One language, one binary: a static site, a single-page app, a PDF or a slide deck from the same file. No Node, and no browser for your PDFs.

The page: https://webfluent.monzeromer.dev/

WebFluent compiles one typed `.wf` file into a static site, a web app, a PDF or a slide deck. Start with [the book](https://webfluent.monzeromer.dev/learn.md), find a task in [the recipes](https://webfluent.monzeromer.dev/recipes.md), look a built-in up in [the reference](https://webfluent.monzeromer.dev/reference.md), or find a compiler message in [the errors](https://webfluent.monzeromer.dev/errors.md). The map for agents is [llms.txt](https://webfluent.monzeromer.dev/llms.txt).

```bash
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
wf init my-site -t static
cd my-site && wf serve
```
