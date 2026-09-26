  // ─── A peer ──────────────────────────────────────────
  //
  // `peer link = rtc(signal: m => room.post(m))`: a WebRTC data channel to
  // another page. The language supplies the channel and not a server: what
  // the two sides must tell each other to connect — an offer, an answer,
  // the routes they can be reached by — goes out through `signal`, over
  // whatever transport the author already has (a socket, a channel, an
  // `api`), and what the other side sent comes back in through
  // `link.signal(m)`. One side is the `initiator`. The handle is a socket's:
  // `state`, `messages`, `send`, `last`, `close` — and closes with its scope.
  function rtc(options) {
    const opts = options || {};
    const state = signal("connecting");
    const messages = signal([]);
    const closure = signal(null);
    const failure = signal(null);
    const waiting = [];
    // What the other side offered as a route before it said who it is.
    const early = [];
    let pc = null;
    let channel = null;

    const fail = (error) => {
      failure.set(error instanceof Error ? error : new Error(String(error)));
      state.set("error");
    };
    const tell = (message) => {
      try {
        if (opts.signal) opts.signal(message);
      } catch (e) {
        fail(e);
      }
    };
    const attach = (ch) => {
      channel = ch;
      ch.onopen = () => {
        state.set("open");
        while (waiting.length) ch.send(waiting.shift());
        if (opts.onOpen) opts.onOpen();
      };
      ch.onmessage = (event) => {
        let data = event.data;
        if (typeof data === "string") {
          try { data = JSON.parse(data); } catch (e) { /* a plain string */ }
        }
        messages.update((held) => [...held, data]);
        if (opts.onMessage) opts.onMessage(data);
      };
      ch.onclose = () => {
        if (!closure()) closure.set({ code: 1000, reason: "the channel closed" });
        if (state() !== "error") state.set("closed");
      };
      ch.onerror = (event) => {
        const error = event && event.error;
        // The other side closing on purpose arrives as an error — an SCTP
        // abort, cause 12, "User-Initiated Abort" — just before the close.
        // That is a peer that left, not one that failed.
        if (error && (error.sctpCauseCode === 12 || /User-Initiated Abort/.test(error.message || ""))) {
          closure.set({ code: 1000, reason: "the other side closed" });
          state.set("closed");
          return;
        }
        fail(error || new Error("The data channel failed"));
      };
    };

    if (typeof RTCPeerConnection === "undefined") {
      fail(new Error("WebRTC is not available in this browser"));
    } else {
      pc = new RTCPeerConnection({ iceServers: opts.ice || [] });
      pc.onicecandidate = (event) => {
        if (event.candidate) {
          const c = event.candidate;
          tell({ type: "candidate", candidate: typeof c.toJSON === "function" ? c.toJSON() : c });
        }
      };
      pc.onconnectionstatechange = () => {
        if (pc.connectionState === "failed") fail(new Error("The peer could not be reached"));
      };
      if (opts.initiator) {
        attach(pc.createDataChannel("wf", { ordered: opts.ordered !== false }));
        pc.createOffer()
          .then((offer) => pc.setLocalDescription(offer))
          .then(() => tell({ type: "offer", sdp: pc.localDescription.sdp }))
          .catch(fail);
      } else {
        pc.ondatachannel = (event) => attach(event.channel);
      }
    }

    const settle = async () => {
      while (early.length) await pc.addIceCandidate(early.shift());
    };
    /// What the other side sent through the signalling: its offer, its
    /// answer, or a route it can be reached by.
    async function receive(message) {
      if (!pc || !message || typeof message !== "object") return;
      try {
        if (message.type === "offer") {
          await pc.setRemoteDescription({ type: "offer", sdp: message.sdp });
          await settle();
          await pc.setLocalDescription(await pc.createAnswer());
          tell({ type: "answer", sdp: pc.localDescription.sdp });
        } else if (message.type === "answer") {
          await pc.setRemoteDescription({ type: "answer", sdp: message.sdp });
          await settle();
        } else if (message.type === "candidate" && message.candidate) {
          if (pc.remoteDescription) await pc.addIceCandidate(message.candidate);
          else early.push(message.candidate);
        }
      } catch (e) {
        fail(e);
      }
    }

    const handle = {
      state,
      messages,
      closure,
      error: failure,
      signal: receive,
      /// Send a value; one sent before the channel opens waits for it.
      send(value) {
        const text = typeof value === "string" ? value : JSON.stringify(value);
        if (channel && channel.readyState === "open") channel.send(text);
        else waiting.push(text);
      },
      last(kind) {
        const held = messages();
        for (let i = held.length - 1; i >= 0; i--) {
          if (kind == null || (held[i] && held[i].type === kind)) return held[i];
        }
        return null;
      },
      close() {
        if (typeof window !== "undefined" && window.removeEventListener) {
          window.removeEventListener("pagehide", goodbye);
        }
        if (channel) channel.close();
        if (pc) pc.close();
        if (state() !== "error") state.set("closed");
      },
    };
    // A page that leaves says so, and the other side sees its peer close at
    // once — without it, the other side only learns when the connection
    // times out, half a minute later.
    function goodbye() { handle.close(); }
    if (typeof window !== "undefined" && window.addEventListener) {
      window.addEventListener("pagehide", goodbye);
    }
    onCleanup(() => handle.close());
    return handle;
  }
