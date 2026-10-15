#!/usr/bin/env bash
# Boot the newest built ISO in QEMU (UEFI) with a disposable virtual disk and
# verify the ArchLume live environment reaches a shell.
#
# Evidence is captured by dumping the emulated framebuffer through the QEMU
# monitor (`screendump`) and OCR'ing it with tesseract. The host's disks are
# never passed through; the virtual disk is created fresh and deleted after.
#
# Environment overrides:
#   ISO_FILE   path to the ISO (default: newest in iso/out/)
#   BOOT_WAIT  seconds to wait before capturing (default: 180)
#   MARKER     extended regex that must match the OCR output (default: root@)
set -u

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ISO_FILE="${ISO_FILE:-$(ls -t "$ROOT"/iso/out/*.iso 2>/dev/null | head -1)}"
BOOT_WAIT="${BOOT_WAIT:-180}"
# OCR is imperfect, so the default marker matches the root shell prompt rather
# than the brand spelling (which OCR often misreads).
MARKER="${MARKER:-root@}"

TMP="${TMPDIR:-/tmp}"
DISK="$TMP/archlume-boot-test.qcow2"
VARS="$TMP/archlume-OVMF_VARS.fd"
SHOT="$TMP/archlume-boot.ppm"
PNG="$TMP/archlume-boot.png"
LOG="$TMP/archlume-boot-qemu.log"
CODE=/usr/share/edk2/x64/OVMF_CODE.4m.fd

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

[ -n "$ISO_FILE" ] && [ -f "$ISO_FILE" ] || fail "no ISO found; run 'just iso' first"
command -v qemu-system-x86_64 >/dev/null || fail "qemu-system-x86_64 not found"
command -v qemu-img >/dev/null || fail "qemu-img not found"
[ -f "$CODE" ] || fail "OVMF firmware not found (install edk2-ovmf)"
command -v tesseract >/dev/null || fail "tesseract not found (needed for OCR verification)"

echo "iso:       $ISO_FILE"
echo "boot wait: ${BOOT_WAIT}s"
echo "marker:    $MARKER"

rm -f "$DISK" "$VARS" "$SHOT" "$PNG" "$LOG"
cp -f /usr/share/edk2/x64/OVMF_VARS.4m.fd "$VARS"
qemu-img create -f qcow2 "$DISK" 20G >/dev/null

# Feed monitor commands on stdin: wait, capture the framebuffer, then quit.
{
  sleep "$BOOT_WAIT"
  echo "screendump $SHOT"
  sleep 5
  echo "quit"
} | timeout "$((BOOT_WAIT + 60))" qemu-system-x86_64 \
  -enable-kvm -m 2048 -cpu host -smp 2 \
  -drive if=pflash,format=raw,readonly=on,file="$CODE" \
  -drive if=pflash,format=raw,file="$VARS" \
  -drive file="$ISO_FILE",media=cdrom,readonly=on \
  -drive file="$DISK",if=virtio,format=qcow2 \
  -device virtio-vga \
  -display none \
  -monitor stdio \
  -netdev user,id=n0 -device virtio-net-pci,netdev=n0 \
  -boot order=d \
  >"$LOG" 2>&1 || true

[ -s "$SHOT" ] || fail "no framebuffer captured (see $LOG)"

if command -v convert >/dev/null; then
  convert "$SHOT" "$PNG" 2>/dev/null || cp -f "$SHOT" "$PNG"
else
  cp -f "$SHOT" "$PNG"
fi

OCR="$(tesseract "$SHOT" - 2>/dev/null || true)"
echo "--- OCR of boot framebuffer ---"
printf '%s\n' "$OCR"
echo "-------------------------------"

if printf '%s' "$OCR" | grep -Eqi "$MARKER"; then
  echo "PASS: live environment booted to a shell (matched /$MARKER/) ($PNG)"
  exit 0
fi

echo "FAIL: marker '$MARKER' not found in boot framebuffer ($PNG)" >&2
exit 1
