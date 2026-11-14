// The preview without a service worker. The page that framed this one
// holds a build inlined into one document (`previewInline` in wf.js); this
// asks for it by its slot, and writes it in place of itself, scripts and
// all. Only the framing page, on this origin, is heard.
const slot = new URLSearchParams(location.search).get("slot") ?? "";
addEventListener("message", (event) => {
  if (event.origin !== location.origin || event.source !== parent || event.data?.type !== "wf-run") return;
  document.open();
  document.write(event.data.html);
  document.close();
});
parent.postMessage({ type: "wf-run-ready", slot }, location.origin);
