
  // ─── Calls that are under way ────────────────────────
  //
  // An async action's calls go through `sameCall`: a call with the same
  // arguments as one still running is that call — the same promise, the
  // work not done twice — so a double click submits once. A call with other
  // arguments (another row's toggle) runs beside it. An action written
  // `concurrent` runs every call.
  //
  // A button's click and a form's submit go through `busy`: what async
  // calls the handler starts are watched, and while any runs the control
  // says so (`aria-busy`, `aria-disabled`) and takes no other press. The
  // focus stays where it is, which a `disabled` button would drop.

  // The calls the handler running now has started, when one is listening.
  let _started = null;

  function sameCall(pending, args, run, concurrent) {
    let key = null;
    if (!concurrent) {
      try {
        key = JSON.stringify(args);
      } catch (e) {
        key = null;
      }
    }
    const calls = pending.calls || (pending.calls = new Map());
    let call = key !== null ? calls.get(key) : undefined;
    if (!call) {
      call = run();
      if (key !== null && call && typeof call.then === "function") {
        calls.set(key, call);
        const done = () => {
          if (calls.get(key) === call) calls.delete(key);
        };
        call.then(done, done);
      }
    }
    if (_started) _started.push(call);
    return call;
  }

  function busy(el, handler) {
    let running = 0;
    return function (e) {
      if (running || el.getAttribute("aria-busy") === "true") {
        // A second press while the first is still at work: nothing, and a
        // form is not sent by the browser either.
        if (e && e.type === "submit" && e.preventDefault) e.preventDefault();
        return undefined;
      }
      const outer = _started;
      const mine = [];
      _started = mine;
      let out;
      try {
        out = handler.call(this, e);
      } finally {
        _started = outer;
      }
      if (out && typeof out.then === "function") mine.push(out);
      const work = mine.filter((p) => p && typeof p.then === "function");
      if (work.length) {
        running += 1;
        el.setAttribute("aria-busy", "true");
        el.setAttribute("aria-disabled", "true");
        const end = () => {
          running -= 1;
          if (!running) {
            el.removeAttribute("aria-busy");
            el.removeAttribute("aria-disabled");
          }
        };
        Promise.allSettled(work).then(end);
      }
      return out;
    };
  }
