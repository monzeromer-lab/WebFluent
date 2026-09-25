  function slot(parent, values, render) {
    const marker = document.createComment("wf-slot");
    parent.appendChild(marker);
    let currentNodes = [];
    let dispose = null;
    effect(() => {
      const handed = values();
      if (dispose) { dispose(); dispose = null; }
      leaveThenRemove(currentNodes, null);
      currentNodes = [];
      const prev = currentEffect;
      currentEffect = null;
      let result;
      try {
        [result, dispose] = scoped(() => render(handed));
      } finally {
        currentEffect = prev;
      }
      if (result == null) return;
      const nodes = result instanceof DocumentFragment
        ? [...result.childNodes]
        : [].concat(result).flat().filter(n => n instanceof Node);
      currentNodes = nodes.slice();
      const frag = document.createDocumentFragment();
      for (const n of nodes) frag.appendChild(n);
      if (marker.parentNode) marker.parentNode.insertBefore(frag, marker.nextSibling);
    });
  }
