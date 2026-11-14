
  // ─── Sheet ───────────────────────────────────────────
  //
  // A modal `<dialog>` (`WF.dialog` opens and closes it) that comes in
  // from an edge: the bottom, or the start or end of the line, which are
  // swapped in a right-to-left page. Dragged by its handle or its heading
  // toward that edge past 30% of its size, or flicked, it closes; let go
  // short of that it springs back. A bottom sheet with `snap` points —
  // fractions of the window's height, `[0.5, 1]` — opens at the first and
  // settles at the nearest one it is let go near.
  function sheet(box, options) {
    live(box);
    const opts = options || {};
    const side = opts.side || "bottom";
    const snaps = (opts.snap || []).map(Number).filter((n) => n > 0 && n <= 1).sort((a, b) => a - b);
    box.setAttribute("data-side", side);
    let at = 0;
    const fit = () => {
      if (side !== "bottom" || !snaps.length || !box.style) return;
      box.style.height = `${snaps[at] * 100}dvh`;
    };
    fit();
    box.addEventListener("close", () => {
      at = 0;
      fit();
    });
    // How far a move went toward the edge the sheet closes to.
    const closing = (dx, dy) => {
      if (side === "bottom") return dy;
      const rtl = _isRtl(box);
      const toStart = side === "start" ? !rtl : rtl;
      return toStart ? -dx : dx;
    };
    const set = (px) => {
      if (box.style && box.style.setProperty) box.style.setProperty("--wf-sheet-drag", `${Math.max(0, px)}px`);
    };
    let from = null;
    box.addEventListener("pointerdown", (e) => {
      if (e.button > 0 || !e.target || !e.target.closest) return;
      if (!e.target.closest(".wf-sheet__handle, .wf-sheet__header")) return;
      // A press on a control in the heading (the close button) is that
      // control's.
      if (e.target.closest("button, a, input, select, textarea") && !e.target.closest(".wf-sheet__handle")) return;
      from = { x: e.clientX, y: e.clientY, t: Date.now(), size: side === "bottom" ? box.offsetHeight : box.offsetWidth, moved: 0 };
      box.setAttribute("data-dragging", "");
      if (box.setPointerCapture && e.pointerId !== undefined) box.setPointerCapture(e.pointerId);
    });
    box.addEventListener("pointermove", (e) => {
      if (!from) return;
      from.moved = closing(e.clientX - from.x, e.clientY - from.y);
      if (from.moved >= 0) set(from.moved);
      else if (side === "bottom" && snaps.length && box.style) {
        // Pulled up: as tall as the pull, up to the last point.
        const most = snaps[snaps.length - 1] * window.innerHeight;
        box.style.height = `${Math.min(most, from.size - from.moved)}px`;
      }
    });
    const end = () => {
      if (!from) return;
      const { moved, size, t } = from;
      from = null;
      box.removeAttribute("data-dragging");
      set(0);
      const quick = Date.now() - t < 250 && moved > 40;
      if (moved > size * 0.3 || quick) {
        if (box._wfShut) box._wfShut();
        return;
      }
      if (side === "bottom" && snaps.length) {
        const now = (box.offsetHeight || size) / (window.innerHeight || 1);
        let best = 0;
        snaps.forEach((p, i) => {
          if (Math.abs(p - now) < Math.abs(snaps[best] - now)) best = i;
        });
        at = best;
        fit();
      }
    };
    box.addEventListener("pointerup", end);
    box.addEventListener("pointercancel", end);
  }
