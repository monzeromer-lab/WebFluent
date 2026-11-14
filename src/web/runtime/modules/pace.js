
  // ─── How often a handler runs ────────────────────────
  //
  // `on input(debounce: 300.ms)` runs once the events have stopped coming
  // for that long, with the last of them; `on scroll(throttle: 100.ms)` at
  // most once in that long — the first at once, the last kept for the end
  // of the wait, so where a scroll stopped is always heard. A wait still
  // pending when its page, branch or item leaves is dropped.

  function debounce(fn, ms) {
    let timer = 0;
    onCleanup(() => clearTimeout(timer));
    return function (...args) {
      clearTimeout(timer);
      timer = setTimeout(() => {
        timer = 0;
        fn.apply(this, args);
      }, Number(ms) || 0);
    };
  }

  function throttle(fn, ms) {
    let last = -Infinity;
    let timer = 0;
    let held = null;
    onCleanup(() => clearTimeout(timer));
    const wait = Number(ms) || 0;
    return function (...args) {
      const now = Date.now();
      if (now - last >= wait) {
        last = now;
        fn.apply(this, args);
        return;
      }
      held = args;
      if (timer) return;
      timer = setTimeout(() => {
        timer = 0;
        last = Date.now();
        const a = held;
        held = null;
        fn.apply(this, a);
      }, wait - (now - last));
    };
  }
