  // ─── Form fields ─────────────────────────────────────
  //
  // `Input(label: …)`, `Select(label: …)`: the label has to be a <label>
  // pointing at the control for a screen reader to read it and a click on
  // it to focus the field. `hint:` is a description the control refers to;
  // `error:` is a message that, while it is not empty, is announced, refers
  // to the control, and marks it invalid — so a validation failure reaches
  // a reader who cannot see the red text.
  let fieldSeq = 0;

  function field(control, opts) {
    const wrapper = el("div", { className: "wf-field" });
    if (!control.id) control.id = "wf-field-" + (++fieldSeq);
    const described = [];
    const existing = control.getAttribute("aria-describedby");
    if (existing) described.push(existing);

    if (opts.label != null) {
      wrapper.appendChild(
        el("label", { className: "wf-label", for: control.id }, opts.label),
      );
    }
    wrapper.appendChild(control);

    if (opts.hint != null) {
      const id = control.id + "-hint";
      wrapper.appendChild(el("p", { className: "wf-field__hint", id }, opts.hint));
      described.push(id);
    }
    if (opts.error !== undefined) {
      const id = control.id + "-error";
      const message = el("p", { className: "wf-field__error", id, role: "alert" });
      wrapper.appendChild(message);
      described.push(id);
      const apply = (value) => {
        const has = value != null && value !== false && value !== "";
        message.textContent = has ? String(value) : "";
        if (has) message.removeAttribute("hidden");
        else message.setAttribute("hidden", "");
        control.setAttribute("aria-invalid", has ? "true" : "false");
      };
      if (typeof opts.error === "function") effect(() => apply(opts.error()));
      else apply(opts.error);
    }
    // A control with a limit says how much is left, and says it politely:
    // a count that shouted on every keystroke would be unusable.
    const limit = Number(control.getAttribute && control.getAttribute("maxlength"));
    if (limit) {
      const id = control.id + "-count";
      const count = el("span", { className: "wf-field__count", id, "aria-live": "polite" });
      const show = () => {
        const used = (control.value || "").length;
        count.textContent = `${used} / ${limit}`;
      };
      control.addEventListener("input", show);
      show();
      wrapper.appendChild(count);
      described.push(id);
    }
    if (described.length) control.setAttribute("aria-describedby", described.join(" "));
    return wrapper;
  }

  /// Wire a tooltip: the tip describes the trigger, and can be reached and
  /// dismissed without a pointer.
  ///
  /// `role="tooltip"` alone announces nothing — the element the reader is on
  /// has to refer to it — so `aria-describedby` goes on the first focusable
  /// thing inside the wrapper, and on the wrapper itself (made focusable)
  /// when there is none. The stylesheet shows the tip on focus as well as on
  /// hover; Escape hides it until the pointer or focus leaves and returns
  /// (WCAG 1.4.13: content on hover or focus must be dismissible).
  function _label(key, fallback) {
    if (i18nInstance) {
      const text = i18nInstance.t(key);
      if (text !== key) return text;
    }
    return fallback;
  }

  /// Wire a carousel: the slides in `root`'s track, the controls, the rotation.
  ///
  /// The WAI-ARIA carousel pattern: the root is a region a reader can name,
  /// each slide a group announced as "n of N", the slides off screen hidden
  /// from assistive technology and from the tab order. Rotation that a
  /// reader cannot stop fails WCAG 2.2.2, so autoplay has a pause button,
  /// pauses while the pointer or focus is on it, does not start at all for
  /// a reader who asked for reduced motion, and stops while the tab is
  /// hidden. While it is not rotating, the track is a polite live region,
  /// so a change made by the controls is read.
