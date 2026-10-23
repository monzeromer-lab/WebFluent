// Markdown in a real Chrome, built pre-rendered and as a single page: a link
// or picture whose URL a browser would run is drawn as its words and runs
// nothing; `Markdown(text).inert` draws text somebody else wrote with no
// link, no picture and no script, its entities shown as what they say; and
// the page the script draws is the page the build painted.
//
//   cd tests/browser && npm ci && node markdown.mjs
//
// WF (default target/release/wf) and CHROME (default /usr/bin/google-chrome)
// say where they are.

import puppeteer from "puppeteer-core";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { host } from "./serve.mjs";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const WF = process.env.WF || path.join(repo, "target/release/wf");
const CHROME = process.env.CHROME || "/usr/bin/google-chrome";

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}

// Every hostile thing tries to set `window.ran`; nothing may.
const HOSTILE = [
  "[run](javascript:window.ran=1)",
  "[Run](JavaScript:window.ran=2)",
  "[RUN](\u001fjavascript:window.ran=3)",
  "![pic](javascript:window.ran=4)",
  "[vb](vbscript:msgbox)",
  "[page](data:text/html;base64,PHNjcmlwdD53aW5kb3cucmFuPTU8L3NjcmlwdD4=)",
].join(" ");
const DOCS = [
  "Returns &lt;color&gt; &mdash; see [the guide](https://example.com/guide) ![badge](/badge.svg) now.",
  "&lt;script&gt;window.ran = 6&lt;/script&gt; <script>window.ran = 7</script> <img src=x onerror=\\\"window.ran = 8\\\">",
  "```ts\\nconst a: Array&lt;T&gt; = [];\\n```",
  "_Widely_ used; `&amp;` stays.",
].join("\\n\\n");

const SOURCE = `page Home(path: "/", title: "Markdown", description: "Markdown in a browser.") {
    state docs = "${DOCS}"
    Heading("Markdown").h1
    Markdown("${HOSTILE} [guide](/guide) [out](https://example.com/)")
    Markdown(docs).inert
    Markdown("Built &lt;color&gt; [here](/x) ![p](/p.png)").inert
}
`;

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-markdown-site-"));
  fs.mkdirSync(path.join(dir, "src/pages"), { recursive: true });
  // Served from under its base path, as a site published at /site is.
  fs.writeFileSync(path.join(dir, "webfluent.app.json"), JSON.stringify({ name: "markdown", build: { ssg, output: "www/site", base_path: "/site" } }));
  fs.writeFileSync(path.join(dir, "src/pages/Home.wf"), SOURCE);
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
  const site = host(path.join(dir, "www"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  const problems = [];
  page.on("pageerror", (e) => problems.push(e.message));
  // A request that failed is named by its URL below; the browser's own
  // guess at a favicon is not the site's to answer.
  page.on("console", (m) => {
    if ((m.type() === "error" && !/^Failed to load resource/.test(m.text())) || /differs from the page drawn/.test(m.text())) problems.push(m.text());
  });
  page.on("response", (r) => { if (r.status() >= 400 && !/favicon\.ico$/.test(r.url())) problems.push(`${r.status()} ${r.url()}`); });
  try {
    await page.goto(`http://localhost:${port}/site/`, { waitUntil: "networkidle0" });
    const seen = await page.evaluate(() => {
      const boxes = [...document.querySelectorAll(".wf-markdown")];
      const of = (box) => ({
        html: box.innerHTML,
        text: box.textContent,
        links: [...box.querySelectorAll("a")].map((a) => a.getAttribute("href")),
        pictures: [...box.querySelectorAll("img")].map((i) => i.getAttribute("src")),
        scripts: box.querySelectorAll("script").length,
        pre: box.querySelector("pre code")?.textContent ?? null,
        em: box.querySelector("em")?.textContent ?? null,
        code: [...box.querySelectorAll("p code")].map((c) => c.textContent),
      });
      return { boxes: boxes.map(of), ran: window.ran, live: typeof WF === "object" };
    });
    const [plain, docs, built] = seen.boxes;
    check("the script runs, and three Markdown elements are drawn", seen.live && seen.boxes.length === 3, `${seen.live} ${seen.boxes.length}`);

    const hostile = (url) => /^\s*(javascript|vbscript|data):/i.test(url.replace(/[\u0000-\u001f]/g, ""));
    check("no link or picture names a scheme a browser would run",
      ![...plain.links, ...plain.pictures].some(hostile), JSON.stringify([plain.links, plain.pictures]));
    check("a site-relative link takes the base path, another site's is kept",
      JSON.stringify(plain.links) === JSON.stringify(["/site/guide", "https://example.com/"]), JSON.stringify(plain.links));
    check("a refused link or picture is drawn as its words",
      ["run", "Run", "RUN", "pic", "vb", "page"].every((w) => plain.text.includes(w)), plain.text);

    check("inert text has no link, no picture and no script",
      docs.links.length === 0 && docs.pictures.length === 0 && docs.scripts === 0
        && built.links.length === 0 && built.pictures.length === 0, docs.html);
    check("its entities show as what they say", docs.text.includes("Returns <color> — see the guide now.") && built.text.includes("Built <color> here"), docs.text);
    check("HTML in it, written or encoded, is text",
      docs.text.includes("<script>window.ran = 6</script> <script>window.ran = 7</script> <img src=x onerror=\"window.ran = 8\">"), docs.text);
    check("a fence is a pre, its entities as written", docs.pre === "const a: Array&lt;T&gt; = [];\n", JSON.stringify(docs.pre));
    check("_Widely_ is emphasis, and code keeps its entity", docs.em === "Widely" && JSON.stringify(docs.code) === JSON.stringify(["&amp;"]), JSON.stringify([docs.em, docs.code]));

    // A click on each link the page drew runs nothing either.
    await page.evaluate(() => {
      for (const a of document.querySelectorAll(".wf-markdown a")) a.addEventListener("click", (e) => e.preventDefault());
      for (const a of document.querySelectorAll(".wf-markdown a")) a.click();
    });
    const ran = await page.evaluate(() => window.ran);
    check("nothing in any of it ran", seen.ran === undefined && ran === undefined, String(seen.ran ?? ran));

    if (ssg) {
      const painted = fs.readFileSync(path.join(dir, "www/site/index.html"), "utf8");
      const statics = await page.evaluate((html) => [...new DOMParser().parseFromString(html, "text/html").querySelectorAll(".wf-markdown")].map((b) => b.innerHTML), painted);
      check("the script draws what the build painted",
        statics.length === 3 && statics.every((h, i) => h.trim() === seen.boxes[i].html.trim()),
        JSON.stringify(statics.map((h, i) => [h, seen.boxes[i].html]).filter(([a, b]) => a.trim() !== b.trim())));
      check("the pre-rendered page names no scheme a browser would run", !/(href|src)="\s*(javascript|vbscript|data):/i.test(painted));
    }
    check("nothing threw", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4407);
await scenario("A single page", false, 4408);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
