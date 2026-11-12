// What a page writes from a value, in a real Chrome: a `head { }` link
// whose address comes from the URL never names `javascript:`, a frame
// drawing HTML from a value runs none of it, and a frame's `on message`
// hears only the frame — not a page from another origin it navigated to.
//
//   cd tests/browser && npm ci && node output_security.mjs

import puppeteer from "puppeteer-core";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { host } from "./serve.mjs";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const WF = process.env.WF || path.join(repo, "target/release/wf");
const CHROME = process.env.CHROME || "/usr/bin/google-chrome";
const PORT = 4441;
const OTHER = 4442;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-output-security-"));
const write = (rel, text) => {
  fs.mkdirSync(path.dirname(path.join(dir, rel)), { recursive: true });
  fs.writeFileSync(path.join(dir, rel), text);
};
write("webfluent.app.json", JSON.stringify({
  name: "t", build: { minify: false, ssg: false, csp: false, compress: false },
}));
write("src/App.wf", `page Home(path: "/", title: "Home", description: "d") {
    head { link(rel: "icon", href: query.icon ?? "/favicon.png") }
    state doc = "<p>x</p><script>try { parent.document.body.setAttribute('data-pwned', 'srcdoc') } catch (e) {}</script>"
    state heard = ""
    Heading("Home").h1
    Frame(srcdoc: doc, title: "From a value")
    Frame(src: "/inner.html", title: "Inner") { on message(m) { heard = heard + m.data + ";" } }
    Text(heard, id: "heard")
}
`);
// The framed page says hello, then goes to a page of another origin,
// which speaks through the same window.
write("public/inner.html", `<!doctype html><script>
parent.postMessage("own", "*");
setTimeout(() => { location.href = "http://127.0.0.1:${OTHER}/evil.html"; }, 100);
</script>`);
write("public/evil.html", `<!doctype html><script>parent.postMessage("forged", "*");</script>`);

const built = spawnSync(WF, ["build", "-d", dir], { encoding: "utf8" });
const said = `${built.stdout}${built.stderr}`;
check("the build succeeds", built.status === 0, said);
check("the build names the frame drawn from a value (V03)", said.includes("V03"), said);
const site = host(path.join(dir, "build"), PORT);
const other = host(path.join(dir, "build"), OTHER);
await site.start();
await other.start();
const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
const page = await browser.newPage();
const problems = [];
page.on("pageerror", (e) => problems.push(e.message));
const dialogs = [];
page.on("dialog", async (d) => { dialogs.push(d.message()); await d.dismiss(); });
try {
  await page.goto(`http://127.0.0.1:${PORT}/?icon=${encodeURIComponent(" JaVaScRiPt:alert(1)")}`, { waitUntil: "networkidle0" });
  const icon = await page.$eval("link[rel=icon][data-wf-head]", (l) => l.getAttribute("href")).catch(() => null);
  check("a head link from the address never names javascript:", !String(icon).toLowerCase().includes("javascript"), String(icon));

  await page.goto(`http://127.0.0.1:${PORT}/?icon=/x.png`, { waitUntil: "networkidle0" });
  const fine = await page.$eval("link[rel=icon][data-wf-head]", (l) => l.getAttribute("href")).catch(() => null);
  check("…and a site address is written as it is", fine === "/x.png", String(fine));

  const sandbox = await page.$eval("iframe[title='From a value']", (f) => f.getAttribute("sandbox"));
  check("a frame drawing HTML from a value is sandboxed", sandbox === "", String(sandbox));
  await new Promise((r) => setTimeout(r, 600));
  const pwned = await page.$eval("body", (b) => b.getAttribute("data-pwned"));
  check("…so nothing in it runs as the page", pwned === null, String(pwned));

  const heard = await page.$eval("#heard", (t) => t.textContent);
  check("a frame's own message is heard", heard.includes("own;"), heard);
  check("…and not a page of another origin it went to", !heard.includes("forged"), heard);
  check("nothing threw", problems.length === 0, problems.join(" | "));
  check("no dialog opened", dialogs.length === 0, dialogs.join(" | "));
} catch (e) {
  check(`the run (${e.message})`, false);
} finally {
  await browser.close();
  await site.stop().catch(() => {});
  await other.stop().catch(() => {});
  fs.rmSync(dir, { recursive: true, force: true });
}
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
