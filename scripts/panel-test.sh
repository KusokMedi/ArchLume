#!/usr/bin/env bash
# Runtime test for the ArchLume panel (Milestone 5 first slice).
#
# Starts the compositor nested on a Hyprland host session, launches the
# archlume-shell panel client against it, checks over the control socket that a
# layer surface is mapped, and verifies with real pixels (grim + ImageMagick)
# that a bar of the expected colour appears across the top of the output while
# the area below stays the background colour.
#
# Development-only: needs a Hyprland host session plus `grim` and `magick`.
set -u

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${ARCHLUME_COMPOSITOR:-$ROOT/target/debug/archlume-compositor}"
SHELL_BIN="${ARCHLUME_SHELL:-$ROOT/target/debug/archlume-shell}"
PAT='archlume-composito[r]'
SHELL_PAT='archlume-shel[l]'

HOST_RT="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
HOST_DISPLAY="${WAYLAND_DISPLAY:-wayland-0}"
HOST_SOCK="$HOST_RT/$HOST_DISPLAY"

OUT="${TMPDIR:-/tmp}/archlume-panel"
LOG="$OUT/compositor.log"
SHELL_LOG="$OUT/shell.log"
RT="$(mktemp -d "${TMPDIR:-/tmp}/archlume-panel.XXXXXX")"
SOCK="$RT/archlume-control.sock"

cleanup() {
  pkill -f "$PAT" 2>/dev/null || true
  pkill -f "$SHELL_PAT" 2>/dev/null || true
  rm -rf "$RT"
}
trap cleanup EXIT

fail() {
  echo "FAIL: $*; see $LOG and $SHELL_LOG" >&2
  exit 1
}

[ -x "$BIN" ] || { echo "compositor binary not found: $BIN" >&2; echo "build it first: cargo build -p archlume-compositor" >&2; exit 1; }
[ -x "$SHELL_BIN" ] || { echo "shell binary not found: $SHELL_BIN" >&2; echo "build it first: cargo build -p archlume-shell" >&2; exit 1; }
[ -S "$HOST_SOCK" ] || fail "no host Wayland socket at $HOST_SOCK"
command -v socat >/dev/null 2>&1 || fail "socat is required"
command -v grim >/dev/null 2>&1 || fail "grim is required"
command -v magick >/dev/null 2>&1 || fail "ImageMagick (magick) is required"
command -v hyprctl >/dev/null 2>&1 || fail "hyprctl (Hyprland host) is required"

mkdir -p "$OUT"
pkill -f "$PAT" 2>/dev/null || true
pkill -f "$SHELL_PAT" 2>/dev/null || true
sleep 0.5

setsid --fork env XDG_RUNTIME_DIR="$RT" WAYLAND_DISPLAY="$HOST_SOCK" RUST_LOG=warn \
  "$BIN" -c true < /dev/null > "$LOG" 2>&1

for _ in $(seq 1 20); do
  [ -S "$SOCK" ] && break
  sleep 0.25
done
[ -S "$SOCK" ] || fail "control socket was not created at $SOCK"

ctl() { printf '%s\n' "$1" | socat -t 1 - "UNIX-CONNECT:$SOCK" 2>/dev/null; }

sock_name=""
for _ in $(seq 1 20); do
  sock_name=$(ls "$RT" 2>/dev/null | grep -E '^wayland-[0-9]+$' | head -1)
  [ -n "$sock_name" ] && break
  sleep 0.25
done
[ -n "$sock_name" ] || fail "compositor did not create a Wayland socket"

# Launch the panel against the nested compositor.
XDG_RUNTIME_DIR="$RT" WAYLAND_DISPLAY="$sock_name" \
  setsid --fork "$SHELL_BIN" < /dev/null > "$SHELL_LOG" 2>&1

for _ in $(seq 1 40); do
  [ "$(ctl layers)" = "ok 1" ] && break
  sleep 0.25
done
check_layers="$(ctl layers)"
[ "$check_layers" = "ok 1" ] || fail "panel layer surface not mapped: '$check_layers'"
echo "ok: layers -> $check_layers"

# Locate the nested compositor window and capture it.
geom="$(hyprctl clients -j 2>/dev/null | python3 -c '
import json,sys
for c in json.load(sys.stdin):
    blob=(c.get("title","")+c.get("class","")+c.get("initialClass","")).lower()
    if "smithay" in blob:
        x,y=c.get("at",[0,0]); w,h=c.get("size",[0,0])
        print(x,y,w,h,c.get("address"))
        break
')"
read -r WX WY WW WH WADDR <<< "$geom"
[ -n "${WW:-}" ] && [ "${WW:-0}" -gt 0 ] || fail "nested compositor window not found"

hyprctl dispatch focuswindow "address:$WADDR" >/dev/null 2>&1 || true
sleep 0.5

REG="${WX},${WY} ${WW}x${WH}"
grim -g "$REG" "$OUT/panel.png" || fail "grim failed"

# A strip inside the bar and one well below it.
bar="$OUT/bar.png"
below="$OUT/below.png"
magick "$OUT/panel.png" -crop "${WW}x8+0+6" +repage "$bar"
magick "$OUT/panel.png" -crop "${WW}x8+0+60" +repage "$below"

# Force sRGB so a region ImageMagick stored as grayscale still reports r/g/b.
read -r br bg bb <<< "$(magick "$bar" -colorspace sRGB -format '%[fx:mean.r] %[fx:mean.g] %[fx:mean.b]' info:)"
read -r lr lg lb <<< "$(magick "$below" -colorspace sRGB -format '%[fx:mean.r] %[fx:mean.g] %[fx:mean.b]' info:)"

python3 - "$br" "$bg" "$bb" "$lr" "$lg" "$lb" <<'PY' || exit 1
import sys
br,bg,bb,lr,lg,lb = (float(a) for a in sys.argv[1:])
print(f"bar   mean r/g/b = {br:.3f}/{bg:.3f}/{bb:.3f}  (expected ~0.118/0.533/0.898)")
print(f"below mean r/g/b = {lr:.3f}/{lg:.3f}/{lb:.3f}  (expected ~0.098/0.098/0.098)")
bar_ok = bb > 0.7 and br < 0.3 and 0.3 < bg < 0.75
below_ok = abs(lr - 0.098) < 0.03 and abs(lg - 0.098) < 0.03 and abs(lb - 0.098) < 0.03
print("RESULT:", "PASS" if (bar_ok and below_ok) else "FAIL")
sys.exit(0 if (bar_ok and below_ok) else 1)
PY
rc=$?
[ "$rc" -eq 0 ] || fail "panel pixels not verified"

echo "PASS: archlume-shell mapped a top layer surface and drew the panel bar"
