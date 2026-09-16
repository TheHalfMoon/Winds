#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
binary="${1:-$repo_root/desktop/src-tauri/target/release/winds-desktop-host}"
out_dir="${2:-$repo_root/t143-evidence}"
mkdir -p "$out_dir"

export DISPLAY="${DISPLAY:-:99}"
export GDK_BACKEND=x11
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export JSC_useJIT=false

xvfb_log="$out_dir/xvfb.log"
http_log="$out_dir/http.log"
webdriver_log="$out_dir/webdriver.log"
Xvfb "$DISPLAY" -screen 0 1920x1080x24 -nolisten tcp >"$xvfb_log" 2>&1 &
xvfb_pid=$!
http_pid=""
webdriver_pid=""
cleanup() {
  [[ -z "$webdriver_pid" ]] || kill "$webdriver_pid" 2>/dev/null || true
  [[ -z "$http_pid" ]] || kill "$http_pid" 2>/dev/null || true
  kill "$xvfb_pid" 2>/dev/null || true
}
trap cleanup EXIT
sleep 1
kill -0 "$xvfb_pid"

python3 "$repo_root/desktop/tests/performance/t143_native.py" \
  --binary "$binary" \
  --output "$out_dir/native.json" \
  --launches 20 \
  --idle-seconds 60

python3 -m http.server 4173 --bind 127.0.0.1 --directory "$repo_root/desktop/dist" >"$http_log" 2>&1 &
http_pid=$!
WebKitWebDriver --port=4444 >"$webdriver_log" 2>&1 &
webdriver_pid=$!

python3 - <<'PY'
import socket, time
for port in (4173, 4444):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.5):
                break
        except OSError:
            time.sleep(0.1)
    else:
        raise SystemExit(f"service on port {port} did not become ready")
PY

export WINDS_T143_WEBKIT_BINARY="$(command -v epiphany)"
export WINDS_T143_WEBKIT_BROWSER_NAME=Epiphany
export WINDS_T143_WEBKIT_ARGUMENT=--automation-mode
python3 "$repo_root/desktop/tests/performance/t143_webkit.py" \
  --base-url http://127.0.0.1:4173/ \
  --webdriver http://127.0.0.1:4444 \
  --output "$out_dir/renderer.json"
