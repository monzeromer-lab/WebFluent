  // ─── Markup a page did not write ─────────────────────
  //
  // Everything else in the language puts text in as text. `Unsafe.Html(…)`
  // is the one door for markup, and it is named so that a reviewer greps
  // for it and finds every one.
  //
  // `sanitize(html)` is what makes passing through that door reasonable: an
  // **allow-list** of elements and attributes, so a tag or an attribute
  // nobody thought of is dropped rather than permitted.
  //
  // It is a filter over the markup, not a use of the DOM parser: the twin
  // of `codegen::sanitize`, so the static paint and the live page put in
  // exactly the same thing, byte for byte. A test holds the two together.

  const SAFE_TAGS = new Set((
    "a abbr address b blockquote br caption cite code col colgroup dd del dfn div dl dt em " +
    "figcaption figure h1 h2 h3 h4 h5 h6 hr i img ins kbd li mark ol p pre q s samp section " +
    "small span strong sub summary sup table tbody td tfoot th thead time tr u ul var wbr"
  ).split(" "));
  const GLOBAL_ATTRS = ["class", "id", "title", "lang", "dir", "role"];
  const TAG_ATTRS = {
    a: ["href", "target", "rel"],
    img: ["src", "alt", "width", "height", "loading", "decoding"],
    td: ["colspan", "rowspan", "headers"],
    th: ["colspan", "rowspan", "scope", "headers"],
    ol: ["start", "reversed", "type"],
    li: ["value"],
    time: ["datetime"],
    blockquote: ["cite"],
    q: ["cite"],
    del: ["cite", "datetime"],
    ins: ["cite", "datetime"],
    col: ["span"],
    colgroup: ["span"],
  };
  const VOID_TAGS = ["br", "hr", "img", "wbr", "col"];

  /// `html` with everything the allow-list does not name taken out.
  function sanitize(html) {
    if (html == null) return "";
    const text = String(html);
    let out = "";
    let i = 0;
    while (i < text.length) {
      if (text[i] !== "<") { out += text[i]; i += 1; continue; }
      // A `<` opens a tag only when a name, a `/` or a `!` follows it
      // immediately — the browser's rule. `1 < 2` is arithmetic.
      const next = text[i + 1];
      const opens = next && (/[a-zA-Z]/.test(next) || next === "/" || next === "!" || next === "?");
      const end = opens ? tagEnd(text, i) : -1;
      if (end < 0) { out += "&lt;"; i += 1; continue; }
      const inside = text.slice(i + 1, end);
      i = end + 1;
      const closing = inside.startsWith("/");
      const body = inside.replace(/^\/+/, "");
      if (body.startsWith("!") || body.startsWith("?")) continue;
      const name = (body.match(/^[a-zA-Z0-9]*/) || [""])[0].toLowerCase();
      if (!name || !SAFE_TAGS.has(name)) {
        // The element goes; what it said stays, so dropping a `<font>`
        // does not drop the sentence inside it. A `<script>`'s body is not
        // a sentence, so it goes with the tag.
        if (!closing && (name === "script" || name === "style" || name === "template")) {
          i = skipToClose(text, i, name);
        }
        continue;
      }
      if (closing) { out += "</" + name + ">"; continue; }
      const allowed = GLOBAL_ATTRS.concat(TAG_ATTRS[name] || []);
      const kept = [];
      for (const [key, value] of attributesOf(body.slice(name.length))) {
        if (!allowed.includes(key)) continue;
        if (key === "href" || key === "src") {
          const guarded = safeUrl(value);
          if (!guarded) continue;
          kept.push([key, guarded]);
          continue;
        }
        kept.push([key, value]);
      }
      // A link that opens elsewhere does not hand it a handle on this page.
      if (name === "a" && kept.some(([k]) => k === "target") && !kept.some(([k]) => k === "rel")) {
        kept.push(["rel", "noopener noreferrer"]);
      }
      out += "<" + name;
      for (const [key, value] of kept) out += ` ${key}="${escapeAttr(value)}"`;
      out += VOID_TAGS.includes(name) ? " />" : ">";
    }
    return out;
  }

  /// Where the tag opened at `at` closes, honouring a quoted attribute
  /// value that holds a `>`.
  function tagEnd(text, at) {
    let quote = null;
    for (let i = at + 1; i < text.length; i++) {
      const c = text[i];
      if (quote) { if (c === quote) quote = null; continue; }
      if (c === '"' || c === "'") { quote = c; continue; }
      if (c === ">") return i;
      if (c === "<") return -1;
    }
    return -1;
  }

  /// Past the closing tag of `name`, or to the end where there is none.
  function skipToClose(text, from, name) {
    const at = text.toLowerCase().indexOf("</" + name, from);
    if (at < 0) return text.length;
    const end = tagEnd(text, at);
    return end < 0 ? text.length : end + 1;
  }

  /// `name="value"` pairs from the inside of a tag, after its name.
  function attributesOf(rest) {
    const out = [];
    let i = 0;
    while (i < rest.length) {
      while (i < rest.length && !/[a-zA-Z]/.test(rest[i])) i += 1;
      const from = i;
      while (i < rest.length && /[a-zA-Z0-9_-]/.test(rest[i])) i += 1;
      if (i === from) break;
      const key = rest.slice(from, i).toLowerCase();
      while (i < rest.length && /\s/.test(rest[i])) i += 1;
      if (rest[i] !== "=") { out.push([key, ""]); continue; }
      i += 1;
      while (i < rest.length && /\s/.test(rest[i])) i += 1;
      let value;
      if (rest[i] === '"' || rest[i] === "'") {
        const quote = rest[i];
        i += 1;
        const start = i;
        while (i < rest.length && rest[i] !== quote) i += 1;
        value = rest.slice(start, i);
        i += 1;
      } else {
        const start = i;
        while (i < rest.length && !/\s/.test(rest[i])) i += 1;
        value = rest.slice(start, i);
      }
      out.push([key, unescapeEntities(value)]);
    }
    return out;
  }

  /// The entities a value may arrive with, so `&#106;avascript:` is not a
  /// way past the scheme check.
  function unescapeEntities(value) {
    let out = "";
    let i = 0;
    while (i < value.length) {
      if (value[i] !== "&") { out += value[i]; i += 1; continue; }
      const rest = value.slice(i, i + 12);
      const end = rest.indexOf(";");
      if (end < 0) { out += "&"; i += 1; continue; }
      const entity = rest.slice(1, end);
      const named = { amp: "&", lt: "<", gt: ">", quot: '"', apos: "'", "#39": "'", colon: ":", "#58": ":" };
      let decoded = named[entity];
      if (decoded === undefined && /^#[xX][0-9a-fA-F]+$/.test(entity)) {
        decoded = String.fromCodePoint(parseInt(entity.slice(2), 16));
      } else if (decoded === undefined && /^#\d+$/.test(entity)) {
        decoded = String.fromCodePoint(parseInt(entity.slice(1), 10));
      }
      if (decoded === undefined) { out += "&"; i += 1; continue; }
      out += decoded;
      i += end + 1;
    }
    return out;
  }

  function escapeAttr(value) {
    return String(value)
      .replace(/&/g, "&amp;")
      .replace(/"/g, "&quot;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  /// `Unsafe.Html(markup)`: markup, as markup. It is not sanitised — that
  /// is what the name says — and `sanitize(…)` is one call away.
  attrHook("html", (el, markup) => {
    if (typeof markup === "function") effect(() => { el.innerHTML = String(markup() ?? ""); });
    else el.innerHTML = String(markup ?? "");
  });
