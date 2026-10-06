// Google Analytics 4 with Consent Mode v2: the set-up Google's snippet does
// in an inline <script>, which the site's policy refuses, and the
// measurement on top of it. The library is `meta.scripts` (async); where it
// sends data is `meta.connect` and `meta.img`; the banner that asks is
// `ConsentBanner`. Both of the author's sites run this same file: only the
// four constants below differ.
//
// What it sends:
//   page_view       once per page drawn (`wf:render`), with its final title
//   generate_lead   a mailto: link, or an element marked data-track="generate_lead"
//   <data-track>    any element marked data-track="event", with data-track-* params
//   search          what is typed into a search box, once the reader pauses
//   LCP CLS INP FCP TTFB   the page's Core Web Vitals, from this visit
//
// Consent: the advertising signals are always denied (no ads). Analytics
// storage starts denied in the EEA, the UK and Switzerland and granted
// elsewhere, until the reader answers the banner; their answer then holds
// everywhere. Declining after allowing deletes Analytics' cookies.
//
// For the author: `?ga_internal=1` marks this browser as internal traffic
// (filter it in Admin → Data filters), `?ga_internal=0` undoes it, and
// `?ga_debug=1` sends this tab's hits to DebugView.

const GA_ID = "G-HPDHSJ9S4G";
const GA_HOST = "webfluent.monzeromer.dev";
const GA_CONSENT_KEY = "wf:analytics-consent";
const GA_INTERNAL_KEY = "wf:analytics-internal";

// Where consent is required before Analytics may store anything: the EEA,
// the United Kingdom and Switzerland.
const GA_ASK_FIRST = [
  "AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR", "DE", "GR", "HU",
  "IE", "IT", "LV", "LT", "LU", "MT", "NL", "PL", "PT", "RO", "SK", "SI", "ES",
  "SE", "IS", "LI", "NO", "GB", "CH",
];

window.dataLayer = window.dataLayer || [];
function gtag() {
  // gtag.js reads the `arguments` object itself, not an array.
  window.dataLayer.push(arguments);
}

function gaRemember(key, value) {
  try {
    if (value === null) window.localStorage.removeItem(key);
    else window.localStorage.setItem(key, value);
  } catch (e) {
    // Without storage the answer holds for this page only.
  }
}

function gaRecall(key) {
  try {
    return window.localStorage.getItem(key) || "";
  } catch (e) {
    return "";
  }
}

/**
 * The reader's answer, or "" when they have not been asked.
 * @returns {string} "granted", "denied" or ""
 */
function consentChoice() {
  return gaRecall(GA_CONSENT_KEY);
}

/**
 * Record the reader's answer and tell Analytics. Declining deletes the
 * cookies Analytics may already have set, so withdrawing consent withdraws
 * what it allowed.
 * @param {boolean} granted
 */
function setConsent(granted) {
  const value = granted ? "granted" : "denied";
  gaRemember(GA_CONSENT_KEY, value);
  gtag("consent", "update", { analytics_storage: value });
  if (!granted) gaForgetCookies();
}

function gaForgetCookies() {
  const host = location.hostname;
  const domains = ["", host, "." + host, "." + host.split(".").slice(-2).join(".")];
  for (const pair of document.cookie.split(";")) {
    const name = pair.split("=")[0].trim();
    if (name !== "_ga" && !name.startsWith("_ga_")) continue;
    for (const domain of domains) {
      document.cookie = `${name}=; Max-Age=0; path=/${domain ? "; domain=" + domain : ""}`;
    }
  }
}

/**
 * Send one event to Analytics (nothing off the live site).
 * @param {string} name
 * @param {Object} [details]
 */
function track(name, details) {
  if (location.hostname === GA_HOST) gtag("event", name, details || {});
}

// The rest runs once, in a scope of its own, so none of its names reach
// the pages' code.
(function () {
// ── Consent, before anything is measured ────────────────────────────────
const answered = consentChoice();
const deniedAds = { ad_storage: "denied", ad_user_data: "denied", ad_personalization: "denied" };
if (answered) {
  gtag("consent", "default", { ...deniedAds, analytics_storage: answered });
} else {
  // The more specific default wins: denied where consent comes first,
  // granted everywhere else.
  gtag("consent", "default", { ...deniedAds, analytics_storage: "denied", region: GA_ASK_FIRST, wait_for_update: 500 });
  gtag("consent", "default", { ...deniedAds, analytics_storage: "granted", wait_for_update: 500 });
}

const live = location.hostname === GA_HOST;
const params = new URLSearchParams(location.search);
if (params.has("ga_internal")) gaRemember(GA_INTERNAL_KEY, params.get("ga_internal") === "0" ? null : "1");
if (params.get("ga_debug") === "1") {
  try { window.sessionStorage.setItem("ga_debug", "1"); } catch (e) { /* this page only */ }
}
let debug = params.get("ga_debug") === "1";
try { debug = debug || window.sessionStorage.getItem("ga_debug") === "1"; } catch (e) { /* no storage */ }

if (live) {
  gtag("js", new Date());
  const settings = { send_page_view: false };
  if (gaRecall(GA_INTERNAL_KEY)) settings.traffic_type = "internal";
  if (debug) settings.debug_mode = true;
  gtag("config", GA_ID, settings);
}


// ── Page views, once the page is drawn and titled ───────────────────────
// Bio links carry `?ref=tiktok`, `?ref=yt`, `?ref=ig`, `?ref=linkedin`.
// Analytics reads a source from `utm_*` only, so the first page view names
// `ref` as the source, and each platform's visits show under Acquisition.
let firstView = true;
function pageView() {
  let address = location.href;
  if (firstView) {
    const ref = params.get("ref");
    if (ref && !params.has("utm_source")) {
      const url = new URL(location.href);
      url.searchParams.set("utm_source", ref);
      url.searchParams.set("utm_medium", "social");
      address = url.href;
    }
    firstView = false;
  }
  track("page_view", { page_location: address, page_title: document.title });
}
document.addEventListener("wf:render", pageView);

// ── What the reader does ────────────────────────────────────────────────
// A mailto: link is a lead; any element marked `data-track="event"` sends
// that event with its `data-track-*` attributes as parameters.
document.addEventListener("click", (event) => {
  const target = event.target instanceof Element ? event.target : null;
  if (!target) return;
  const marked = target.closest("[data-track]");
  if (marked) {
    const details = {};
    for (const attr of marked.attributes) {
      if (attr.name.startsWith("data-track-")) details[attr.name.slice(11).replace(/-/g, "_")] = attr.value;
    }
    track(marked.getAttribute("data-track"), details);
    return;
  }
  const link = target.closest('a[href^="mailto:"]');
  if (link) track("generate_lead", { method: "email", link_url: link.getAttribute("href") });
});

// What is typed into a search box, once the reader has paused on it.
let searchTimer = 0;
let lastSearch = "";
document.addEventListener("input", (event) => {
  const box = event.target;
  if (!(box instanceof HTMLInputElement) || box.type !== "search") return;
  clearTimeout(searchTimer);
  searchTimer = setTimeout(() => {
    const term = box.value.trim();
    if (term.length >= 2 && term !== lastSearch) {
      lastSearch = term;
      track("search", { search_term: term });
    }
  }, 1200);
});

// ── Core Web Vitals, from this visit ────────────────────────────────────
// What a reader's browser measured, sent once as the page is hidden: the
// largest paint, the layout that moved, how fast it answered input, and
// when it first painted and first heard from the server. Ratings use
// Google's thresholds.
const vitals = {};
const RATING = { LCP: [2500, 4000], CLS: [0.1, 0.25], INP: [200, 500], FCP: [1800, 3000], TTFB: [800, 1800] };

function observe(type, fn, options) {
  try {
    new PerformanceObserver((list) => list.getEntries().forEach(fn)).observe({ type, buffered: true, ...options });
  } catch (e) {
    // This browser does not measure it.
  }
}

observe("paint", (e) => { if (e.name === "first-contentful-paint") vitals.FCP = e.startTime; });
observe("largest-contentful-paint", (e) => { vitals.LCP = e.startTime; });
// A page that never moved scores 0, which is a result, not a gap.
if (PerformanceObserver.supportedEntryTypes?.includes("layout-shift")) vitals.CLS = 0;
let windowValue = 0, windowStart = 0, windowEnd = 0;
observe("layout-shift", (e) => {
  if (e.hadRecentInput) return;
  // The largest burst of shifts less than a second apart, in a 5 s window.
  if (e.startTime - windowEnd > 1000 || e.startTime - windowStart > 5000) {
    windowValue = 0;
    windowStart = e.startTime;
  }
  windowValue += e.value;
  windowEnd = e.startTime;
  vitals.CLS = Math.max(vitals.CLS || 0, windowValue);
});
observe("event", (e) => {
  if (e.interactionId) vitals.INP = Math.max(vitals.INP || 0, e.duration);
}, { durationThreshold: 40 });
const nav = performance.getEntriesByType("navigation")[0];
if (nav) vitals.TTFB = nav.responseStart;

let reported = false;
function reportVitals() {
  if (reported) return;
  reported = true;
  for (const [name, value] of Object.entries(vitals)) {
    const [good, poor] = RATING[name];
    track(name, {
      value: Math.round(name === "CLS" ? value * 1000 : value),
      metric_value: name === "CLS" ? Number(value.toFixed(4)) : Math.round(value),
      metric_rating: value <= good ? "good" : value <= poor ? "needs-improvement" : "poor",
      page_path: location.pathname,
      transport_type: "beacon",
    });
  }
}
document.addEventListener("visibilitychange", () => { if (document.visibilityState === "hidden") reportVitals(); });
window.addEventListener("pagehide", reportVitals);
})();
