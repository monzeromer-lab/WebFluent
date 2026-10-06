// Puppeteer downloads its browsers here rather than into ~/.cache, so the
// benchmark can measure exactly what an install brings and a clean run
// starts from nothing.
const { join } = require("path");

module.exports = {
  cacheDirectory: join(__dirname, ".cache", "puppeteer"),
};
