// Where the reader is on a page: the section they are reading, for the
// rails that list a page's sections, and whether they have reached the end,
// for the book's progress.

/**
 * Follow the section the reader is in: the last of the headings named by
 * `ids` whose top has passed a line a quarter down the window (the last
 * one once the page is scrolled to its end). `seen` hears its id each time
 * it changes.
 * @param {string[]} ids the headings' ids, in the page's order
 * @param {(id: string) => void} seen
 * @returns {{ stop(): void }}
 */
function wfSpy(ids, seen) {
  let frame = 0;
  let last = null;
  const pick = () => {
    frame = 0;
    const heads = ids.map((id) => document.getElementById(id)).filter(Boolean);
    if (!heads.length) return;
    const line = window.innerHeight * 0.25;
    let current = heads[0];
    for (const h of heads) {
      if (h.getBoundingClientRect().top <= line) current = h;
      else break;
    }
    const end = window.innerHeight + window.scrollY >= document.documentElement.scrollHeight - 2;
    if (end) current = heads[heads.length - 1];
    if (current.id !== last) {
      last = current.id;
      seen(current.id);
    }
  };
  const soon = () => {
    if (!frame) frame = requestAnimationFrame(pick);
  };
  addEventListener("scroll", soon, { passive: true });
  addEventListener("resize", soon);
  soon();
  return {
    stop() {
      removeEventListener("scroll", soon);
      removeEventListener("resize", soon);
      cancelAnimationFrame(frame);
    },
  };
}

/**
 * Call `seen` once, the first time `node` comes into view: the end of a
 * chapter, reached.
 * @param {HTMLElement} node
 * @param {() => void} seen
 * @returns {{ stop(): void }}
 */
function wfWhenSeen(node, seen) {
  if (!("IntersectionObserver" in window)) {
    seen();
    return { stop() {} };
  }
  const watch = new IntersectionObserver((entries) => {
    if (entries.some((e) => e.isIntersecting)) {
      watch.disconnect();
      seen();
    }
  });
  watch.observe(node);
  return {
    stop() {
      watch.disconnect();
    },
  };
}
