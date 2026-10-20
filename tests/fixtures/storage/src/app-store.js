// A storage of the project's own, as a desktop app's store would be: what
// an earlier session left is seeded, a read answers a moment later (as an
// IPC call does), and every write is recorded for the test to see.
window.wfWrites = [];
const kept = new Map([["wf:Layout.side:halyard", "320"], ["wf:Home.mode", "\"design\""]]);
window.wfStorage = {
  app: {
    get(name) {
      return new Promise((resolve) => setTimeout(() => resolve(kept.has(name) ? kept.get(name) : null), 20));
    },
    set(name, text) {
      kept.set(name, text);
      window.wfWrites.push(`${name}=${text}`);
    },
  },
};
