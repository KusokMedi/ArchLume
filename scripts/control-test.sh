#!/usr/bin/env bash
# Runtime test for the compositor control socket (see compositor/src/control.rs).
#
# Starts the compositor nested on the host session, then drives it over the
# control socket and checks the responses. This exercises real compositor state
# (workspace switching) without needing to inject input events. A second phase
# launches a real client so window counting, focus-on-open and moving a window
# to another workspace are verified end to end.
#
# Development-only; uses a private XDG_RUNTIME_DIR and never touches the host
# session beyond a nested window.
set -u

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${ARCHLUME_COMPOSITOR:-$ROOT/target/debug/archlume-compositor}"
CLIENT="${ARCHLUME_CONTROL_CLIENT:-foot}"
PAT='archlume-composito[r]'

HOST_RT="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
HOST_DISPLAY="${WAYLAND_DISPLAY:-wayland-0}"
HOST_SOCK="$HOST_RT/$HOST_DISPLAY"

LOG="${TMPDIR:-/tmp}/archlume-control-compositor.log"
CLIENT_LOG="${TMPDIR:-/tmp}/archlume-control-client.log"
RT="$(mktemp -d "${TMPDIR:-/tmp}/archlume-control.XXXXXX")"
SOCK="$RT/archlume-control.sock"

cleanup() {
  pkill -f "$PAT" 2>/dev/null || true
  pkill -x "$CLIENT" 2>/dev/null || true
  # The clients are launched as `$CLIENT sh -c 'sleep 30'`; kill the detached
  # sleep children too so the script (and its caller) can exit promptly.
  pkill -f 'sleep 3[0]' 2>/dev/null || true
  rm -rf "$RT"
}
trap cleanup EXIT

fail() {
  echo "FAIL: $*; see $LOG" >&2
  exit 1
}

if [ ! -x "$BIN" ]; then
  echo "compositor binary not found: $BIN" >&2
  echo "build it first: cargo build -p archlume-compositor" >&2
  exit 1
fi
[ -S "$HOST_SOCK" ] || fail "no host Wayland socket at $HOST_SOCK"
command -v socat >/dev/null 2>&1 || fail "socat is required"
command -v "$CLIENT" >/dev/null 2>&1 || fail "control client '$CLIENT' not found"

pkill -f "$PAT" 2>/dev/null || true
sleep 1

# `-c true` keeps the compositor from auto-launching a terminal window, so the
# window count is deterministic.
setsid --fork env XDG_RUNTIME_DIR="$RT" WAYLAND_DISPLAY="$HOST_SOCK" RUST_LOG=warn \
  "$BIN" -c true < /dev/null > "$LOG" 2>&1

for _ in $(seq 1 20); do
  [ -S "$SOCK" ] && break
  sleep 0.25
done
[ -S "$SOCK" ] || fail "control socket was not created at $SOCK"
pgrep -f "$PAT" >/dev/null || fail "compositor did not stay running"

ctl() {
  printf '%s\n' "$1" | socat -t 2 - "UNIX-CONNECT:$SOCK" 2>/dev/null
}

check() {
  local command="$1" expected="$2" got
  got="$(ctl "$command")"
  if [ "$got" != "$expected" ]; then
    fail "command '$command' -> '$got' (expected '$expected')"
  fi
  echo "ok: $command -> $got"
}

# Phase 1: pure state, no clients.
check "ping" "ok pong"
check "workspace" "ok 1"
check "switch 5" "ok 5 changed"
check "workspace" "ok 5"
check "switch 5" "ok 5 same"
check "windows" "ok 0"
# Out-of-range numbers clamp to the last workspace (number 10).
check "switch 99" "ok 10 changed"
check "workspace" "ok 10"
check "bogus" "err unknown command"

# Phase 2: a real client.
check "switch 1" "ok 1 changed"

sock_name=""
for _ in $(seq 1 20); do
  sock_name=$(ls "$RT" 2>/dev/null | grep -E '^wayland-[0-9]+$' | head -1)
  [ -n "$sock_name" ] && break
  sleep 0.25
done
[ -n "$sock_name" ] || fail "compositor did not create a Wayland socket"

XDG_RUNTIME_DIR="$RT" WAYLAND_DISPLAY="$sock_name" \
  setsid --fork "$CLIENT" sh -c 'sleep 30' < /dev/null > "$CLIENT_LOG" 2>&1

# Wait for the window to be mapped (count becomes 1).
for _ in $(seq 1 40); do
  [ "$(ctl windows)" = "ok 1" ] && break
  sleep 0.25
done
check "windows" "ok 1"

# New windows get keyboard focus, so the focused window can be moved.
check "move 2" "ok 2"
check "workspace" "ok 2"
check "windows" "ok 1"
check "switch 1" "ok 1 changed"
check "switch 2" "ok 2 changed"

# Phase 3: cascade placement of two fresh clients on one workspace.
pkill -x "$CLIENT" 2>/dev/null || true
for _ in $(seq 1 20); do
  [ "$(ctl windows)" = "ok 0" ] && break
  sleep 0.25
done
check "windows" "ok 0"
check "switch 1" "ok 1 changed"

launch_client() {
  XDG_RUNTIME_DIR="$RT" WAYLAND_DISPLAY="$sock_name" \
    setsid --fork "$CLIENT" sh -c 'sleep 30' < /dev/null >> "$CLIENT_LOG" 2>&1
}
wait_for_windows() {
  for _ in $(seq 1 40); do
    [ "$(ctl windows)" = "ok $1" ] && return 0
    sleep 0.25
  done
  fail "expected $1 window(s)"
}

launch_client
wait_for_windows 1
launch_client
wait_for_windows 2

check "window 99" "err no such window"

# The two windows must be cascaded by CASCADE_STEP (32) in x and y.
read -r _ x0 y0 _ _ <<< "$(ctl 'window 0')"
read -r _ x1 y1 _ _ <<< "$(ctl 'window 1')"
if [ "$((x1 - x0))" -ne 32 ] || [ "$((y1 - y0))" -ne 32 ]; then
  fail "cascade offset wrong: window0=($x0,$y0) window1=($x1,$y1)"
fi
echo "ok: cascade offset window0=($x0,$y0) window1=($x1,$y1)"

# Phase 4: ask a window to close.
check "close 99" "err no such window"
check "close 1" "ok"
for _ in $(seq 1 40); do
  [ "$(ctl windows)" = "ok 1" ] && break
  sleep 0.25
done
check "windows" "ok 1"

# Phase 5: the render loop reports frame-time statistics.
stats="$(ctl stats)"
if ! printf '%s' "$stats" | grep -Eq '^ok frames [0-9]+ mean [0-9]+ max [0-9]+$'; then
  fail "stats -> '$stats'"
fi
frames="$(printf '%s' "$stats" | sed -E 's/^ok frames ([0-9]+).*/\1/')"
[ "$frames" -gt 0 ] || fail "stats reported no frames: '$stats'"
echo "ok: stats -> $stats"

# Phase 6: gestures reuse the workspace action system. Arguments are the normalized
# swipe delta; the direction and threshold logic lives in compositor/src/gestures.rs.
check "switch 5" "ok 5 changed"
check "gesture 3 -0.5 0" "ok prev 4"
check "gesture 3 0.5 0" "ok next 5"
check "gesture 3 0.5 0" "ok next 6"
check "gesture 3 0 0.5" "ok none"
check "gesture 3 0.02 0.01" "ok none"
check "gesture 4 0 -0.4" "ok none"

echo "PASS: control socket works; workspace, count, focus-on-open, move, cascade, close, frame stats and gestures verified"
