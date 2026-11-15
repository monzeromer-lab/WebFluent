  // ─── Results ─────────────────────────────────────────
  // A result is `["ok", value]` (`"ok"` when there is none) or
  // `["err", error]`. `try r` is `tryOk(r, target)`: the value, or the error
  // thrown up to the action that declares what it fails with, which hands
  // it back as its own `.err`.

  /// `try r`: the value of `r`, or its error passed up. `target` is what
  /// the action fails with: `"enum"`, an error enum, which takes an
  /// `HttpError` as its `http` case; `"http"`, an `HttpError`, which takes
  /// an error enum's `http` case as what it holds; `"same"`, an error of
  /// the same type; `"report"`, no action — a handler — which reports it.
  function tryOk(r, target) {
    if (r === "ok") return undefined;
    if (Array.isArray(r) && r[0] === "ok") return r[1];
    const failed = Array.isArray(r) ? r[1] : r;
    let error = failed;
    if (target === "enum" && failed && failed.httpError) error = ["http", failed];
    else if (target === "http" && Array.isArray(failed) && !failed.httpError && failed[0] === "http") error = failed[1];
    if (target === "report") {
      const message = (failed && failed.message) || (Array.isArray(failed) && failed[1] && failed[1].message) || JSON.stringify(failed);
      throw new Error("`try` failed: " + message);
    }
    throw { wfTried: true, error };
  }
