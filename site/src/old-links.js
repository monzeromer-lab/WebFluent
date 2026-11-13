// Links an older `wf` printed, to places that have moved.
//
// Up to 6.0, every finding linked to its section of one diagnostics page:
// `/docs/guide/diagnostics#t05-a-field-or-method-that-does-not-exist`. Each
// code now has a page of its own, `/errors/T05`. The part after `#` never
// reaches the server, so the page that answers the old address is the one
// place that can read it — before the router draws anything.
(function () {
  var path = location.pathname.replace(/(\.html|\/)$/, "");
  // A chapter that became recipes: its `#section` is a recipe now
  // (old-links-map.js, written with the recipes).
  if (typeof OLD_LINKS === "object" && location.hash) {
    var page = path.replace(/^.*?(\/docs\/)/, "/docs/");
    var to = OLD_LINKS[page + location.hash];
    if (to) {
      location.replace(path.slice(0, path.length - page.length) + to);
      return;
    }
  }
  // A page of the guide that moved (old-links-pages.js, written with the
  // guide): the same section of the page where it is now.
  if (typeof OLD_PAGES === "object" && path !== "") {
    var at = path.replace(/^.*?(\/docs\/)/, "/docs/");
    var moved = OLD_PAGES[at];
    if (moved) {
      location.replace(path.slice(0, path.length - at.length) + moved + location.hash);
      return;
    }
  }
  if (!/\/docs\/guide\/diagnostics$/.test(path)) return;
  var base = path.slice(0, path.length - "/docs/guide/diagnostics".length);
  var code = /^#([a-z]\d{2,3})(-|$)/i.exec(location.hash);
  var to = code ? "/errors/" + code[1].toUpperCase() : "/errors" + location.hash;
  location.replace(base + to);
})();
