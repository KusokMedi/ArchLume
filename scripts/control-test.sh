#!/usr/bin/env bash
# Runtime test for the compositor control socket (see compositor/src/control.rs).
#
# Starts the compositor nested on the host session, then drives it over the
# control socket and checks the responses. This exercises real compositor state
# (workspace switching) without needing to inject input events.
#
# Development-only; uses a private XDG_RUNTIME_DIR and never touches the host
# session beyond a nested window.
set -u

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${ARCHLUME_COMPOSITOR:-$ROOT/target/debug/archlume-compositor}"
PAT='archlume-composito[r]'

HOST_RT="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
HOST_DISPLAY="${WAYLAND_DISPLAY:-wayland-0}"
HOST_SOCK="$HOST_RT/$HOST_DISPLAY"

LOG="${TMPDIR:-/tmp}/archlume-control-compositor.log"
RT="$(mktemp -d "${TMPDIR:-/tmp}/archlume-control.XXXXXX")"
SOCK="$RT/archlume-control.sock"

cleanup() {
  pkill -f "$PAT" 2>/dev/null || true
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

echo "PASS: control socket works and workspace state is consistent"
