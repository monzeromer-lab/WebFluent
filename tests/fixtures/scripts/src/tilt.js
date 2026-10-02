// What happened, for the browser test to read.
var events = [];
var renders = [];

/**
 * Lean an element, and say so on it.
 * @param {HTMLElement} node
 * @param {number} max
 */
function initTilt(node, max) {
  node.dataset.tilt = String(max);
  node.classList.add("from-script");
  events.push("mount " + max);
  return {
    setMax(m) { node.dataset.tilt = String(m); events.push("update " + m); },
    destroy() { events.push("cleanup"); },
  };
}

/**
 * Report straight away that the element is on screen.
 * @param {HTMLElement} node
 * @param {(v: boolean) => void} onChange
 */
function watchVisible(node, onChange) {
  onChange(true);
}

document.addEventListener("wf:render", (e) => renders.push(e.detail.route));
