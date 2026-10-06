// Warm renders through Puppeteer, as its documentation recommends for a
// server: launch the browser once and keep it, then for each document load
// the HTML with setContent and print it.
//
//   node chrome/warm.mjs <data.json> <out.pdf> [renders] [warmup] [chrome|shell] [reuse|new]
//
// `reuse` keeps one tab and loads every document into it; `new` opens a tab
// per document and closes it afterwards (isolation between documents, at a
// cost). The browser launch is timed separately and not counted in a render.
//
// Prints one JSON object shaped like the Rust bench's.

import { readFileSync, writeFileSync } from "node:fs";
import puppeteer from "puppeteer";
import { pdfOptions, prepare, render } from "./template.mjs";

const [dataPath, outPath, n = "100", w = "5", mode = "chrome", tabs = "reuse"] = process.argv.slice(2);
const renders = Number(n);
const warmup = Number(w);
const data = JSON.parse(readFileSync(dataPath, "utf8"));

let t = performance.now();
const shared = prepare();
const browser = await puppeteer.launch({ headless: mode === "shell" ? "shell" : true });
const setup_ms = performance.now() - t;

let page = tabs === "reuse" ? await browser.newPage() : null;
async function once() {
  const p = page ?? (await browser.newPage());
  await p.setContent(await render(data, shared), { waitUntil: "load" });
  const pdf = await p.pdf(pdfOptions);
  if (!page) await p.close();
  return pdf;
}

let pdf;
for (let i = 0; i < warmup; i++) pdf = await once();
const times = [];
const all = performance.now();
for (let i = 0; i < renders; i++) {
  t = performance.now();
  pdf = await once();
  times.push(performance.now() - t);
}
const total_ms = performance.now() - all;
const version = await browser.version();
const executable = browser.process()?.spawnfile;
await browser.close();
writeFileSync(outPath, pdf);

console.log(
  JSON.stringify({
    setup_ms: +setup_ms.toFixed(3),
    total_ms: +total_ms.toFixed(3),
    warmup,
    bytes: pdf.length,
    browser: version,
    executable,
    times_ms: times.map((x) => +x.toFixed(3)),
  }),
);
