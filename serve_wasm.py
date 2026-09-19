#!/usr/bin/env python3
"""Serve a makepad wasm build, with the headers it needs.

    python3 serve_wasm.py [port] [app]        # defaults: 8080 gallery

`app` is the crate name, which is also the directory `cargo makepad wasm build -p <app>`
writes into: `target/makepad-wasm-app/release/<app>`.

## Why this is a script and not `python3 -m http.server`

Two headers, and neither is optional in the general case:

- `Cross-Origin-Opener-Policy: same-origin` and
  `Cross-Origin-Embedder-Policy: require-corp` are what make a document
  **cross-origin isolated**, which is the only state in which `SharedArrayBuffer` exists. A
  build made with `--threads` allocates its memory as a `SharedArrayBuffer` and will not
  start without them — and it fails by hanging rather than by saying so, which is a bad
  afternoon. A build without `--threads` does not need them and is unharmed by them, so they
  are always sent rather than being a flag to remember.
- `.wasm` and `.bin` need their right types: `application/wasm` and
  `application/octet-stream`. A server that guesses `text/plain` makes the browser refuse the
  module, and one that omits the type for `.bin` breaks `--split` builds.

## And no caching, which is the third thing

`Cache-Control: no-store` on everything. This is a development server: the whole point is to
rebuild and reload, and a cached `.wasm` from a previous build is the single most confusing
failure available here — the source says one thing, the browser runs another, and the fix
looks like it did nothing.
"""

import http.server
import json
import os
import socketserver
import sys
import urllib.parse
from pathlib import Path

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
APP = sys.argv[2] if len(sys.argv) > 2 else "gallery"

ROOT = Path(__file__).resolve().parent
# Both the plain and the `small`/`release` profiles, since `cargo makepad wasm build` writes to
# whichever the caller asked for and there is no way to tell from here which they meant.
CANDIDATES = [
    ROOT / "target" / "makepad-wasm-app" / "release" / APP,
    ROOT / "target" / "makepad-wasm-app" / "small" / APP,
    ROOT / "target" / "makepad-wasm-app" / APP,
]

# What a makepad wasm build emits, so a wrong directory is caught here rather than as a blank
# page: finding the directory is not the same as finding the app in it. The wasm is named with a
# content hash (`gallery.613ff5c5….wasm`), so the wasm check is a glob, not a literal.
def has_wasm(app_dir):
    return any(app_dir.glob(f"{APP}.*.wasm"))


def complete(app_dir):
    return (app_dir / "index.html").exists() and has_wasm(app_dir)

TYPES = {
    ".wasm": "application/wasm",
    ".js": "text/javascript",
    ".css": "text/css",
    ".html": "text/html",
    ".json": "application/json",
    ".bin": "application/octet-stream",
    ".ttf": "application/font-sfnt",
    ".otf": "application/font-sfnt",
    ".svg": "image/svg+xml",
    ".png": "image/png",
    ".jpg": "image/jpeg",
    ".jpeg": "image/jpeg",
    ".webp": "image/webp",
    ".wasm.secondary": "application/wasm",
}


class Handler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        # See the module doc: a cached wasm from a previous build is the most confusing failure this
        # server can produce.
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def do_POST(self):
        # makepad's crash reporter in `web.js` POSTs every browser-side error here. A static
        # handler answers 501 and the report — the one artifact that says *why* the page is a
        # blank "Loading.." — is dropped unread. This is a development server; its job is to make
        # failure readable, so the payload is decoded and printed instead.
        if self.path.split("?")[0] != "/api/crash":
            self.send_error(404, "only /api/crash accepts POST")
            return
        length = int(self.headers.get("Content-Length") or 0)
        body = self.rfile.read(length).decode("utf-8", "replace") if length else "(empty body)"
        print("\n=== crash report ===", flush=True)
        try:
            report = json.loads(body)
            for key, value in report.items():
                print(f"{key}: {value}", flush=True)
        except json.JSONDecodeError:
            print(body, flush=True)
        print("====================\n", flush=True)
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.send_header("Cross-Origin-Resource-Policy", "same-origin")
        self.end_headers()
        self.wfile.write(b"ok")

    def do_GET(self):
        # The inline fallback reporter in the generated `index.html` GETs this with the report
        # urlencoded into `data`; same reason to print it rather than 404 it.
        if self.path.startswith("/$report_error"):
            query = urllib.parse.urlparse(self.path).query
            params = urllib.parse.parse_qs(query)
            data = params.get("data", ["(no data)"])[0]
            print("\n=== crash report (inline fallback) ===", flush=True)
            print(urllib.parse.unquote(data), flush=True)
            print("======================================\n", flush=True)
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.end_headers()
            self.wfile.write(b"ok")
            return
        super().do_GET()

    def guess_type(self, path):
        text = str(path)
        for suffix, mime in TYPES.items():
            if text.endswith(suffix):
                return mime
        return super().guess_type(path)

    def log_message(self, fmt, *args):
        # Quieter than the default: a page load is dozens of requests and the useful lines are the
        # ones that are not 200s.
        code = args[1] if len(args) > 1 else "?"
        if str(code).startswith(("4", "5")):
            super().log_message(fmt, *args)


def main():
    app_dir = next((c for c in CANDIDATES if complete(c)), None)
    if app_dir is None:
        print(f"No wasm build of {APP!r} found. Looked in:", file=sys.stderr)
        for candidate in CANDIDATES:
            print(f"  {candidate.relative_to(ROOT)}", file=sys.stderr)
        print("\nBuild it first:\n", file=sys.stderr)
        print(f"  cargo makepad wasm install-toolchain        # once", file=sys.stderr)
        print(f"  cargo makepad wasm build -p {APP} --release\n", file=sys.stderr)
        return 1

    handler = lambda *args, **kwargs: Handler(*args, directory=str(app_dir), **kwargs)
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("", PORT), handler) as httpd:
        print(f"Serving {app_dir.relative_to(ROOT)} at http://localhost:{PORT}/")
        print("  \u00b7 ?page=<title|index> opens one page, the way GALLERY_PAGE does natively")
        print("  \u00b7 COOP/COEP sent, so a --threads build can allocate its SharedArrayBuffer")
        print("  \u00b7 Ctrl-C to stop")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
