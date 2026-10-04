// Plain browser code the .wf files call by name. A .js file under src/ is
// linked on every page as it is — no modules, no build step — and its JSDoc
// types the calls, so `saveTextFile(1)` is a compile error.

/**
 * Hand the reader a file to save, made from a string.
 *
 * Three steps of DOM work — make the file, point a link at it, click the
 * link — which read best as plain JavaScript.
 *
 * @param {string} name what the file is called
 * @param {string} text what is in it
 * @param {string} type its media type, such as "text/csv"
 */
function saveTextFile(name, text, type) {
  const url = URL.createObjectURL(new Blob([text], { type: type + ";charset=utf-8" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = name;
  document.body.appendChild(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
