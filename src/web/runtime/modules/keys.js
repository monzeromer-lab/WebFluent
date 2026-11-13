  // `on key("ctrl+k") { }`: a keydown whose key and modifiers match the
  // spelling — `mod` (⌘ on Apple's systems, Ctrl elsewhere), `ctrl`,
  // `shift`, `alt`, `meta`/`cmd`, then the key name as `KeyboardEvent.key`
  // spells it (`k`, `Enter`, `Escape`, `ArrowDown`). A chord is its steps
  // with a space between, `mod+k mod+s`: the second within a second and a
  // half of the first.
  const KEY_NAMES = { esc: "escape", return: "enter", space: " ", up: "arrowup", down: "arrowdown", left: "arrowleft", right: "arrowright", plus: "+" };
  const CHORD_MS = 1500;
  function _apple() {
    if (typeof navigator === "undefined") return false;
    const p = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || navigator.userAgent || "";
    return /mac|iphone|ipad|ipod/i.test(p);
  }
  function _steps(spelling) {
    return String(spelling).trim().split(/\s+/);
  }
  // One step, `mod+shift+p`, as what it asks of an event.
  function _step(spelling) {
    const parts = String(spelling).toLowerCase().split("+").map((p) => p.trim());
    let key = parts.pop();
    // `mod++` and `ctrl++` name the plus key.
    if (key === "" && parts[parts.length - 1] === "") { parts.pop(); key = "+"; }
    const has = (m) => parts.includes(m);
    const mod = has("mod");
    return {
      key: KEY_NAMES[key] || key,
      ctrl: has("ctrl") || (mod && !_apple()),
      shift: has("shift"),
      alt: has("alt"),
      meta: has("meta") || has("cmd") || (mod && _apple()),
    };
  }
  // The US keyboard's punctuation keys, by the place each is at.
  const PUNCT_CODES = { ",": "comma", ".": "period", "/": "slash", ";": "semicolon", "'": "quote", "[": "bracketleft", "]": "bracketright", "\\": "backslash", "`": "backquote", "-": "minus", "=": "equal" };
  function _stepIs(e, step) {
    // A letter or digit is also its place on the keyboard, when the
    // layout gave no Latin letter or digit there: on a Mac ⌥⌘P sends `π`,
    // and an Arabic layout sends `ح`, with `KeyP` as the code. A layout
    // that does give one is read by it, so Ctrl+Q on AZERTY (code `KeyA`)
    // is `mod+q`, never `mod+a` too. A punctuation key likewise, when the
    // layout gave no ASCII there: the Arabic layout's comma key sends `و`,
    // so Ctrl+, is still `mod+,`; AZERTY's sends `;`, and is read by it.
    const pressed = (e.key || "").toLowerCase();
    if (pressed !== step.key) {
      const code = /^[a-z]$/.test(step.key) ? `key${step.key}` : /^[0-9]$/.test(step.key) ? `digit${step.key}` : PUNCT_CODES[step.key] || null;
      // With Shift, a punctuation key sends its other character — `/` is
      // `?` on a US layout — so `mod+shift+/` is that key held with Shift,
      // read by its place.
      const shifted = step.shift && PUNCT_CODES[step.key] && e.shiftKey;
      const latin = PUNCT_CODES[step.key] ? !shifted && /^[\x20-\x7e]$/.test(pressed) : /^[a-z0-9]$/.test(pressed);
      if (!code || latin || (e.code || "").toLowerCase() !== code) return false;
    }
    return !!e.ctrlKey === step.ctrl && !!e.shiftKey === step.shift && !!e.altKey === step.alt && !!e.metaKey === step.meta;
  }
  // The keys that went down, newest last, so a chord's later step can see
  // its earlier ones; a modifier alone is not a step.
  const _pressed = [];
  let _hearingKeys = false;
  function _hearKeys() {
    if (_hearingKeys || typeof document === "undefined" || !document.addEventListener) return;
    _hearingKeys = true;
    document.addEventListener("keydown", (e) => {
      if (/^(control|shift|alt|meta|os)$/i.test(e.key || "")) return;
      _pressed.push({ e, at: Date.now() });
      if (_pressed.length > 8) _pressed.shift();
    }, true);
  }
  _hearKeys();
  function keyIs(e, spelling) {
    const steps = _steps(spelling).map(_step);
    if (!_stepIs(e, steps[steps.length - 1])) return false;
    if (steps.length === 1) return true;
    // The earlier steps are the keys pressed just before this one.
    let at = -1;
    for (let i = _pressed.length - 1; i >= 0; i--) if (_pressed[i].e === e) { at = i; break; }
    if (at < steps.length - 1) return false;
    for (let k = 1; k < steps.length; k++) {
      const was = _pressed[at - k];
      const next = _pressed[at - k + 1];
      if (!_stepIs(was.e, steps[steps.length - 1 - k]) || next.at - was.at > CHORD_MS) return false;
    }
    return true;
  }
  // The first steps of the chords a page listens for: the key after one is
  // the chord's, not a shortcut of its own.
  const _chordStarts = [];
  function _afterChordStart(e) {
    let at = -1;
    for (let i = _pressed.length - 1; i >= 0; i--) if (_pressed[i].e === e) { at = i; break; }
    if (at < 1) return false;
    const was = _pressed[at - 1];
    if (_pressed[at].at - was.at > CHORD_MS) return false;
    return _chordStarts.some((step) => _stepIs(was.e, step));
  }
  // How a combination is written for the reader on this system: `⌘⇧P` on
  // a Mac, `Ctrl+Shift+P` elsewhere; a chord's steps with a space between.
  const KEY_SHOWN = { arrowup: "↑", arrowdown: "↓", arrowleft: "←", arrowright: "→", escape: "Esc", enter: "Enter", " ": "Space", backspace: "Backspace", delete: "Delete", tab: "Tab", pageup: "Page Up", pagedown: "Page Down", home: "Home", end: "End" };
  function _keyShown(key) {
    if (KEY_SHOWN[key]) return KEY_SHOWN[key];
    return key.length === 1 ? key.toUpperCase() : key.charAt(0).toUpperCase() + key.slice(1);
  }
  function keyLabel(spelling) {
    const apple = _apple();
    return _steps(spelling).map((text) => {
      const s = _step(text);
      const key = _keyShown(s.key);
      if (apple) return (s.ctrl ? "⌃" : "") + (s.alt ? "⌥" : "") + (s.shift ? "⇧" : "") + (s.meta ? "⌘" : "") + key;
      return [s.ctrl && "Ctrl", s.alt && "Alt", s.shift && "Shift", s.meta && "Meta", key].filter(Boolean).join("+");
    }).join(" ");
  }
  // The same, as `aria-keyshortcuts` writes it: `Meta+Shift+P`.
  function keyAria(spelling) {
    return _steps(spelling).map((text) => {
      const s = _step(text);
      const key = s.key === " " ? "Space" : _keyShown(s.key).replace(/^[↑↓←→]$/, (a) => ({ "↑": "ArrowUp", "↓": "ArrowDown", "←": "ArrowLeft", "→": "ArrowRight" })[a]).replace(/^Esc$/, "Escape");
      return [s.ctrl && "Control", s.alt && "Alt", s.shift && "Shift", s.meta && "Meta", key].filter(Boolean).join("+");
    }).join(" ");
  }
  // On the page, a key with no modifier — `n`, `/`, `?` — is the reader
  // typing when the focus is in a field, not a shortcut; one with `ctrl`,
  // `alt` or `meta`, and `Escape`, still is. On an element it is that
  // element's, and always answers.
  function _typing(e) {
    const t = e.target;
    if (!t || t.nodeType !== 1) return false;
    return t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName);
  }
  function onKey(target, spelling, fn) {
    _hearKeys();
    const steps = _steps(spelling);
    const first = steps[0];
    const plain = !/(^|\+)(mod|ctrl|control|alt|option|meta|cmd|command)\+/i.test(first)
      && !/^(escape|esc)$/i.test(first.split("+").pop());
    const page = target === document || target === window;
    if (steps.length > 1) {
      const start = _step(first);
      _chordStarts.push(start);
      onCleanup(() => {
        const i = _chordStarts.indexOf(start);
        if (i >= 0) _chordStarts.splice(i, 1);
      });
    }
    listen(target, "keydown", (e) => {
      if (page && plain && _typing(e)) return;
      if (steps.length === 1 && _afterChordStart(e)) return;
      if (keyIs(e, spelling)) fn(e);
    });
  }
