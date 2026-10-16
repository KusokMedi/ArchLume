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
CLIENT="${ARCHLUME_PANEL_CLIENT:-foot}"
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
command -v "$CLIENT" >/dev/null 2>&1 || fail "panel test client '$CLIENT' not found"

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

ctl() { printf '%s\n' "$*" | socat -t 1 - "UNIX-CONNECT:$SOCK" 2>/dev/null; }

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
# Fraction of near-white ("text") pixels in a region, as a 0..1 mean after a threshold.
white_frac() { magick "$1" -crop "$2" +repage -colorspace Gray -threshold 80% -format '%[fx:mean]' info:; }

# Panel layout is fixed: PAD=10, 6*scale advance, scale=2 -> 96px for 8 glyphs.
label="96x16+10+7"
clock="96x16+$((WW - 10 - 96))+7"
label_w="$(white_frac "$OUT/panel.png" "$label")"
clock_w="$(white_frac "$OUT/panel.png" "$clock")"

python3 - "$br" "$bg" "$bb" "$lr" "$lg" "$lb" "$label_w" "$clock_w" <<'PY' || exit 1
import sys
br,bg,bb,lr,lg,lb,lw,cw = (float(a) for a in sys.argv[1:])
print(f"bar    mean r/g/b = {br:.3f}/{bg:.3f}/{bb:.3f}  (expected ~0.118/0.533/0.898)")
print(f"below  mean r/g/b = {lr:.3f}/{lg:.3f}/{lb:.3f}  (expected ~0.098/0.098/0.098)")
print(f"label  text frac  = {lw:.4f}   clock text frac = {cw:.4f}")
bar_ok = bb > 0.7 and br < 0.3 and 0.3 < bg < 0.75
below_ok = abs(lr - 0.098) < 0.03 and abs(lg - 0.098) < 0.03 and abs(lb - 0.098) < 0.03
text_ok = lw > 0.02 and cw > 0.02
print("RESULT:", "PASS" if (bar_ok and below_ok and text_ok) else "FAIL")
sys.exit(0 if (bar_ok and below_ok and text_ok) else 1)
PY
rc=$?
[ "$rc" -eq 0 ] || fail "panel background/text pixels not verified"

# The clock must actually tick: the seconds field changes every second.
sleep 1.5
grim -g "$REG" "$OUT/panel2.png" || fail "grim (second capture) failed"
magick "$OUT/panel.png" -crop "$clock" +repage "$OUT/clock1.png"
magick "$OUT/panel2.png" -crop "$clock" +repage "$OUT/clock2.png"
diff_px="$(magick compare -metric AE "$OUT/clock1.png" "$OUT/clock2.png" null: 2>&1 || true)"
diff_px="$(printf '%s' "$diff_px" | awk '{print $1}')"
awk -v d="${diff_px:-0}" 'BEGIN { exit !(d > 0) }' \
  || fail "clock did not change between captures (diff=$diff_px)"
echo "ok: clock ticking (clock-region pixel diff = $diff_px)"

# The brand label must stay fixed while the clock ticks, proving we render two
# distinct regions and only the clock redraws.
magick "$OUT/panel.png" -crop "$label" +repage "$OUT/label1.png"
magick "$OUT/panel2.png" -crop "$label" +repage "$OUT/label2.png"
label_diff="$(magick compare -metric AE "$OUT/label1.png" "$OUT/label2.png" null: 2>&1 || true)"
label_diff="$(printf '%s' "$label_diff" | awk '{print $1}')"
awk -v d="${label_diff:-0}" 'BEGIN { exit !(d == 0) }' \
  || fail "brand label changed between captures (diff=$label_diff)"
echo "ok: brand label stable while the clock ticks (diff=$label_diff)"

# Workspace indicator: PAD(10) + label(8 glyphs * 12) + WS_GAP(24) = x 130.
ws_region="96x16+130+7"
ws_w="$(white_frac "$OUT/panel.png" "$ws_region")"
awk -v f="$ws_w" 'BEGIN { exit !(f > 0.02) }' \
  || fail "workspace indicator text missing (white frac=$ws_w)"
magick "$OUT/panel.png" -crop "$ws_region" +repage "$OUT/ws1.png"

switch_out="$(ctl switch 4)"
sleep 1
ws_now="$(ctl workspace)"
[ "$ws_now" = "ok 4" ] || fail "compositor did not switch to workspace 4 (switch='$switch_out', workspace='$ws_now')"
grim -g "$REG" "$OUT/panel4.png" || fail "grim (workspace capture) failed"
magick "$OUT/panel4.png" -crop "$ws_region" +repage "$OUT/ws4.png"
ws_diff="$(magick compare -metric AE "$OUT/ws1.png" "$OUT/ws4.png" null: 2>&1 || true)"
ws_diff="$(printf '%s' "$ws_diff" | awk '{print $1}')"
awk -v d="${ws_diff:-0}" 'BEGIN { exit !(d > 0) }' \
  || fail "workspace indicator did not update after switch (diff=$ws_diff)"
echo "ok: workspace indicator updated to 4 (diff=$ws_diff)"

# Active-window title: TITLE_X(260) .. clock, fixed 200px probe region.
title_region="200x16+260+7"
title_empty="$(white_frac "$OUT/panel.png" "$title_region")"
awk -v f="$title_empty" 'BEGIN { exit !(f < 0.01) }' \
  || fail "title region should be empty with no client (frac=$title_empty)"

setsid --fork env XDG_RUNTIME_DIR="$RT" WAYLAND_DISPLAY="$sock_name" \
  "$CLIENT" -T ZZZ sh -c 'sleep 20' < /dev/null > "$OUT/client.log" 2>&1
for _ in $(seq 1 40); do
  [ "$(ctl windows)" = "ok 1" ] && break
  sleep 0.25
done
[ "$(ctl windows)" = "ok 1" ] || fail "panel test client did not map a window"
sleep 1
focused="$(ctl focused)"
[ "$focused" = "ok ZZZ" ] || fail "compositor focused-title mismatch: '$focused'"
grim -g "$REG" "$OUT/panel_title.png" || fail "grim (title capture) failed"
title_w="$(white_frac "$OUT/panel_title.png" "$title_region")"
awk -v f="$title_w" 'BEGIN { exit !(f > 0.02) }' \
  || fail "panel did not render the window title (frac=$title_w)"
echo "ok: panel shows focused title '$focused' (text frac=$title_w)"

# Launcher overlay: toggled by the compositor, rendered as a centred navy card.
[ "$(ctl launcher)" = "ok off" ] || fail "launcher should start off ($(ctl launcher))"
[ "$(ctl launcher on)" = "ok on" ] || fail "launcher on did not take effect"
for _ in $(seq 1 40); do
  [ "$(ctl layers)" = "ok 2" ] && break
  sleep 0.25
done
[ "$(ctl layers)" = "ok 2" ] || fail "launcher overlay did not map (layers=$(ctl layers))"
sleep 0.6
grim -g "$REG" "$OUT/panel_launcher.png" || fail "grim (launcher capture) failed"

# Overlay is 600x400 centred on a 664x720 output -> x 32..632, y 160..560.
bg_region="160x100+440+240"
bg_b="$(magick "$OUT/panel_launcher.png" -crop "$bg_region" +repage -colorspace sRGB -format '%[fx:mean.b]' info:)"
bg_r="$(magick "$OUT/panel_launcher.png" -crop "$bg_region" +repage -colorspace sRGB -format '%[fx:mean.r]' info:)"
app_frac="$(white_frac "$OUT/panel_launcher.png" '200x120+52+206')"
python3 - "$bg_b" "$bg_r" "$app_frac" <<'PY' || exit 1
import sys
b, r, f = (float(a) for a in sys.argv[1:])
print(f"launcher bg mean r/b = {r:.3f}/{b:.3f} (expected ~0.063/0.180), app-list text frac = {f:.4f}")
ok = b > 0.15 and r < 0.12 and f > 0.01
print("RESULT:", "PASS" if ok else "FAIL")
sys.exit(0 if ok else 1)
PY
rc=$?
[ "$rc" -eq 0 ] || fail "launcher overlay pixels not verified"

[ "$(ctl launcher off)" = "ok off" ] || fail "launcher off did not take effect"
for _ in $(seq 1 40); do
  [ "$(ctl layers)" = "ok 1" ] && break
  sleep 0.25
done
[ "$(ctl layers)" = "ok 1" ] || fail "launcher overlay did not unmap (layers=$(ctl layers))"
echo "ok: launcher overlay mapped, rendered an app list, and unmapped"

echo "PASS: archlume-shell panel maps a layer surface, shows a ticking clock, a live workspace indicator and the active-window title; launcher overlay renders installed apps"
