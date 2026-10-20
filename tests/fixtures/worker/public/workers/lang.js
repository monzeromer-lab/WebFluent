// A language worker, as small as one can be: it answers `upper`, fails
// `fail`, answers `slow` late (and is told when a newer one supersedes
// it), never answers `never`, pushes diagnostics on `lint`, and crashes
// on `crash`.
const cancelled = new Set();
self.onmessage = (event) => {
  const { id, method, params, cancel } = event.data || {};
  if (cancel !== undefined) {
    cancelled.add(cancel);
    self.postMessage({ type: "cancelled", id: cancel });
    return;
  }
  if (method === "upper") self.postMessage({ id, result: String(params.text).toUpperCase() });
  else if (method === "fail") self.postMessage({ id, error: { message: "no such word", code: 7 } });
  else if (method === "slow") setTimeout(() => { if (!cancelled.has(id)) self.postMessage({ id, result: `slow ${params.n}` }); }, 300);
  else if (method === "never") { /* no answer */ }
  else if (method === "lint") self.postMessage({ type: "diagnostics", items: [{ line: 1, message: `${params.text.length} characters` }] });
  else if (method === "crash") throw new Error("the worker fell over");
};
