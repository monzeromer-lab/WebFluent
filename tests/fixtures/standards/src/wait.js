/**
 * Resolve after a while: a request that takes its time.
 * @param {number} ms
 */
function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
