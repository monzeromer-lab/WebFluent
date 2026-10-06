// The benchmark invoice as HTML, for Chrome: the twin of ../invoice.wf.
//
// What a typical Puppeteer pipeline does: a template function turns the data
// into an HTML string, which a page loads with setContent and prints.
// `prepare()` is the once-per-process part (reading the stylesheet, the
// fonts and the logo); `render()` is the per-document part.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import QRCode from "qrcode";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");

const money = new Intl.NumberFormat("en-US", { style: "currency", currency: "USD" });
const fmt = (n) => money.format(n);

const esc = (s) =>
  String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

// A CSS string literal, for the footer's text in an @page margin box.
const cssString = (s) => `"${String(s).replace(/["\\]/g, "\\$&")}"`;

/** Read what every render shares: the sheet, with the fonts inlined, and the logo. */
export function prepare() {
  const font = (file) => readFileSync(join(root, "fonts", file)).toString("base64");
  // Both fonts are variable; one face each covers every weight the sheet asks for.
  const faces = `
@font-face { font-family: "Manrope"; font-weight: 200 800; src: url(data:font/ttf;base64,${font("Manrope.ttf")}) format("truetype"); }
@font-face { font-family: "JetBrains Mono"; font-weight: 100 800; src: url(data:font/ttf;base64,${font("JetBrainsMono.ttf")}) format("truetype"); }
`;
  const css = faces + readFileSync(join(here, "invoice.css"), "utf8");
  const logo = readFileSync(join(root, "logo.svg"), "utf8");
  return { css, logo };
}

/** The invoice for `d`, as one self-contained HTML document. */
export async function render(d, { css, logo }) {
  // The same error-correction level (M) and quiet zone (4) as WebFluent's QrCode.
  const qr = await QRCode.toString(d.pay_url, { type: "svg", errorCorrectionLevel: "M", margin: 4, color: { dark: "#000000", light: "#ffffff" } });
  const footer = `@page {
  @bottom-left { content: ${cssString(`${d.studio.name} · ${d.studio.street} · ${d.studio.city} · ${d.studio.email}`)}; }
  @bottom-right { content: ${cssString(`Invoice ${d.number} · page `)} counter(page) " of " counter(pages); }
}`;
  const rows = d.lines
    .map(
      (l) => `<tr>
  <td class="num">${l.n}</td>
  <td><p class="strong">${esc(l.item)}</p><p class="detail">${esc(l.detail)}</p></td>
  <td class="num">${l.qty}</td>
  <td>${esc(l.unit)}</td>
  <td class="num">${fmt(l.price)}</td>
  <td class="num">${fmt(l.amount)}</td>
</tr>`,
    )
    .join("\n");

  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Invoice ${esc(d.number)} — ${esc(d.studio.name)}</title>
<style>${css}
${footer}</style>
</head>
<body>
<div class="watermark">PAID</div>

<header class="letterhead">
  <div class="brand">
    <img alt="" src="data:image/svg+xml;utf8,${encodeURIComponent(logo)}">
    <div class="brand-text">
      <p class="brand-name">${esc(d.studio.name)}</p>
      <p class="brand-tag">${esc(d.studio.tagline)}</p>
    </div>
  </div>
  <div class="title">
    <h1>Invoice</h1>
    <p class="mono muted">No. ${esc(d.number)}</p>
  </div>
</header>

<section class="parties">
  <div class="party">
    <p class="label">From</p>
    <p class="strong">${esc(d.studio.name)}</p>
    <p>${esc(d.studio.street)}</p>
    <p>${esc(d.studio.city)}</p>
    <p>${esc(d.studio.ein)}</p>
  </div>
  <div class="party">
    <p class="label">Bill to</p>
    <p class="strong">${esc(d.client.name)}</p>
    <p>${esc(d.client.attn)}</p>
    <p>${esc(d.client.street)}</p>
    <p>${esc(d.client.city)}</p>
  </div>
  <div class="meta">
    <div class="meta-row"><span class="muted">Issued</span><span>${esc(d.issued)}</span></div>
    <div class="meta-row"><span class="muted">Terms</span><span>${esc(d.terms)}</span></div>
    <div class="meta-row"><span class="muted">PO</span><span class="mono">${esc(d.po)}</span></div>
    <div class="meta-row"><span class="muted">Due</span><span class="due">${esc(d.due)}</span></div>
  </div>
</section>

<table class="items">
  <thead><tr>
    <th class="num">#</th><th>Description</th><th class="num">Qty</th><th>Unit</th><th class="num">Rate</th><th class="num">Amount</th>
  </tr></thead>
  <tbody>
${rows}
  </tbody>
</table>

<section class="summary">
  <div class="notes">
    <p class="label">Notes</p>
    ${d.notes.map((n) => `<p>${esc(n)}</p>`).join("\n    ")}
  </div>
  <div class="totals">
    <div class="totals-row"><span>Subtotal</span><span>${fmt(d.subtotal)}</span></div>
    <div class="totals-row"><span>Discount (5%)</span><span>−${fmt(d.discount)}</span></div>
    <div class="totals-row"><span>Sales tax (8.75%)</span><span>${fmt(d.tax)}</span></div>
    <div class="totals-row grand"><span>Total due</span><span>${fmt(d.total)}</span></div>
  </div>
</section>

<section class="pay">
  <div class="qr">${qr}</div>
  <div class="pay-text">
    <p class="label">Pay online or by transfer</p>
    <p>Scan to pay by card or ACH, or transfer to the account below with the invoice number as the reference.</p>
    <div class="bank">
      <span class="muted">Bank</span><span>${esc(d.bank.name)}</span>
      <span class="muted">Account</span><span class="mono">${esc(d.bank.account)}</span>
      <span class="muted">Routing</span><span class="mono">${esc(d.bank.routing)}</span>
      <span class="muted">Reference</span><span class="mono">${esc(d.number)}</span>
    </div>
  </div>
</section>
</body>
</html>`;
}

/** The options every print uses: the paper and margins come from the sheet's @page rule. */
export const pdfOptions = { preferCSSPageSize: true, printBackground: true };
