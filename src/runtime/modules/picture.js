  // ─── A picture the build made ────────────────────────
  //
  // `Image(hero, alt: "…", sizes: "…")` where `hero` is an `image` the
  // program declares: the build wrote the file at several widths and in
  // several formats, and this puts them in front of the browser so it can
  // pick. The box is already the right shape — `width` and `height` come
  // from the real file — so nothing on the page moves when the image
  // lands, and a placeholder stands in the meantime.
  function picture(asset, options) {
    const opts = options || {};
    const held = typeof asset === "function" ? asset() : asset;
    const source = held && typeof held === "object" ? held : { src: held };

    const img = el("img", {
      src: source.src,
      alt: opts.alt != null ? opts.alt : "",
      className: opts.className,
      decoding: "async",
      loading: opts.loading,
    });
    if (source.width) img.setAttribute("width", String(source.width));
    if (source.height) img.setAttribute("height", String(source.height));
    if (opts.sizes) img.setAttribute("sizes", opts.sizes);
    // The widths of the original format, for a browser that takes none of
    // the others.
    if (source.srcset) img.setAttribute("srcset", source.srcset);

    // What fills the box until the image arrives. It is cleared on load,
    // and on an error too — a broken image should not sit behind a blur.
    const kind = opts.placeholder || "none";
    if (kind !== "none" && (source.placeholder || source.color)) {
      const fill = kind === "color" ? source.color : `url("${source.placeholder}")`;
      img.style.background = kind === "color" ? fill : `${fill} center / cover no-repeat`;
      if (source.width && source.height) {
        img.style.aspectRatio = `${source.width} / ${source.height}`;
      }
      const clear = () => { img.style.background = ""; };
      img.addEventListener("load", clear);
      img.addEventListener("error", clear);
    }

    const sources = source.sources || [];
    if (!sources.length) return img;

    const wrapper = el("picture", {});
    for (const one of sources) {
      const tag = el("source", { type: one.type, srcset: one.srcset });
      if (opts.sizes) tag.setAttribute("sizes", opts.sizes);
      wrapper.appendChild(tag);
    }
    wrapper.appendChild(img);
    return wrapper;
  }
