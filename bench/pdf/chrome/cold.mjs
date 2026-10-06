// One cold render: a fresh Node process that launches Chrome, prints the
// invoice to a PDF file, and exits. What `wf render` is compared with.
//
//   node chrome/cold.mjs <data.json> <out.pdf> [chrome|shell]
//
// `chrome` (the default) is Puppeteer's default browser in its default
// headless mode; `shell` is chrome-headless-shell, which Puppeteer also
// downloads and which is lighter for printing.

import { readFileSync, writeFileSync } from "node:fs";
import puppeteer from "puppeteer";
import { pdfOptions, prepare, render } from "./template.mjs";

const [dataPath, outPath, mode = "chrome"] = process.argv.slice(2);
const data = JSON.parse(readFileSync(dataPath, "utf8"));

const browser = await puppeteer.launch({ headless: mode === "shell" ? "shell" : true });
try {
  const page = await browser.newPage();
  await page.setContent(await render(data, prepare()), { waitUntil: "load" });
  writeFileSync(outPath, await page.pdf(pdfOptions));
} finally {
  await browser.close();
}
