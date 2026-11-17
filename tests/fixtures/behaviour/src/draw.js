/**
 * Draw a circle into an SVG host, and say beside it what the host is.
 * @param {Element} node
 */
function markDrawing(node) {
  node.setAttribute("viewBox", "0 0 100 100");
  node.setAttribute("width", "100");
  node.setAttribute("height", "100");
  const c = document.createElementNS("http://www.w3.org/2000/svg", "circle");
  c.setAttribute("cx", "50");
  c.setAttribute("cy", "50");
  c.setAttribute("r", "40");
  node.append(c);
  // Once the host is in the page: what it was made as, and how wide the
  // circle drawn in it paints (0 when the browser took it for HTML).
  setTimeout(() => {
    const said = document.createElement("p");
    const kind = node.namespaceURI === "http://www.w3.org/2000/svg" ? "svg" : "html";
    said.textContent = "drawn as " + kind + ", " + Math.round(c.getBoundingClientRect().width) + " wide";
    node.after(said);
  }, 50);
}
