#!/usr/bin/env bash
# Runtime test for the snap-preview overlay (compositor/src/wm.rs, winit.rs).
#
# Starts the compositor nested on a Hyprland host session, asks it over the
# control socket to show the left-half snap preview, and verifies with real
# pixels (grim + ImageMagick) that the overlay is drawn on the left half of the
# window while the right half is untouched.
#
# Development-only: needs a Hyprland host session plus `grim` and `magick`. It
# uses a private XDG_RUNTIME_DIR and only opens a nested window.
set -u

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${ARCHLUME_COMPOSITOR:-$ROOT/target/debug/archlume-compositor}"
PAT='archlume-composito[r]'

HOST_RT="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
HOST_DISPLAY="${WAYLAND_DISPLAY:-wayland-0}"
HOST_SOCK="$HOST_RT/$HOST_DISPLAY"

OUT="${TMPDIR:-/tmp}/archlume-preview"
LOG="$OUT/compositor.log"
RT="$(mktemp -d "${TMPDIR:-/tmp}/archlume-preview.XXXXXX")"
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

[ -x "$BIN" ] || { echo "compositor binary not found: $BIN" >&2; echo "build it first: cargo build -p archlume-compositor" >&2; exit 1; }
[ -S "$HOST_SOCK" ] || fail "no host Wayland socket at $HOST_SOCK"
command -v socat >/dev/null 2>&1 || fail "socat is required"
command -v grim >/dev/null 2>&1 || fail "grim is required"
command -v magick >/dev/null 2>&1 || fail "ImageMagick (magick) is required"
command -v hyprctl >/dev/null 2>&1 || fail "hyprctl (Hyprland host) is required"

mkdir -p "$OUT"
pkill -f "$PAT" 2>/dev/null || true
sleep 0.5

setsid --fork env XDG_RUNTIME_DIR="$RT" WAYLAND_DISPLAY="$HOST_SOCK" RUST_LOG=warn \
  "$BIN" -c true < /dev/null > "$LOG" 2>&1

for _ in $(seq 1 20); do
  [ -S "$SOCK" ] && break
  sleep 0.25
done
[ -S "$SOCK" ] || fail "control socket was not created at $SOCK"

ctl() { printf '%s\n' "$1" | socat -t 1 - "UNIX-CONNECT:$SOCK" 2>/dev/null; }

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

# Bring the window to the front so grim captures it unobstructed.
hyprctl dispatch focuswindow "address:$WADDR" >/dev/null 2>&1 || true
sleep 0.4

REG="${WX},${WY} ${WW}x${WH}"
mean() { magick "$OUT/$1" -crop "$2" +repage -colorspace Gray -format "%[fx:mean]" info:; }

# Control response advertises the left-half geometry.
resp="$(ctl 'preview left')"
read -r _ px py pw ph <<< "$resp"
[ "$px" = "0" ] && [ "$py" = "0" ] || fail "preview left -> '$resp'"
[ "$pw" -eq "$((WW / 2))" ] || fail "preview width ${pw:-?} != window half $((WW / 2))"
[ "$ph" -eq "$WH" ] || fail "preview height ${ph:-?} != window height $WH"

# Clear -> capture, show left -> capture, clear -> capture. Each change is
# followed by a short pause so the compositor has redrawn before grim captures.
ctl 'preview off' >/dev/null
sleep 0.3
grim -g "$REG" "$OUT/off.png" || fail "grim (off) failed"
ctl 'preview left' >/dev/null
sleep 0.3
grim -g "$REG" "$OUT/left.png" || fail "grim (left) failed"
ctl 'preview off' >/dev/null
sleep 0.3
grim -g "$REG" "$OUT/off2.png" || fail "grim (off2) failed"

half=$((WW / 2))
off_l=$(mean off.png "${half}x${WH}+0+0")
left_l=$(mean left.png "${half}x${WH}+0+0")
off_r=$(mean off.png "${half}x${WH}+${half}+0")
left_r=$(mean left.png "${half}x${WH}+${half}+0")
same=$(magick compare -metric AE "$OUT/off.png" "$OUT/off2.png" null: 2>&1 || true)
same="${same%% *}"

python3 - "$off_l" "$left_l" "$off_r" "$left_r" "$same" <<'PY' || exit 1
import sys
off_l, left_l, off_r, left_r, same = (float(a) for a in sys.argv[1:])
print(f"left-half mean: off={off_l:.4f} left={left_l:.4f} (brighter={left_l-off_l:+.4f})")
print(f"right-half mean: off={off_r:.4f} left={left_r:.4f} (delta={abs(left_r-off_r):.4f})")
print(f"pixels differing between off and off2: {int(same)}")
ok = (left_l - off_l) > 0.02 and abs(left_r - off_r) < 0.01 and int(same) == 0
print("RESULT:", "PASS" if ok else "FAIL")
sys.exit(0 if ok else 1)
PY
rc=$?
[ "$rc" -eq 0 ] || fail "snap preview overlay not verified"

echo "PASS: snap preview overlay drawn on the left half and cleared again"
