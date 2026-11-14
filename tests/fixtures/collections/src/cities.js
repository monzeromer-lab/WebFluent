/**
 * Cities whose name starts with what was typed, as a server would answer.
 * @param {string} q
 * @returns {Promise<Array<{ name: string, country: string }>>}
 */
function findCities(q) {
  const all = [
    { name: "Paris", country: "France" },
    { name: "Palermo", country: "Italy" },
    { name: "Porto", country: "Portugal" },
  ];
  const want = String(q || "").toLowerCase();
  return new Promise((resolve) => setTimeout(() => resolve(all.filter((c) => c.name.toLowerCase().startsWith(want))), 20));
}
