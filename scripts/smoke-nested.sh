#!/usr/bin/env bash
# Nested smoke test for the ArchLume compositor.
#
# Verifies that the compositor:
#   1. starts on the host Wayland session (winit backend) and stays alive;
#   2. creates its own Wayland listening socket;
#   3. accepts a real client, which maps a window and exits cleanly.
#
# This is a development-only check. It never takes over the host session: the
# compositor runs nested, with a private XDG_RUNTIME_DIR, and connects to the
# host compositor via an absolute socket path.
#
# Override the client with ARCHLUME_SMOKE_CLIENT (default: foot).
set -u

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${ARCHLUME_COMPOSITOR:-$ROOT/target/debug/archlume-compositor}"
CLIENT="${ARCHLUME_SMOKE_CLIENT:-foot}"
# The bracket trick keeps this pattern from matching this script's own cmdline.
PAT='archlume-composito[r]'

HOST_RT="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
HOST_DISPLAY="${WAYLAND_DISPLAY:-wayland-0}"
HOST_SOCK="$HOST_RT/$HOST_DISPLAY"

LOG="${TMPDIR:-/tmp}/archlume-smoke-compositor.log"
CLIENT_LOG="${TMPDIR:-/tmp}/archlume-smoke-client.log"
RT="$(mktemp -d "${TMPDIR:-/tmp}/archlume-smoke.XXXXXX")"

cleanup() {
  pkill -f "$PAT" 2>/dev/null || true
  rm -rf "$RT"
}
trap cleanup EXIT

if [ ! -x "$BIN" ]; then
  echo "compositor binary not found: $BIN" >&2
  echo "build it first: cargo build -p archlume-compositor" >&2
  exit 1
fi

if [ ! -S "$HOST_SOCK" ]; then
  echo "no host Wayland socket at $HOST_SOCK (run this from a Wayland session)" >&2
  exit 1
fi

command -v "$CLIENT" >/dev/null 2>&1 || {
  echo "smoke client '$CLIENT' not found in PATH" >&2
  exit 1
}

pkill -f "$PAT" 2>/dev/null || true
sleep 1

# Connect to the host session via an absolute path, but let the compositor
# create its own socket inside the private runtime dir.
setsid --fork env XDG_RUNTIME_DIR="$RT" WAYLAND_DISPLAY="$HOST_SOCK" RUST_LOG=debug \
  "$BIN" < /dev/null > "$LOG" 2>&1

sock_name=""
for _ in $(seq 1 20); do
  sock_name=$(ls "$RT" 2>/dev/null | grep -E '^wayland-[0-9]+$' | head -1)
  [ -n "$sock_name" ] && break
  sleep 0.25
done

pid=$(pgrep -f "$PAT" | head -1)
if [ -z "$pid" ]; then
  echo "FAIL: compositor did not stay running; see $LOG" >&2
  exit 1
fi

if [ -z "$sock_name" ]; then
  echo "FAIL: compositor did not create a Wayland socket in $RT; see $LOG" >&2
  exit 1
fi

XDG_RUNTIME_DIR="$RT" WAYLAND_DISPLAY="$sock_name" timeout 8 "$CLIENT" sh -c 'sleep 2' \
  > "$CLIENT_LOG" 2>&1
rc=$?

if [ "$rc" -eq 0 ]; then
  echo "PASS: compositor alive (pid $pid), socket $sock_name, client '$CLIENT' mapped and exited"
  exit 0
fi

echo "FAIL: client '$CLIENT' exited rc=$rc; see $CLIENT_LOG" >&2
exit 1
