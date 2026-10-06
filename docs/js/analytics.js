// Google Analytics 4 (gtag.js, G-FV6VK6N6VM) with Consent Mode v2: the
// set-up Google's snippet does in an inline <script>, which the site's
// policy refuses. The library itself is `meta.scripts` (async, so it never
// holds up the page); where it sends data is `meta.connect`; the banner
// that asks is `ConsentBanner`.
//
// Nothing is stored until the reader allows it: every consent type starts
// `denied`, so Analytics sets no cookies and sends only cookieless pings.
// Only the live site reports, so `wf serve` and test runs are not counted.

const CONSENT_KEY = "wf:analytics-consent";

window.dataLayer = window.dataLayer || [];
function gtag() {
  // gtag.js reads the `arguments` object itself, not an array.
  window.dataLayer.push(arguments);
}

/**
 * The reader's answer, or "" when they have not been asked.
 * @returns {string} "granted", "denied" or ""
 */
function consentChoice() {
  try {
    return window.localStorage.getItem(CONSENT_KEY) || "";
  } catch (e) {
    return "";
  }
}

/**
 * Record the reader's answer and tell Analytics. The site runs no ads, so
 * the advertising signals stay denied either way.
 * @param {boolean} granted
 */
function setConsent(granted) {
  const value = granted ? "granted" : "denied";
  try {
    window.localStorage.setItem(CONSENT_KEY, value);
  } catch (e) {
    // Without storage the answer holds for this page only.
  }
  gtag("consent", "update", { analytics_storage: value });
}

gtag("consent", "default", {
  analytics_storage: consentChoice() === "granted" ? "granted" : "denied",
  ad_storage: "denied",
  ad_user_data: "denied",
  ad_personalization: "denied",
  wait_for_update: 500,
});

if (location.hostname === "webfluent.monzeromer.dev") {
  gtag("js", new Date());
  // Bio links carry `?ref=tiktok`, `?ref=yt`, `?ref=ig`: Analytics reads a
  // campaign from `utm_*` only, so `ref` is passed on as the source, and
  // each platform's visits are counted under Acquisition.
  const ref = new URLSearchParams(location.search).get("ref");
  gtag("config", "G-FV6VK6N6VM", ref ? { campaign_source: ref, campaign_medium: "social" } : {});
}
