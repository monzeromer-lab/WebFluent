  // ─── Offline ─────────────────────────────────────────
  //
  // The page's side of the service worker `wf build` writes when the config
  // names `offline`: registering it, the update flow — `update.available`,
  // `update.apply()` — and, with `sync`, the writes made while the network
  // was gone, kept in IndexedDB and sent in order when it returns.
  let _update = null;
  let _waitingWorker = null;
  let _applying = false;

  /// `update.available` — a new version of the site has installed and waits
  /// for the page to take it; `update.apply()` takes it, and reloads once.
  function update() {
    if (!_update) _update = signal(false);
    return {
      get available() { return _update(); },
      apply() {
        if (!_waitingWorker) return;
        _applying = true;
        _waitingWorker.postMessage({ type: "wf:activate" });
      },
    };
  }

  function offline(options) {
    const opts = options || {};
    if (!_update) _update = signal(false);
    if (typeof navigator === "undefined" || !navigator.serviceWorker) return;
    const sw = navigator.serviceWorker;
    // Under `wf serve` a worker would answer from its own store and hide
    // every edit, so the dev server's page takes any worker away instead.
    if (typeof document !== "undefined" && document.querySelector('script[src$="/__wf/dev.js"]')) {
      sw.getRegistrations().then((all) => all.forEach((r) => r.unregister()));
      return;
    }
    // Reloaded once, and only for an update. The first install takes over a
    // page that was already working — reloading it would only throw its state
    // away — and reloading on every `controllerchange` is how a page ends up
    // refreshing twice for one update.
    const hadController = !!sw.controller;
    let reloaded = false;
    sw.addEventListener("controllerchange", () => {
      if (reloaded || !(hadController || _applying)) return;
      reloaded = true;
      location.reload();
    });
    if (opts.sync) {
      offlineWrites = queueWrite;
      if (!offlineQueued) offlineQueued = signal(0);
      refreshQueued();
      // The worker says when it has sent some, so the count follows.
      sw.addEventListener("message", (e) => {
        if (e.data && e.data.type === "wf:writes") refreshQueued();
      });
    }
    sw.register(`${_basePath}/sw.js`, { scope: `${_basePath}/`, updateViaCache: "none" })
      .then((reg) => {
        const noteWaiting = () => {
          _waitingWorker = reg.waiting;
          _update.set(!!(reg.waiting && sw.controller));
        };
        if (reg.waiting) noteWaiting();
        reg.addEventListener("updatefound", () => {
          const worker = reg.installing;
          if (!worker) return;
          worker.addEventListener("statechange", () => {
            if (worker.state === "installed") noteWaiting();
          });
        });
        // A tab left open asks for a new version when it comes back into view.
        if (typeof document !== "undefined") {
          document.addEventListener("visibilitychange", () => {
            if (document.visibilityState === "visible") reg.update().catch(() => {});
          });
        }
        if (opts.sync) startSending(reg);
      })
      .catch(() => {});
  }

  // ─── Writes kept for later ──
  //
  // One store, `writes` in the `wf-offline` database, which the worker reads
  // too: where the browser has Background Sync the worker sends them, closed
  // page or not; elsewhere the page does.
  const WRITES_DB = "wf-offline";
  const WRITES = "writes";
  let _syncReg = null;

  function writesStore(mode, work) {
    return new Promise((resolve, reject) => {
      if (typeof indexedDB === "undefined") return reject(new Error("no IndexedDB"));
      const open = indexedDB.open(WRITES_DB, 1);
      open.onupgradeneeded = () => {
        open.result.createObjectStore(WRITES, { keyPath: "id", autoIncrement: true });
      };
      open.onerror = () => reject(open.error);
      open.onsuccess = () => {
        const db = open.result;
        const tx = db.transaction(WRITES, mode);
        let out;
        tx.oncomplete = () => { db.close(); resolve(out); };
        tx.onerror = () => { db.close(); reject(tx.error); };
        work(tx.objectStore(WRITES), (value) => { out = value; });
      };
    });
  }

  const allWrites = () =>
    writesStore("readonly", (store, done) => {
      const req = store.getAll();
      req.onsuccess = () => done(req.result || []);
    });

  function refreshQueued() {
    if (!offlineQueued) return;
    writesStore("readonly", (store, done) => {
      const req = store.count();
      req.onsuccess = () => done(req.result);
    }).then((n) => offlineQueued.set(n || 0), () => {});
  }

  /// What the request engine hands over when a write fails for want of a
  /// network: kept, and the call resolves with nothing so what it showed
  /// stays shown. A body that cannot be stored — a `File`, a `FormData` — is
  /// not kept, and the caller gets the error as before.
  function queueWrite(request) {
    if (request.body != null && typeof request.body !== "string") return null;
    const write = {
      url: request.url,
      method: request.method,
      headers: request.headers || {},
      body: request.body == null ? null : request.body,
      at: Date.now(),
    };
    return writesStore("readwrite", (store) => { store.add(write); }).then(() => {
      refreshQueued();
      askToSend();
      return undefined;
    });
  }

  function askToSend() {
    if (_syncReg && _syncReg.sync) _syncReg.sync.register("wf-writes").catch(() => {});
    sendWrites();
  }

  let _sending = false;
  /// Every kept write, oldest first, until one cannot be sent. A server that
  /// answers — even to refuse — has the write, and a retry would not change
  /// its mind; one that is down, busy or asking to wait keeps it for later.
  ///
  /// The worker sends them too, for a page that has closed, so both take the
  /// `wf-writes` lock first: whoever holds it sends, deletes what went, and
  /// the other finds nothing left — no write goes twice.
  function sendWrites() {
    if (_sending) return Promise.resolve();
    _sending = true;
    const send = async () => {
      for (const write of await allWrites()) {
        let response;
        try {
          response = await fetch(write.url, { method: write.method, headers: write.headers, body: write.body });
        } catch (e) {
          return;
        }
        if (response.status >= 500 || response.status === 408 || response.status === 429) return;
        await writesStore("readwrite", (store) => { store.delete(write.id); });
      }
    };
    const locked = typeof navigator !== "undefined" && navigator.locks
      ? navigator.locks.request("wf-writes", send)
      : send();
    return Promise.resolve(locked)
      .catch(() => {})
      .finally(() => {
        _sending = false;
        refreshQueued();
      });
  }

  function startSending(reg) {
    // Background Sync, where there is one, wakes the worker for a page that
    // has closed; the page itself sends whenever it can.
    if (typeof window !== "undefined" && "SyncManager" in window && reg.sync) _syncReg = reg;
    const again = () => sendWrites();
    if (typeof window !== "undefined" && window.addEventListener) window.addEventListener("online", again);
    if (typeof document !== "undefined") {
      document.addEventListener("visibilitychange", () => {
        if (document.visibilityState === "visible") again();
      });
    }
    // The browser can be online while the server is not, so while anything
    // waits it is tried again now and then.
    setInterval(() => { if (offlineQueued && offlineQueued() > 0) again(); }, 30000);
    again();
  }
