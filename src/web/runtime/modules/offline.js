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
      offlineClearRefused = clearRefused;
      if (!offlineRefused) offlineRefused = signal([]);
      if (!offlineQueued) offlineQueued = signal(0);
      refreshQueued();
      // The worker says when it has sent some, so the count follows.
      sw.addEventListener("message", (e) => {
        if (!e.data || e.data.type !== "wf:writes") return;
        // What the writes it sent made stale asks again here.
        staleNow(e.data.stale);
        refreshQueued();
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

  /// What a write that was sent made stale asks again: the reads its
  /// endpoint `invalidates`, by their cache keys. Only a build with the
  /// request engine has any.
  function staleNow(keys) {
    if (typeof invalidate !== "function") return;
    for (const key of keys || []) if (typeof key === "string") invalidate(key);
  }

  // ─── Writes kept for later ──
  //
  // One store, `writes` in the `wf-offline` database, which the worker reads
  // too: where the browser has Background Sync the worker sends them, closed
  // page or not; elsewhere the page does. A site under a base path has a
  // database of its own (`wf-offline:/shop`), so two sites on one origin
  // never send each other's writes.
  const WRITES = "writes";
  let _syncReg = null;
  const writesDb = () => (_basePath ? "wf-offline:" + _basePath : "wf-offline");
  const writesLock = () => (_basePath ? "wf-writes:" + _basePath : "wf-writes");

  function writesStore(mode, work) {
    return new Promise((resolve, reject) => {
      if (typeof indexedDB === "undefined") return reject(new Error("no IndexedDB"));
      const open = indexedDB.open(writesDb(), 1);
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

  /// `network.queued` and `network.refused`, read again from the store:
  /// the writes still waiting, and those the server refused once sent.
  function refreshQueued() {
    if (!offlineQueued) return;
    allWrites().then((all) => {
      offlineQueued.set(all.filter((w) => !w.refused).length);
      if (offlineRefused) offlineRefused.set(all.filter((w) => w.refused).map((w) => ({
        method: w.method,
        url: w.url,
        status: w.refused,
        at: new Date(w.settled || w.at).toISOString(),
      })));
    }, () => {});
  }

  /// What the request engine hands over when a write fails for want of a
  /// network: kept, and the call resolves with nothing so what it showed
  /// stays shown. A body that cannot be stored — a `File`, a `FormData` — is
  /// not kept, and the caller gets the error as before.
  ///
  /// A credential is never stored. What a service's `on request` hook adds
  /// is added again when the write is sent, by the hook as it is then — so
  /// a write made before a logout goes with no one's token, and one made by
  /// another user with theirs. A write whose call itself carries a
  /// credential header cannot be kept without it, so it is not kept: the
  /// caller gets the error.
  function queueWrite(request) {
    if (request.body != null && typeof request.body !== "string") return null;
    const given = request.given || {};
    if (_credentialHeaders(given).length) return null;
    const write = {
      url: request.url,
      method: request.method,
      headers: _withoutCredentials(given),
      api: request.hooked ? request.api || null : null,
      credentials: request.credentials || null,
      body: request.body == null ? null : request.body,
      stale: Array.isArray(request.stale) ? request.stale.filter((k) => typeof k === "string") : [],
      at: Date.now(),
    };
    // Hooked, with no name to find its hook by later: it cannot be sent as
    // it was meant to be.
    if (request.hooked && !write.api) return null;
    return writesStore("readwrite", (store) => { store.add(write); }).then(() => {
      refreshQueued();
      // Sent after this call has said it was kept, so a write that goes at
      // once is still one the page heard of as kept.
      setTimeout(askToSend, 0);
      return undefined;
    });
  }

  function askToSend() {
    if (_syncReg && _syncReg.sync) _syncReg.sync.register(writesLock()).catch(() => {});
    sendWrites();
  }

  let _sending = false;
  /// Every kept write, oldest first, until one cannot be sent. A server that
  /// answers — even to refuse — has the write, and a retry would not change
  /// its mind; one that is down, busy or asking to wait keeps it for later.
  /// One it took makes stale what its endpoint `invalidates`; one it refused
  /// is kept aside, for `network.refused`, and never sent again.
  ///
  /// The worker sends them too, for a page that has closed, so both take the
  /// `wf-writes` lock first: whoever holds it sends, deletes what went, and
  /// the other finds nothing left — no write goes twice.
  function sendWrites() {
    if (_sending) return Promise.resolve();
    _sending = true;
    const send = async () => {
      for (const write of await allWrites()) {
        if (write.refused) continue;
        let request = {
          url: write.url,
          method: write.method,
          // A write kept by an older build may hold a credential: it is not
          // sent on.
          headers: _withoutCredentials(write.headers),
          body: write.body,
        };
        if (write.credentials) request.credentials = write.credentials;
        if (write.api) {
          // The service's hook gives the credentials of now. A page that
          // has not opened the service yet keeps the write, and the ones
          // after it, in order, for one that has.
          const hook = _requestHooks.get(write.api);
          if (!hook) return;
          try {
            request = (await hook(request)) || request;
          } catch (e) {
            return;
          }
        }
        let response;
        try {
          const init = { method: request.method, headers: request.headers, body: request.body };
          if (request.credentials) init.credentials = request.credentials;
          response = await fetch(request.url, init);
        } catch (e) {
          return;
        }
        if (response.status >= 500 || response.status === 408 || response.status === 429) return;
        if (response.status >= 400) {
          const refused = { ...write, refused: response.status, settled: Date.now() };
          await writesStore("readwrite", (store) => { store.put(refused); });
        } else {
          await writesStore("readwrite", (store) => { store.delete(write.id); });
          staleNow(write.stale);
        }
      }
    };
    const locked = typeof navigator !== "undefined" && navigator.locks
      ? navigator.locks.request(writesLock(), send)
      : send();
    return Promise.resolve(locked)
      .catch(() => {})
      .finally(() => {
        _sending = false;
        refreshQueued();
      });
  }

  /// Forget the writes the server refused, once the page has said so:
  /// `network.clearRefused()`.
  function clearRefused() {
    return writesStore("readwrite", (store) => {
      const req = store.getAll();
      req.onsuccess = () => { for (const w of req.result || []) if (w.refused) store.delete(w.id); };
    })
      .catch(() => {})
      .then(() => { refreshQueued(); });
  }

  /// Forget every write kept for later — for a logout, or a reader who
  /// would rather not send them. `WF.clearQueued()` from a project's script.
  function clearQueued() {
    return writesStore("readwrite", (store) => { store.clear(); })
      .catch(() => {})
      .then(() => { refreshQueued(); });
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
