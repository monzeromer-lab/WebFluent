// A custom element of the project's own, placed with `Element`.
class XBadge extends HTMLElement {
  static get observedAttributes() { return ["label"]; }
  connectedCallback() {
    this.render();
    this.dispatchEvent(new Event("ready"));
  }
  attributeChangedCallback() { this.render(); }
  render() { this.textContent = "badge: " + (this.getAttribute("label") || ""); }
}
customElements.define("x-badge", XBadge);
