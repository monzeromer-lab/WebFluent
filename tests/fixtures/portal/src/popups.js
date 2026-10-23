// A layer of popups a library made outside the page's root, as an editor
// makes its tooltips: each is an empty node the library places, and the
// page draws what is inside it.
const layer = document.createElement("div");
layer.id = "popup-layer";
for (const name of ["a", "b"]) {
  const popup = document.createElement("div");
  popup.id = `popup-${name}`;
  popup.className = "popup";
  layer.appendChild(popup);
}
document.body.appendChild(layer);

/**
 * The node the library made for a popup, or null when it has none.
 * @param {string} name
 * @returns {HTMLElement|null}
 */
function popupHost(name) {
  return document.getElementById(`popup-${name}`);
}

/**
 * What a mount inside the popup saw of its node.
 * @param {HTMLElement} node
 * @returns {boolean}
 */
function sawMount(node) {
  window.mounted = (window.mounted || []).concat([node.isConnected]);
  return node.isConnected;
}
