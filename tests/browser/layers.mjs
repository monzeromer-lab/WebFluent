// Layers in a real Chrome, against `tests/fixtures/layers`, built twice —
// pre-rendered and as a single page. A popover, a menu's list, a submenu, a
// context menu and a tooltip are drawn in the top layer, so the box that
// clips them does not; each is placed at its anchor, or at the pointer;
// Escape and a press outside close the top one and write the state back;
// the arrow keys move through a menu and skip what is disabled; a disabled
// item cannot be chosen; a submenu opens on ArrowRight and closes with its
// menu; a tooltip waits for its delay, shows its shortcut, and goes on
// leaving; and the policy the build ships refuses nothing.
//
//   cd tests/browser && npm ci && node layers.mjs
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
const fixture = path.join(repo, "tests/fixtures/layers");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 3000) =>
  page.waitForFunction(fn, { timeout, polling: 30 }, arg).then(() => true, () => false);

// What the page says about the layers: which are open and in the top layer,
// and where each is against its anchor.
const look = (page) => page.evaluate(() => {
  const rect = (sel) => {
    const el = document.querySelector(sel);
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return { left: Math.round(r.left), top: Math.round(r.top), right: Math.round(r.right), bottom: Math.round(r.bottom) };
  };
  const open = (sel) => {
    const el = document.querySelector(sel);
    if (!el) return null;
    let top = false;
    try { top = el.matches(":popover-open"); } catch { top = false; }
    return { open: el.hasAttribute("data-open"), top, shown: getComputedStyle(el).display !== "none" };
  };
  return {
    said: document.getElementById("said")?.textContent ?? null,
    focus: document.activeElement?.id || document.activeElement?.textContent?.trim() || null,
    pop: open("#pop"), popRect: rect("#pop"), branches: rect("#branches"),
    view: open("#view .wf-menu__items"), viewRect: rect("#view .wf-menu__items"), viewButton: rect("#view > button"),
    sub: open("#appearance > .wf-menu__items"), subRect: rect("#appearance > .wf-menu__items"), appearance: rect("#appearance"),
    ctx: open("#context"), ctxRect: rect("#context"),
    tip: open(".wf-tooltip__text"), tipRect: rect(".wf-tooltip__text"), save: rect("#save"),
    clip: rect("#clip"),
    wrapChecked: document.getElementById("wrap")?.getAttribute("aria-checked") ?? null,
    expanded: document.querySelector("#view > button")?.getAttribute("aria-expanded") ?? null,
  };
});

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-layers-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  const configPath = path.join(dir, "webfluent.app.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  config.build.ssg = ssg;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });

  const site = host(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  await page.setViewport({ width: 900, height: 600 });
  const problems = [];
  page.on("pageerror", (e) => problems.push(e.message));
  page.on("console", (m) => { if (m.type() === "error" && !/^Failed to load resource/.test(m.text())) problems.push(m.text()); });
  page.on("response", (r) => { if (r.status() >= 400 && !/favicon\.ico$/.test(r.url())) problems.push(`${r.status()} ${r.url()}`); });
  const centre = async (sel) => {
    const r = await page.$eval(sel, (el) => { const b = el.getBoundingClientRect(); return { x: b.left + b.width / 2, y: b.top + b.height / 2 }; });
    return r;
  };
  const press = async (sel) => { const c = await centre(sel); await page.mouse.click(c.x, c.y); };
  try {
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    // ── A menu open from the first paint ──
    await until(page, () => document.querySelector("#first")?.hasAttribute("data-open"));
    const first = await page.evaluate(() => {
      const list = document.getElementById("first");
      return { open: list?.hasAttribute("data-open"), top: list?.matches(":popover-open"), button: !!list?.parentElement?.querySelector(":scope > button[aria-haspopup]"), name: list?.getAttribute("aria-label"), tag: list?.tagName };
    });
    check("a menu that state opens from the first paint opens in the top layer", first.open && first.top, JSON.stringify(first));
    check("…draws no button, and is named by its label", !first.button && first.name === "Open from the start" && first.tag === "UL", JSON.stringify(first));
    await page.keyboard.press("Escape");
    await until(page, () => !document.querySelector("#first")?.hasAttribute("data-open"));
    // ── Menus a loop draws ──
    const many = await page.evaluate(() => [...document.querySelectorAll("#many .wf-menu")].map((m) => {
      const b = m.querySelector(":scope > button");
      const id = b?.getAttribute("aria-controls");
      const list = id ? document.getElementById(id) : null;
      return { id, own: !!list && m.contains(list), trigger: b?.classList.contains("wf-menu__trigger") };
    }));
    check("three menus a loop draws have three lists, each named by its own button",
      many.length === 3 && new Set(many.map((m) => m.id)).size === 3 && many.every((m) => m.own && m.trigger), JSON.stringify(many));
    const tips = await page.evaluate(() => [...document.querySelectorAll("#tips .wf-tooltip")].map((t) => {
      const b = t.querySelector("button");
      const tip = t.querySelector(".wf-tooltip__text");
      return { id: tip?.id, describes: b?.getAttribute("aria-describedby"), says: tip?.textContent };
    }));
    check("three tooltips a loop draws have three ids, each describing its own trigger",
      tips.length === 3 && new Set(tips.map((t) => t.id)).size === 3 && tips.every((t) => t.id && t.describes === t.id),
      JSON.stringify(tips));
    let s = await look(page);
    check("nothing is open at first", !s.pop.shown && !s.view.shown && !s.ctx.shown && !s.tip.shown, JSON.stringify([s.pop, s.view, s.ctx, s.tip]));

    // ── Popover ──
    await press("#branches");
    await until(page, () => document.querySelector("#pop")?.hasAttribute("data-open"));
    s = await look(page);
    check("a popover opens in the top layer", s.pop.open && s.pop.top && s.pop.shown, JSON.stringify(s.pop));
    check("below its anchor, from its start",
      s.popRect && Math.abs(s.popRect.left - s.branches.left) <= 1 && s.popRect.top >= s.branches.bottom && s.popRect.top - s.branches.bottom <= 10,
      JSON.stringify([s.popRect, s.branches]));
    // Scrolled out of view, its anchor takes it along; back, it returns.
    const away = await page.evaluate(async () => {
      const frames = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
      const pop = document.querySelector("#pop");
      const spacer = document.createElement("div");
      spacer.style.height = "3000px";
      document.body.appendChild(spacer);
      scrollTo({ top: 2000, behavior: "instant" });
      await frames();
      const gone = { hidden: getComputedStyle(pop).visibility === "hidden", marked: pop.hasAttribute("data-anchor-hidden"), open: pop.hasAttribute("data-open"), y: scrollY };
      scrollTo({ top: 0, behavior: "instant" });
      await frames();
      const back = { hidden: getComputedStyle(pop).visibility === "hidden", marked: pop.hasAttribute("data-anchor-hidden") };
      spacer.remove();
      return { gone, back };
    });
    check("scrolled out of view, its anchor takes it along, still open",
      away.gone.hidden && away.gone.marked && away.gone.open, JSON.stringify(away.gone));
    check("…and scrolled back, it shows again", !away.back.hidden && !away.back.marked, JSON.stringify(away.back));
    await page.keyboard.press("Escape");
    await until(page, () => !document.querySelector("#pop")?.hasAttribute("data-open"));
    s = await look(page);
    check("Escape closes it, writes the state back and says why", !s.pop.shown && s.said === "popover escape", s.said);
    await press("#branches");
    await until(page, () => document.querySelector("#pop")?.hasAttribute("data-open"));
    await page.mouse.click(850, 550);
    await until(page, () => !document.querySelector("#pop")?.hasAttribute("data-open"));
    s = await look(page);
    check("a press outside closes it", !s.pop.shown && s.said === "popover outside", s.said);
    await press("#branches");
    await until(page, () => document.querySelector("#pop")?.hasAttribute("data-open"));
    await press("#branches");
    await until(page, () => !document.querySelector("#pop")?.hasAttribute("data-open"));
    s = await look(page);
    check("a press on its anchor is the anchor's: it toggles it shut", !s.pop.shown, JSON.stringify(s.pop));

    // ── Menu ──
    await press("#view > button");
    await until(page, () => document.querySelector("#view .wf-menu__items")?.hasAttribute("data-open"));
    s = await look(page);
    check("a menu's list opens in the top layer, past the box that clips it",
      s.view.top && s.view.shown && s.viewRect.bottom > s.clip.bottom, JSON.stringify([s.view, s.viewRect, s.clip]));
    check("at its button", Math.abs(s.viewRect.left - s.viewButton.left) <= 1 && s.viewRect.top >= s.viewButton.bottom, JSON.stringify([s.viewRect, s.viewButton]));
    check("aria-expanded says so", s.expanded === "true", s.expanded);
    const shortcut = await page.$eval("#sidebar", (el) => [el.querySelector("kbd")?.textContent, el.getAttribute("aria-keyshortcuts")]);
    check("an item shows its shortcut, and says it", shortcut[0] === "Ctrl+B" && shortcut[1] === "Control+B", JSON.stringify(shortcut));
    await press("#zen");
    s = await look(page);
    check("a disabled item cannot be chosen, and the menu stays", s.said !== "zen" && s.view.shown, `${s.said} ${JSON.stringify(s.view)}`);
    await press("#wrap");
    await until(page, () => !document.querySelector("#view .wf-menu__items")?.hasAttribute("data-open"));
    s = await look(page);
    check("choosing an item runs it and closes the menu", s.wrapChecked === "true" && !s.view.shown, `${s.wrapChecked} ${JSON.stringify(s.view)}`);
    check("focus is back on the button", s.focus === "View", s.focus);

    await page.focus("#view > button");
    await page.keyboard.press("ArrowDown");
    await until(page, () => document.activeElement?.id === "sidebar");
    s = await look(page);
    check("ArrowDown on the button opens it with the first item focused", s.view.shown && s.focus === "sidebar", s.focus);
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("ArrowDown");
    s = await look(page);
    check("the arrow keys skip a disabled item", s.focus === "appearance", s.focus);
    await page.keyboard.press("ArrowRight");
    await until(page, () => document.activeElement?.id === "light");
    s = await look(page);
    check("ArrowRight opens the submenu, in the top layer, beside its item",
      s.sub.top && s.sub.shown && s.focus === "light" && s.subRect.left >= s.appearance.right - 4,
      JSON.stringify([s.sub, s.focus, s.subRect, s.appearance]));
    await page.keyboard.press("ArrowDown");
    s = await look(page);
    check("its arrow keys move in it, not in its menu", s.focus === "dark", s.focus);
    await page.keyboard.press("Escape");
    await until(page, () => !document.querySelector("#appearance > .wf-menu__items")?.hasAttribute("data-open"));
    s = await look(page);
    check("Escape closes the submenu first, back on its item", !s.sub.shown && s.view.shown && s.focus === "appearance", JSON.stringify([s.sub, s.view, s.focus]));
    await page.keyboard.press("ArrowRight");
    await until(page, () => document.activeElement?.id === "light");
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");
    await until(page, () => !document.querySelector("#view .wf-menu__items")?.hasAttribute("data-open"));
    s = await look(page);
    check("Enter in the submenu chooses, and closes the whole menu", s.said === "dark" && !s.view.shown && !s.sub.shown, JSON.stringify([s.said, s.view, s.sub]));

    await press("#view > button");
    await until(page, () => document.querySelector("#view .wf-menu__items")?.hasAttribute("data-open"));
    await page.hover("#appearance");
    await until(page, () => document.querySelector("#appearance > .wf-menu__items")?.hasAttribute("data-open"));
    s = await look(page);
    check("a hover opens the submenu", s.sub.shown, JSON.stringify(s.sub));
    await page.mouse.click(850, 550);
    await until(page, () => !document.querySelector("#view .wf-menu__items")?.hasAttribute("data-open"));
    s = await look(page);
    check("a press on the page closes the submenu and its menu", !s.sub.shown && !s.view.shown, JSON.stringify([s.sub, s.view]));

    // ── Context menu ──
    const area = await centre("#area");
    await page.mouse.click(area.x + 20, area.y, { button: "right" });
    await until(page, () => document.querySelector("#context")?.hasAttribute("data-open"));
    s = await look(page);
    check("a context menu opens at the pointer, in the top layer",
      s.ctx.top && s.ctx.shown && Math.abs(s.ctxRect.left - Math.round(area.x + 20)) <= 6 && Math.abs(s.ctxRect.top - Math.round(area.y)) <= 6,
      JSON.stringify([s.ctxRect, area]));
    check("with its first item focused", s.focus === "definition", s.focus);
    // Near the right edge there is no room after the pointer: it slides
    // back to stay on screen.
    const edge = await page.$eval("#area", (el) => { const b = el.getBoundingClientRect(); return { x: b.right - 4, y: b.top + b.height / 2 }; });
    await page.mouse.click(edge.x, edge.y, { button: "right" });
    await until(page, () => document.querySelector("#context")?.hasAttribute("data-open"));
    s = await look(page);
    check("near an edge it stays on screen, and moves with the pointer",
      s.ctx.shown && s.ctxRect.right <= 900 - 8 + 1 && s.ctxRect.left < edge.x && Math.abs(s.ctxRect.top - Math.round(edge.y)) <= 6,
      JSON.stringify([s.ctxRect, edge]));
    await press("#rename");
    await until(page, () => !document.querySelector("#context")?.hasAttribute("data-open"));
    s = await look(page);
    check("choosing closes it and writes the state back", !s.ctx.shown && s.said === "context choose", s.said);

    // ── Tooltip ──
    await page.mouse.move(10, 590);
    await page.hover("#save");
    await new Promise((r) => setTimeout(r, 80));
    s = await look(page);
    check("a tooltip waits for its delay", !s.tip.shown, JSON.stringify(s.tip));
    await until(page, () => document.querySelector(".wf-tooltip__text")?.hasAttribute("data-open"));
    s = await look(page);
    check("then shows in the top layer, past the box that clips it", s.tip.top && s.tip.shown && s.tipRect.bottom <= s.save.top, JSON.stringify([s.tip, s.tipRect, s.save]));
    const chip = await page.$eval(".wf-tooltip__text kbd", (el) => el.textContent).catch(() => null);
    check("with its shortcut chip", chip === "Ctrl+S", chip);
    const keys = await page.$eval("#save", (b) => b.getAttribute("aria-keyshortcuts"));
    check("and its control says the shortcut to assistive technology, in ARIA's spelling", keys === "Control+S", keys);
    await page.mouse.move(10, 590);
    await until(page, () => !document.querySelector(".wf-tooltip__text")?.hasAttribute("data-open"));
    s = await look(page);
    check("leaving hides it", !s.tip.shown, JSON.stringify(s.tip));
    await page.focus("#branches");
    await page.keyboard.press("Tab");
    await page.keyboard.press("Tab");
    await until(page, () => document.querySelector(".wf-tooltip__text")?.hasAttribute("data-open"));
    s = await look(page);
    check("keyboard focus shows it at once", s.tip.shown && s.focus === "save", JSON.stringify([s.tip, s.focus]));
    await page.keyboard.press("Escape");
    s = await look(page);
    check("and Escape hides it", !s.tip.shown, JSON.stringify(s.tip));

    // ── Escape is the top one's: a modal's is not heard behind it ──
    await page.mouse.move(5, 5);
    const heard = async () => Number(await page.$eval("#escapes", (e) => e.textContent));
    const before = await heard();
    await press("#modal-open");
    await until(page, () => document.querySelector("dialog.wf-modal")?.open);
    await page.keyboard.press("Escape");
    await until(page, () => !document.querySelector("dialog.wf-modal")?.open);
    check("Escape closes an open modal, and the page behind it does not hear it",
      !(await page.$eval("dialog.wf-modal", (d) => d.open)) && (await heard()) === before, `${before} → ${await heard()}`);
    await page.keyboard.press("Escape");
    check("with nothing open, the page hears Escape", (await heard()) === before + 1, `${before} → ${await heard()}`);
    check("nothing threw, and the policy refused nothing", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4391);
await scenario("A single page", false, 4392);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
