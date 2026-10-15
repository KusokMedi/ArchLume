# Native DRM/libseat backend — plan

Status: **planned, not started.** The compositor currently runs only on the
winit (nested) backend. This document records the plan so the work can be done
safely and verified, per AGENTS sections 3 and 6.1.

## Why

The winit backend renders inside a parent Wayland session. ArchLume must be able
to own the display itself so the live ISO can boot straight into the ArchLume
session (M4 deliverable: "session startup, outputs, input, surfaces ... safe
fallback"). That requires a DRM/KMS backend driven through a session provider
(libseat), reading input from libinput.

## Dependency and feature surface

Smithay features to enable (only under the `native` cargo feature, so the
default build and its tests are unaffected):

- `backend_drm`, `backend_gbm`, `backend_udev`, `backend_libinput`,
  `backend_session_libseat`
- `renderer_gl` (already implied by the winit path)
- `desktop` / `wayland_frontend` (already used)

Crates: `libseat`, `udev`, `input` (libinput), `drm`, `gbm`, `nix`/`libc`.
All corresponding system libraries are present on the reference host
(libseat 0.9.3, libinput 1.31.3, libdrm 2.4.134, gbm, libudev).

## Module layout

- `session.rs` — open a libseat session with automatic backend detection
  (logind when running under systemd, direct otherwise).
- `udev.rs` — for each DRM device:
  - `DrmDevice` + `GbmBufferedSurface` per connector; `DrmOutputManager` for
    mode-setting and frame feedback.
  - map each connector to a `Space` output (`Space::map_output`), reusing the
    existing workspace offset logic per output.
  - hotplug: `UdevBackend` as a calloop source; add/remove outputs on events.
- `input` — a libinput source feeding the existing
  `ArchLume::process_input_event` (already generic over `InputBackend`), so the
  keyboard/pointer/workspace logic is unchanged.
- `cursor` — software cursor rendering (RGBA cursor buffer drawn as a custom
  render element into the output damage); later a hardware cursor plane.

## Backend selection and fallback

Detect at startup: if a usable DRM device is available and libseat can acquire
it, run native; otherwise fall back to the winit nested backend, and if that
also fails, print an actionable error and exit non-zero. Never leave the user
with a black screen and no message.

## Verification plan (required before this is called done)

The native backend cannot be run on the development host, because doing so would
take DRM master away from the running Omarchy session. Verification therefore
happens in a virtual machine:

1. `cargo check --features native` and `--no-default-features --features native`
   compile cleanly (clippy `-D warnings`).
2. Package the release compositor binary into the archiso `airootfs` and start
   it on the live system (systemd unit on tty1 or an autostart hook).
3. Boot the ISO in QEMU with virtio-gpu, capture the framebuffer via the QEMU
   monitor `screendump`, and OCR/verify that the compositor started and a test
   client mapped a window.
4. Hotplug and mode-change tests: add/remove a virtual output, change mode, and
   confirm the session survives an unsupported mode.

Only after steps 1–4 pass may the roadmap mark native startup as supported.
Real-hardware validation on the reference ThinkPad T450 (Intel HD 5500) remains a
separate, attended step.

## Known limitations to state honestly

- Single primary GPU first; AMD/NVIDIA and multi-GPU are follow-up work.
- Hardware cursor plane, gestures, and per-output scaling come after basic
  native scanout + input work.
- The live ISO's kernel/initramfs must already contain the Intel display driver;
  this is inherited from Arch's `linux` package and needs no ArchLume kernel
  patch for the reference hardware.
