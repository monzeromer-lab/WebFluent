  // ─── Markdown ────────────────────────────────────────
  // The same small Markdown the compiler renders at build time (see
  // `src/codegen/markdown.rs`): headings, paragraphs, fenced code, quotes,
  // one-level lists, rules; code spans, strong, emphasis, links, images.
  // The text is escaped first, so HTML in it is shown, not run, and a link
  // or image whose URL a browser would run is drawn as its words. A
  // backslash before ASCII punctuation draws that character and nothing
  // more (`\*`, `\[`, `\\`), as CommonMark's does.
  //
  // `{ inert: true }` (`Markdown(text).inert`) is for text somebody else
  // wrote: its entities are decoded before they are escaped, a link is
  // drawn as its words and an image is left out.
  function mdEscape(text) {
    return String(text).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/\x22/g, "&quot;");
  }
  // Written as `RegExp` over strings, so a tool that scans the bundle for
  // balanced brackets is not misled by a bracket inside a pattern.
  const MD = {
    heading: new RegExp("^(#{1,6}) (.*)$"),
    bullet: new RegExp("^[-*] (.*)$"),
    numbered: new RegExp("^[0-9]+\\. (.*)$"),
    // An ASCII punctuation character, which a backslash escapes.
    punct: new RegExp("^[!-\\/:-@\\[-\\x60{-~]$"),
    escaped: new RegExp("\\\\([!-\\/:-@\\[-\\x60{-~])", "g"),
    held: new RegExp("\\u0000(0|[1-9][0-9]*)\\u0000", "g"),
    space: new RegExp("\\s"),
    image: new RegExp("!\\[([^\\[\\]]*)\\]\\(([^)\\s]+)\\)", "g"),
    link: new RegExp("\\[([^\\[\\]]+)\\]\\(([^)\\s]+)\\)", "g"),
    // Inert's: an image with the space after it, and a link whose words
    // may be empty — what is left of a linked image.
    picture: new RegExp("!\\[([^\\[\\]]*)\\]\\(([^)\\s]+)\\) ?", "g"),
    words: new RegExp("\\[([^\\[\\]]*)\\]\\(([^)\\s]+)\\)", "g"),
    strong: new RegExp("\\*\\*([^*]+)\\*\\*", "g"),
    em: new RegExp("\\*([^*]+)\\*", "g"),
    em2: new RegExp("(^|[^A-Za-z0-9])_([^_]+)_($|[^A-Za-z0-9])", "g"),
    entity: new RegExp("&amp;(#[0-9]+|#[xX][0-9a-fA-F]+|[A-Za-z][A-Za-z0-9]{1,31});", "g"),
    edges: new RegExp("^[ \\t\\n]+|[ \\t\\n]+$", "g"),
    markers: new RegExp("[\\u0000\\u0001]", "g"),
  };
  // The named entities inert text decodes: the compiler's `ENTITIES`.
  const MD_NAMED = {
    amp: "&", lt: "<", gt: ">", quot: "\x22", apos: "'", nbsp: "\u00a0", ensp: "\u2002", emsp: "\u2003",
    thinsp: "\u2009", shy: "\u00ad", zwj: "\u200d", zwnj: "\u200c", copy: "\u00a9", reg: "\u00ae",
    trade: "\u2122", mdash: "\u2014", ndash: "\u2013", minus: "\u2212", hellip: "\u2026", lsquo: "\u2018",
    rsquo: "\u2019", ldquo: "\u201c", rdquo: "\u201d", laquo: "\u00ab", raquo: "\u00bb", bull: "\u2022",
    middot: "\u00b7", times: "\u00d7", divide: "\u00f7", plusmn: "\u00b1", deg: "\u00b0", le: "\u2264",
    ge: "\u2265", ne: "\u2260", larr: "\u2190", rarr: "\u2192", uarr: "\u2191", darr: "\u2193",
    harr: "\u2194", lArr: "\u21d0", rArr: "\u21d2", hArr: "\u21d4", euro: "\u20ac", pound: "\u00a3",
    yen: "\u00a5", cent: "\u00a2", sect: "\u00a7", para: "\u00b6",
  };
  /// The character an entity's body (`lt`, `#60`, `#x3C`) stands for, or
  /// null for a name the table does not have. A number may have any number
  /// of digits (`&#00060;` is `<`); one that is no character a page should
  /// hold is U+FFFD.
  function mdEntity(body) {
    if (body[0] !== "#") return Object.prototype.hasOwnProperty.call(MD_NAMED, body) ? MD_NAMED[body] : null;
    const code = body[1] === "x" || body[1] === "X" ? parseInt(body.slice(2), 16) : parseInt(body.slice(1), 10);
    const refused = code === 0 || code > 0x10ffff || (code >= 0xd800 && code <= 0xdfff)
      || (code < 0x20 && code !== 0x9 && code !== 0xa && code !== 0xd) || (code >= 0x7f && code <= 0x9f);
    return refused ? "\ufffd" : String.fromCodePoint(code);
  }
  function mdHeading(line) {
    const m = MD.heading.exec(line);
    return m ? [m[1].length, m[2].trim()] : null;
  }
  function mdListItem(line) {
    let m = MD.bullet.exec(line);
    if (m) return [m[1].trim(), false];
    m = MD.numbered.exec(line);
    if (m) return [m[1].trim(), true];
    return null;
  }
  /// `text` escaped, with its code spans and backslash escapes held aside
  /// as `\u0000N\u0000`, `held[N]` being `[html, isEscapedChar]`: one pass
  /// left to right, as CommonMark reads them, so an escaped backtick opens
  /// no span and a backslash in a span is a backslash. Before ASCII
  /// punctuation a backslash is that character and nothing more; before a
  /// newline (or the `\u0001` of a line ending in two spaces) it breaks
  /// the line, and the spaces before it go; before anything else it stays.
  /// The twin of the compiler's `hold`.
  function mdHold(text) {
    const held = [];
    let out = "";
    let plain = 0;
    let i = 0;
    const mark = (html, isChar) => { held.push([html, isChar]); return "\u0000" + (held.length - 1) + "\u0000"; };
    while (i < text.length) {
      const c = text[i];
      const next = text[i + 1];
      if (c === "\\" && next !== undefined && MD.punct.test(next)) {
        out += mdEscape(text.slice(plain, i)) + mark(mdEscape(next), true);
        i += 2;
        plain = i;
        continue;
      }
      if (c === "\\" && (next === "\n" || next === "\u0001")) {
        out += mdEscape(text.slice(plain, i));
        let end = out.length;
        while (end > 0 && out[end - 1] !== "\n" && MD.space.test(out[end - 1])) end--;
        out = out.slice(0, end);
        if (next === "\n") out += "\u0001";
        i += 1;
        plain = i;
        continue;
      }
      if (c === "\x60") {
        const close = text.indexOf("\x60", i + 1);
        if (close > i + 1) {
          out += mdEscape(text.slice(plain, i)) + mark("<code>" + mdEscape(text.slice(i + 1, close)) + "</code>", false);
          i = close + 1;
          plain = i;
          continue;
        }
      }
      i++;
    }
    return [out + mdEscape(text.slice(plain)), held];
  }
  /// `text` with what `mdHold` set aside put back: only the escaped
  /// characters when `charsOnly`, everything otherwise.
  function mdRestore(text, held, charsOnly) {
    if (!held.length) return text;
    return text.replace(MD.held, (whole, n) => {
      const h = held[Number(n)];
      return h && (h[1] || !charsOnly) ? h[0] : whole;
    });
  }
  function mdInline(text, inert) {
    const [heldText, held] = mdHold(text);
    let s = heldText;
    if (inert) {
      s = s.replace(MD.picture, "").replace(MD.words, "$1");
    } else {
      // A site-relative link or image is addressed from the base path, as
      // `Link(to:)` and `Image(src:)` are. The URL is checked as it was
      // escaped, which names the scheme it would unescaped: escaping adds
      // only `&` and `;`, neither of which a scheme may hold. It is checked
      // with its escaped characters as the characters they are, and
      // written with them still held, so none of them is emphasis.
      const at = (url) => (url.startsWith("/") && !url.startsWith("//") ? _basePath : "");
      s = s.replace(MD.image, (_, alt, url) => { const u = mdRestore(url, held, true); return safeUrl(u) ? '<img src="' + at(u) + url + '" alt="' + alt + '">' : alt; });
      s = s.replace(MD.link, (_, label, url) => { const u = mdRestore(url, held, true); return safeUrl(u) ? '<a href="' + at(u) + url + '">' + label + "</a>" : label; });
    }
    s = s.replace(MD.strong, "<strong>$1</strong>");
    s = s.replace(MD.em, "<em>$1</em>");
    s = s.replace(MD.em2, "$1<em>$2</em>$3");
    // Decoded where it was escaped, once the spans are found, so what an
    // entity says is a character and never Markdown; code and escaped
    // characters, still held aside, keep theirs as written.
    if (inert) s = s.replace(MD.entity, (whole, body) => { const c = mdEntity(body); return c === null ? whole : mdEscape(c); });
    return mdRestore(s, held, false).replace(/\u0001\n/g, "<br>\n").replace(/\u0001/g, "");
  }
  // ─── Syntax colouring ───────────────────────────────
  //
  // `Code(…, language: "wf")` and a Markdown fence: the code as HTML with
  // a `<span class="wf-tok-…">` around each token kind — the same tokens
  // the compiler's `codegen::highlight` writes into the static paint.
  const HL_KEYWORDS = new Set(("page component store theme app type enum const data animation test state persist " +
    "derived effect action use resource event slot part if else for in by show match let return await try catch emit " +
    "navigate log on style transition children every after from to cleanup expect not true false null loading error ready").split(" "));
  const hlSpan = (kind, text) => '<span class="wf-tok-' + kind + '">' + mdEscape(text) + "</span>";
  const hlWord = (c) => /[\p{L}\p{N}_-]/u.test(c);
  /// The index just past the closing `"#…` of a raw string opening at `at`,
  /// or null when none opens there. The twin of `codegen::highlight`.
  function hlRawEnd(chars, at) {
    let hashes = 0;
    let i = at;
    while (chars[i] === "#") { hashes++; i++; }
    if (chars[i] !== '"') return null;
    i++;
    while (i < chars.length) {
      if (chars[i] === '"') {
        let all = true;
        for (let n = 1; n <= hashes; n++) if (chars[i + n] !== "#") { all = false; break; }
        if (all) return i + hashes + 1;
      }
      i++;
    }
    return chars.length;
  }

  function hlCodeLike(code, keywords) {
    const chars = Array.from(code);
    let out = "";
    let plain = "";
    const flush = () => { if (plain) { out += mdEscape(plain); plain = ""; } };
    let i = 0;
    while (i < chars.length) {
      const c = chars[i];
      if (c === "/" && chars[i + 1] === "/") {
        flush();
        const start = i;
        while (i < chars.length && chars[i] !== "\n") i++;
        out += hlSpan("cmt", chars.slice(start, i).join(""));
        continue;
      }
      // `#"…"#`: a raw string, however many hashes it was written with.
      if (keywords && c === "#") {
        const end = hlRawEnd(chars, i);
        if (end !== null) {
          flush();
          out += hlSpan("str", chars.slice(i, end).join(""));
          i = end;
          continue;
        }
      }
      if (c === '"') {
        flush();
        const start = i;
        // `"""…"""`: a block string, over as many lines as it likes.
        if (keywords && chars[i + 1] === '"' && chars[i + 2] === '"') {
          i += 3;
          while (i + 2 < chars.length && !(chars[i] === '"' && chars[i + 1] === '"' && chars[i + 2] === '"')) i++;
          i = Math.min(i + 3, chars.length);
          out += hlSpan("str", chars.slice(start, i).join(""));
          continue;
        }
        i++;
        while (i < chars.length && chars[i] !== '"') { if (chars[i] === "\\") i++; i++; }
        i = Math.min(i + 1, chars.length);
        out += hlSpan("str", chars.slice(start, i).join(""));
        continue;
      }
      if (c === "$" && chars[i + 1] !== undefined && hlWord(chars[i + 1])) {
        flush();
        const start = i;
        i++;
        while (i < chars.length && hlWord(chars[i])) i++;
        out += hlSpan("tok", chars.slice(start, i).join(""));
        continue;
      }
      if (c === "#" && chars[i + 1] !== undefined && /[0-9a-fA-F]/.test(chars[i + 1])) {
        flush();
        const start = i;
        i++;
        while (i < chars.length && /[0-9a-fA-F]/.test(chars[i])) i++;
        out += hlSpan("num", chars.slice(start, i).join(""));
        continue;
      }
      const prev = i > 0 ? chars[i - 1] : "";
      if (/[0-9]/.test(c) && !(i > 0 && (/[\p{L}\p{N}]/u.test(prev) || prev === "_"))) {
        flush();
        const start = i;
        while (i < chars.length && /[0-9.]/.test(chars[i])) i++;
        while (i < chars.length && /[A-Za-z%]/.test(chars[i])) i++;
        out += hlSpan("num", chars.slice(start, i).join(""));
        continue;
      }
      if (/\p{L}/u.test(c) || c === "_") {
        const start = i;
        while (i < chars.length && (/[\p{L}\p{N}]/u.test(chars[i]) || chars[i] === "_" || (!keywords && chars[i] === "-"))) i++;
        const word = chars.slice(start, i).join("");
        let j = i;
        while (j < chars.length && chars[j] === " ") j++;
        const afterDot = start > 0 && chars[start - 1] === ".";
        let kind = null;
        if (chars[j] === ":" && chars[j + 1] !== ":" && !afterDot) kind = "prop";
        else if (keywords && !afterDot && HL_KEYWORDS.has(word)) kind = "kw";
        else if (keywords && /\p{Lu}/u.test(word[0])) kind = "name";
        if (kind) { flush(); out += hlSpan(kind, word); } else plain += word;
        continue;
      }
      plain += c;
      i++;
    }
    flush();
    return out;
  }
  function hlJson(code) {
    const chars = Array.from(code);
    let out = "";
    let plain = "";
    const flush = () => { if (plain) { out += mdEscape(plain); plain = ""; } };
    let i = 0;
    while (i < chars.length) {
      const c = chars[i];
      if (c === '"') {
        flush();
        const start = i;
        i++;
        while (i < chars.length && chars[i] !== '"') { if (chars[i] === "\\") i++; i++; }
        i = Math.min(i + 1, chars.length);
        let j = i;
        while (j < chars.length && chars[j] === " ") j++;
        out += hlSpan(chars[j] === ":" ? "prop" : "str", chars.slice(start, i).join(""));
        continue;
      }
      if (/[0-9]/.test(c) || (c === "-" && /[0-9]/.test(chars[i + 1] || ""))) {
        flush();
        const start = i;
        i++;
        while (i < chars.length && /[0-9.eE+-]/.test(chars[i])) i++;
        out += hlSpan("num", chars.slice(start, i).join(""));
        continue;
      }
      if (/[A-Za-z]/.test(c)) {
        const start = i;
        while (i < chars.length && /[A-Za-z]/.test(chars[i])) i++;
        const word = chars.slice(start, i).join("");
        if (word === "true" || word === "false" || word === "null") { flush(); out += hlSpan("kw", word); }
        else plain += word;
        continue;
      }
      plain += c;
      i++;
    }
    flush();
    return out;
  }
  function hlShell(code) {
    return code.split("\n").map((line) => {
      if (line.startsWith("$ ")) return hlSpan("prompt", "$ ") + mdEscape(line.slice(2));
      if (line.trimStart().startsWith("#")) return hlSpan("cmt", line);
      return mdEscape(line);
    }).join("\n");
  }
  function highlightKnows(lang) {
    return ["wf", "wfx", "json", "bash", "sh", "shell", "css"].includes(lang);
  }
  function highlight(code, lang) {
    code = String(code == null ? "" : code);
    switch (lang) {
      case "wf": case "wfx": return hlCodeLike(code, true);
      case "css": return hlCodeLike(code, false);
      case "json": return hlJson(code);
      case "bash": case "sh": case "shell": return hlShell(code);
      default: return mdEscape(code);
    }
  }

  attrHook("markdown", (el, v) => {
    // `{ text, inert }`: `Markdown(text).inert`; either may follow state.
    const opts = v !== null && typeof v === "object" && "inert" in v ? v : null;
    const text = opts ? opts.text : v;
    const read = (x) => (typeof x === "function" ? x() : x);
    const paint = () => { el.innerHTML = markdown(read(text), opts && read(opts.inert) ? { inert: true } : undefined); };
    if (typeof text === "function" || (opts && typeof opts.inert === "function")) effect(paint);
    else paint();
  });
  attrHook("highlight", (el, v) => {
    // `{ code, lang }`: the code coloured as the language.
    const paint = () => {
      const code = typeof v.code === "function" ? v.code() : v.code;
      const lang = typeof v.lang === "function" ? v.lang() : v.lang;
      el.innerHTML = highlight(code, lang);
    };
    if (typeof v.code === "function" || typeof v.lang === "function") effect(paint);
    else paint();
  });

  function markdown(text, opts) {
    const inert = !!(opts && opts.inert);
    let source = String(text == null ? "" : text);
    // The two characters the renderer marks its own places with cannot
    // arrive in text somebody else wrote.
    if (inert) source = source.replace(MD.markers, "\ufffd");
    // Lines as the compiler's `str::lines` has them: split at `\n` or
    // `\r\n`, with no empty line after the last newline. A fence left
    // open at the end, or a text with Windows line endings, is painted
    // the same at build time and here.
    const lines = source.split(/\r?\n/);
    if (lines[lines.length - 1] === "") lines.pop();
    let out = "";
    let i = 0;
    const isBlock = (t) => t === "" || t.startsWith("```") || t === "---" || t === "***" || mdHeading(t) || t.startsWith(">") || mdListItem(t);
    while (i < lines.length) {
      const t = lines[i].trim();
      if (t === "") { i++; continue; }
      if (t.startsWith("```")) {
        // The language takes backslash escapes, as CommonMark's info
        // string does; the code does not.
        const lang = t.slice(3).trim().replace(MD.escaped, "$1");
        let raw = "";
        i++;
        while (i < lines.length && lines[i].trim() !== "```") { raw += lines[i] + "\n"; i++; }
        i++;
        const code = highlightKnows(lang) ? highlight(raw, lang) : mdEscape(raw);
        out += lang ? '<pre><code class="language-' + mdEscape(lang) + '">' + code + "</code></pre>\n" : "<pre><code>" + code + "</code></pre>\n";
        continue;
      }
      if (t === "---" || t === "***") { out += "<hr>\n"; i++; continue; }
      const h = mdHeading(t);
      if (h) { out += "<h" + h[0] + ">" + mdInline(h[1], inert) + "</h" + h[0] + ">\n"; i++; continue; }
      if (t.startsWith(">")) {
        const quoted = [];
        while (i < lines.length && lines[i].trim().startsWith(">")) { quoted.push(lines[i].trim().slice(1).replace(/^\s+/, "")); i++; }
        out += "<blockquote>\n" + markdown(quoted.join("\n"), opts) + "</blockquote>\n";
        continue;
      }
      const item = mdListItem(t);
      if (item) {
        const ordered = item[1];
        const tag = ordered ? "ol" : "ul";
        out += "<" + tag + ">\n";
        while (i < lines.length) {
          const it = mdListItem(lines[i].trim());
          if (!it || it[1] !== ordered) break;
          out += "<li>" + mdInline(it[0], inert) + "</li>\n";
          i++;
        }
        out += "</" + tag + ">\n";
        continue;
      }
      // Lines run together, as CommonMark's do; one that ends in two spaces
      // breaks there (`\u0001` marks it), and so does one that ends in a
      // backslash, which `mdHold` sees before the newline.
      const para = [];
      while (i < lines.length && !isBlock(lines[i].trim())) {
        const t = lines[i].trim();
        para.push(lines[i].endsWith("  ") ? t + "\u0001" : t);
        i++;
      }
      if (para.length) para[para.length - 1] = para[para.length - 1].replace(/\u0001+$/, "");
      const body = mdInline(para.join("\n"), inert);
      // A paragraph of nothing but images has nothing left to draw once
      // they are left out.
      if (inert && body.replace(MD.edges, "") === "") continue;
      out += "<p>" + body + "</p>\n";
    }
    return out;
  }
