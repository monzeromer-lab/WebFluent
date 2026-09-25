  // ─── Markdown ────────────────────────────────────────
  // The same small Markdown the compiler renders at build time (see
  // `src/codegen/markdown.rs`): headings, paragraphs, fenced code, quotes,
  // one-level lists, rules; code spans, strong, emphasis, links, images.
  // The text is escaped first, so HTML in it is shown, not run.
  function mdEscape(text) {
    return String(text).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/\x22/g, "&quot;");
  }
  // Written as `RegExp` over strings, so a tool that scans the bundle for
  // balanced brackets is not misled by a bracket inside a pattern.
  const MD = {
    heading: new RegExp("^(#{1,6}) (.*)$"),
    bullet: new RegExp("^[-*] (.*)$"),
    numbered: new RegExp("^[0-9]+\\. (.*)$"),
    code: new RegExp("\\x60([^\\x60]+)\\x60", "g"),
    image: new RegExp("!\\[([^\\]]*)\\]\\(([^)\\s]+)\\)", "g"),
    link: new RegExp("\\[([^\\]]+)\\]\\(([^)\\s]+)\\)", "g"),
    strong: new RegExp("\\*\\*([^*]+)\\*\\*", "g"),
    em: new RegExp("\\*([^*]+)\\*", "g"),
    em2: new RegExp("(^|[^A-Za-z0-9])_([^_]+)_($|[^A-Za-z0-9])", "g"),
  };
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
  function mdInline(text) {
    const spans = [];
    let s = mdEscape(text).replace(MD.code, (_, c) => { spans.push("<code>" + c + "</code>"); return "\u0000" + (spans.length - 1) + "\u0000"; });
    // A site-relative link or image is addressed from the base path, as
    // `Link(to:)` and `Image(src:)` are.
    const at = (url) => (url.startsWith("/") && !url.startsWith("//") ? _basePath + url : url);
    s = s.replace(MD.image, (_, alt, url) => '<img src="' + at(url) + '" alt="' + alt + '">');
    s = s.replace(MD.link, (_, label, url) => '<a href="' + at(url) + '">' + label + "</a>");
    s = s.replace(MD.strong, "<strong>$1</strong>");
    s = s.replace(MD.em, "<em>$1</em>");
    s = s.replace(MD.em2, "$1<em>$2</em>$3");
    spans.forEach((span, i) => { s = s.split("\u0000" + i + "\u0000").join(span); });
    return s.replace(/\n/g, "<br>\n");
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
    if (typeof v === "function") effect(() => { el.innerHTML = markdown(v()); });
    else el.innerHTML = markdown(v);
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

  function markdown(text) {
    const lines = String(text == null ? "" : text).split("\n");
    let out = "";
    let i = 0;
    const isBlock = (t) => t === "" || t.startsWith("```") || t === "---" || t === "***" || mdHeading(t) || t.startsWith(">") || mdListItem(t);
    while (i < lines.length) {
      const t = lines[i].trim();
      if (t === "") { i++; continue; }
      if (t.startsWith("```")) {
        const lang = t.slice(3).trim();
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
      if (h) { out += "<h" + h[0] + ">" + mdInline(h[1]) + "</h" + h[0] + ">\n"; i++; continue; }
      if (t.startsWith(">")) {
        const quoted = [];
        while (i < lines.length && lines[i].trim().startsWith(">")) { quoted.push(lines[i].trim().slice(1).replace(/^\s+/, "")); i++; }
        out += "<blockquote>\n" + markdown(quoted.join("\n")) + "</blockquote>\n";
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
          out += "<li>" + mdInline(it[0]) + "</li>\n";
          i++;
        }
        out += "</" + tag + ">\n";
        continue;
      }
      const para = [];
      while (i < lines.length && !isBlock(lines[i].trim())) { para.push(lines[i].trim()); i++; }
      out += "<p>" + mdInline(para.join("\n")) + "</p>\n";
    }
    return out;
  }
