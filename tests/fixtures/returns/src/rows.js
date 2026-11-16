/** @typedef {{ id: string, title: string }} Row */

/**
 * The rows as they should be.
 * @returns {Row[]}
 */
function goodRows() {
  return [{ id: "1", title: "Milk" }];
}

/**
 * Rows missing what the type says they have.
 * @returns {Row[]}
 */
function badRows() {
  return [{ id: "1" }];
}

/**
 * A promise of a count that arrives as text.
 * @returns {Promise<number>}
 */
async function laterCount() {
  return "seven";
}
