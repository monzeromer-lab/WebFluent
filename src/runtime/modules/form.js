  // ─── What a value must be ────────────────────────────
  //
  // A `validate` block beside a state compiles to one call of `validate`:
  // the value, the rules, and the form it belongs to. What comes back is a
  // signal holding the message to show — empty when the value is fine —
  // which the control then shows through the field plumbing that already
  // exists: `role="alert"`, `aria-invalid`, `aria-describedby`.
  //
  // When to show it is the form's: after the reader leaves the field
  // (`.onBlur`, the default), only after a submit (`.onSubmit`), or as
  // they type (`.live`). A message that is not shown is still counted, so
  // `form.valid` is the truth whatever the reader has seen.

  /// The message a rule shows when the author writes none. The project's
  /// translations replace it: a key of `form.required` wins over this.
  const RULE_MESSAGES = {
    required: "This is required",
    email: "Enter an email address",
    url: "Enter a web address",
    minLength: "Too short",
    maxLength: "Too long",
    min: "Too small",
    max: "Too large",
    pattern: "That is not the right shape",
    matches: "The two do not match",
    oneOf: "Choose one of the options",
    custom: "That is not right",
    async: "That is not right",
  };

  const EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

  /// Whether a value is there at all: an empty string, an empty list, null
  /// and `false` on a checkbox are all "not given".
  function given(value) {
    if (value == null) return false;
    if (typeof value === "string") return value.trim() !== "";
    if (Array.isArray(value)) return value.length > 0;
    if (typeof value === "boolean") return value;
    return true;
  }

  /// Whether one rule holds. `null` means "cannot say yet", which is what
  /// an async rule is until its answer arrives.
  function holds(rule, value) {
    const arg = rule.args && rule.args.length ? rule.args[0] : undefined;
    switch (rule.name) {
      case "required": return given(value);
      // Every other rule passes an empty value: `required` is what says a
      // value must be there, so a blank optional field is not two errors.
      case "email": return !given(value) || EMAIL.test(String(value));
      case "url": return !given(value) || /^[a-z][a-z0-9+.-]*:\/\/[^\s]+$/i.test(String(value));
      case "minLength": return !given(value) || String(value).length >= Number(arg);
      case "maxLength": return !given(value) || String(value).length <= Number(arg);
      case "min": return !given(value) || compare(value, arg) >= 0;
      case "max": return !given(value) || compare(value, arg) <= 0;
      case "pattern": return !given(value) || new RegExp(arg.source || arg, arg.flags || "").test(String(value));
      case "matches": return value === (typeof arg === "function" ? arg() : arg);
      case "oneOf": return !given(value) || (Array.isArray(arg) ? arg : []).includes(value);
      case "custom": return !!rule.check;
      default: return true;
    }
  }

  /// Two values in order: numbers as numbers, dates and times as text,
  /// which is what their carriers compare as.
  function compare(a, b) {
    const x = typeof a === "number" ? a : Number(a);
    const y = typeof b === "number" ? b : Number(b);
    if (!Number.isNaN(x) && !Number.isNaN(y)) return x === y ? 0 : x < y ? -1 : 1;
    return String(a) === String(b) ? 0 : String(a) < String(b) ? -1 : 1;
  }

  /// The message a rule shows, translated when the project has a table.
  function ruleMessage(rule) {
    if (rule.message != null) {
      return typeof rule.message === "function" ? rule.message() : rule.message;
    }
    const key = "form." + rule.name;
    if (i18nInstance) {
      const said = i18nInstance.t(key);
      if (said !== key) return said;
    }
    return RULE_MESSAGES[rule.name] || "That is not right";
  }

  /// The form a rule belongs to, when it belongs to one.
  function formOf(handle) {
    if (handle && handle.__wfForm) return handle;
    return null;
  }

  /// One state's rules. Gives back a signal of the message to show.
  function validate(read, rules, options) {
    const opts = options || {};
    const form = formOf(opts.form);
    const shown = signal("");
    const touched = signal(false);
    const pending = signal(false);
    // What the rules say, whether or not it is being shown.
    const fault = computed(() => {
      const value = read();
      for (const rule of rules) {
        if (rule.name === "async") continue;
        if (rule.name === "custom") {
          if (!rule.check()) return ruleMessage(rule);
          continue;
        }
        if (!holds(rule, value)) return ruleMessage(rule);
      }
      return "";
    });
    // An async rule answers after the rest have passed, so a name is not
    // asked of the server on every keystroke of an invalid address.
    const remote = signal("");
    const asyncRules = rules.filter((r) => r.name === "async");
    if (asyncRules.length) {
      let generation = 0;
      let asked;
      effect(() => {
        const value = read();
        if (fault()) { remote.set(""); asked = undefined; return; }
        // One question per value: the rules and the value both wake this,
        // and the server should hear about each value once.
        if (value === asked) return;
        asked = value;
        const mine = ++generation;
        pending.set(true);
        Promise.all(asyncRules.map((r) => Promise.resolve(r.check(value)).then((ok) => (ok ? "" : ruleMessage(r)))))
          .then((said) => {
            if (mine !== generation) return;
            remote.set(said.find(Boolean) || "");
            pending.set(false);
          })
          .catch(() => { if (mine === generation) pending.set(false); });
      });
    }

    // What the server said about this field, which stands until the reader
    // changes the value it was said about.
    const server = signal("");
    let said = read();
    effect(() => {
      const now = read();
      if (now !== said) {
        said = now;
        server.set("");
      }
    });
    const message = computed(() => fault() || remote() || server());
    const when = (form && form.show()) || opts.show || "onBlur";
    effect(() => {
      const said = message();
      const ready = when === "live" || (when === "onBlur" && touched()) || (form && form.submitted());
      shown.set(ready ? said : "");
    });

    const entry = { message, shown, touched, pending, server, read };
    if (form) form.__register(opts.name || "", entry);
    return entry;
  }

  /// The handle a `Form(bind: f)` gives: what every field says, together.
  ///
  /// It is the element's handle as well — `ref: f` sets `current` — so the
  /// browser's own checks answer for the controls no `validate` block
  /// guards, and the rules answer for the ones it does.
  function form(options) {
    const opts = options || {};
    const fields = new Map();
    const submitted = signal(false);
    const show = signal(opts.show || "onBlur");
    const node = signal(null);
    const native = signal(true);
    // A field registers after the form is made, so what reads the fields
    // reads this too, and is worked out again when one arrives.
    const registered = signal(0);

    const readNative = () => {
      const element = node();
      if (!element) return;
      native.set(typeof element.checkValidity === "function" ? element.checkValidity() : true);
    };

    const anyOf = (pick) =>
      computed(() => {
        registered();
        for (const entry of fields.values()) {
          const value = pick(entry);
          if (value) return true;
        }
        return false;
      });

    // Each of these is a signal the handle reads through a getter, so
    // `form.valid` is a value where it is written and still a read the
    // effect around it follows.
    const isValid = computed(() => {
      registered();
      for (const entry of fields.values()) {
        if (entry.message()) return false;
      }
      return native();
    });
    const isPending = anyOf((e) => e.pending());
    const messages = computed(() => {
      registered();
      const out = {};
      for (const [name, entry] of fields) {
        const said = entry.message();
        if (said) out[name] = said;
      }
      return out;
    });
    const left = computed(() => {
      registered();
      const out = {};
      for (const [name, entry] of fields) out[name] = entry.touched();
      return out;
    });

    const handle = {
      __wfForm: true,
      show,
      submitted,
      /// Whether every field is happy, whether or not it has been shown:
      /// the rules, and the browser's own checks for everything else.
      get valid() { return isValid(); },
      /// Whether an async rule is still asking.
      get pending() { return isPending(); },
      /// The message of every field that has one, by name.
      get errors() { return messages(); },
      /// The fields the reader has left.
      get touched() { return left(); },
      /// What the controls hold, by their `name`.
      get values() {
        const element = node();
        if (!element) return {};
        const out = {};
        const controls = element.elements
          ? Array.from(element.elements)
          : Array.from(element.querySelectorAll("input, select, textarea"));
        for (const control of controls) {
          const name = control.name || (control.getAttribute && control.getAttribute("name"));
          if (!name) continue;
          const kind = control.type || (control.getAttribute && control.getAttribute("type"));
          if (kind === "checkbox") out[name] = !!control.checked;
          else if (kind === "radio") { if (control.checked) out[name] = control.value; }
          else out[name] = control.value;
        }
        return out;
      },
      /// The element itself, for anything the language does not say.
      get current() { return node(); },
      set current(element) {
        node.set(element);
        if (!element || !element.addEventListener) return;
        element.addEventListener("input", readNative);
        element.addEventListener("change", readNative);
        element.addEventListener("reset", () => setTimeout(readNative, 0));
        readNative();
      },
      get element() { return node(); },
      /// What the server said, shown on the fields it named.
      apply(said) {
        for (const [name, message] of Object.entries(said || {})) {
          const entry = fields.get(name);
          if (entry) {
            entry.touched.set(true);
            entry.server.set(Array.isArray(message) ? message[0] : String(message));
          }
        }
        submitted.set(true);
        focusFirstInvalid(node(), fields);
      },
      reset() {
        submitted.set(false);
        readNative();
        for (const entry of fields.values()) {
          entry.touched.set(false);
          entry.shown.set("");
        }
        const element = node();
        if (element && element.reset) element.reset();
      },
      submit() {
        const element = node();
        if (element && element.requestSubmit) element.requestSubmit();
        else if (element && element.submit) element.submit();
      },
      /// Called by the submit handler before the author's own: it says
      /// whether to go on, and moves focus when it says not to.
      __submitting() {
        submitted.set(true);
        if (isValid()) return true;
        focusFirstInvalid(node(), fields);
        return false;
      },
      __register(name, entry) {
        fields.set(name || String(fields.size), entry);
        registered.update((n) => n + 1);
      },
      __node: node,
    };
    return handle;
  }

  /// Focus the first control that has something to say, and say it — which
  /// is what a reader who cannot see the form needs on a failed submit.
  function focusFirstInvalid(element, fields) {
    for (const [name, entry] of fields) {
      if (!entry.message()) continue;
      const control = element && element.querySelector
        ? element.querySelector(`[name="${name}"]`) || element.querySelector("[aria-invalid='true']")
        : null;
      if (control && control.focus) control.focus();
      announce(entry.message());
      return;
    }
  }
