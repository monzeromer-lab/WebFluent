#!/usr/bin/env python3
"""Time a language server's answers on one file of a project, as an editor
asks them: open, completion, hover, definition, outline, semantic tokens,
formatting, and ten edits in a row.

Usage: scripts/bench/lsp.py PATH/TO/wf-lsp PROJECT_DIR FILE_IN_PROJECT
"""
import json, subprocess, sys, time, os, threading, queue
LSP, root, rel = sys.argv[1], os.path.abspath(sys.argv[2]), sys.argv[3]
path = os.path.join(root, rel); text = open(path).read()
p = subprocess.Popen([LSP], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
q = queue.Queue()
def reader():
    while True:
        h = {}
        while True:
            line = p.stdout.readline()
            if not line: return
            line = line.decode().strip()
            if not line: break
            k, v = line.split(":", 1); h[k.lower()] = v.strip()
        body = p.stdout.read(int(h["content-length"]))
        q.put((time.perf_counter(), json.loads(body)))
threading.Thread(target=reader, daemon=True).start()
nid = [0]
def send(msg):
    b = json.dumps(msg).encode(); p.stdin.write(b"Content-Length: %d\r\n\r\n" % len(b) + b); p.stdin.flush()
def req(method, params):
    nid[0] += 1; i = nid[0]; t = time.perf_counter(); send({"jsonrpc": "2.0", "id": i, "method": method, "params": params})
    while True:
        at, m = q.get(timeout=120)
        if m.get("id") == i: return (at - t) * 1000, m
def note(method, params): send({"jsonrpc": "2.0", "method": method, "params": params})
def wait_diag(uri, t):
    while True:
        at, m = q.get(timeout=120)
        if m.get("method") == "textDocument/publishDiagnostics" and m["params"]["uri"] == uri:
            return (at - t) * 1000, len(m["params"]["diagnostics"])
uri = "file://" + path
ms, _ = req("initialize", {"processId": None, "rootUri": "file://" + root, "capabilities": {"textDocument": {"completion": {"completionItem": {"snippetSupport": True}}}}, "workspaceFolders": [{"uri": "file://" + root, "name": "p"}]})
print(f"initialize            {ms:8.1f} ms")
note("initialized", {})
t = time.perf_counter(); note("textDocument/didOpen", {"textDocument": {"uri": uri, "languageId": "webfluent", "version": 1, "text": text}})
ms, n = wait_diag(uri, t); print(f"open → diagnostics    {ms:8.1f} ms  ({n} findings, {len(text.splitlines())} lines)")
lines = text.splitlines()
# positions: a line inside the body, mid-file
mid = len(lines) // 2
def pos(l):
    l = min(l, len(lines) - 1)
    return {"line": l, "character": max(0, len(lines[l]) // 2)}
for name, method in [("completion", "textDocument/completion"), ("hover", "textDocument/hover"), ("definition", "textDocument/definition"), ("documentSymbol", "textDocument/documentSymbol"), ("semanticTokens", "textDocument/semanticTokens/full"), ("formatting", "textDocument/formatting")]:
    times = []
    for k in range(5):
        params = {"textDocument": {"uri": uri}}
        if method in ("textDocument/completion", "textDocument/hover", "textDocument/definition"): params["position"] = pos(mid + k)
        if method == "textDocument/formatting": params["options"] = {"tabSize": 4, "insertSpaces": True}
        ms, m = req(method, params); times.append(ms)
    times.sort(); print(f"{name:20}  {times[len(times)//2]:8.1f} ms median  (max {times[-1]:.1f})")
# typing: 10 edits, measure each edit → diagnostics
v = 1; times = []
for k in range(10):
    v += 1
    lines[mid] = lines[mid] + " "
    t = time.perf_counter(); note("textDocument/didChange", {"textDocument": {"uri": uri, "version": v}, "contentChanges": [{"text": "\n".join(lines)}]})
    ms, n = wait_diag(uri, t); times.append(ms)
times.sort(); print(f"edit → diagnostics    {times[len(times)//2]:8.1f} ms median  (max {times[-1]:.1f})")
times = []
for k in range(5):
    ms, m = req("textDocument/completion", {"textDocument": {"uri": uri}, "position": pos(mid)}); times.append(ms)
times.sort(); print(f"completion after edit {times[len(times)//2]:8.1f} ms median")
req("shutdown", None); note("exit", None)
