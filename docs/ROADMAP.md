# ArchLume Roadmap

Milestones follow `AGENTS.md` §24 (dependency order). Each milestone lists
**deliverables** and **acceptance criteria**. A milestone is `done` only when its
acceptance criteria have been verified with concrete evidence — not when code
compiles or a screenshot looks right.

Legend: `done` · `in progress` · `planned`

---

## M1 — Repository and build foundation

**Status: done (local) — first CI run pending a push**

Deliverables
- Repository layout, README, architecture/roadmap/development docs, decision log.
- Pinned Rust toolchain; formatting/lint configuration.
- Build/lint/test entry points (`just` targets).
- CI skeleton.

Acceptance criteria
- [x] Layout, docs, and ADRs committed (`ac9597f`).
- [x] Rust toolchain pinned and reproducible (`rust-toolchain.toml`).
- [x] `just fmt`, `just lint`, `just test`, `just build` defined.
- [ ] CI workflow runs and is honest about what it checks. *(workflow committed;
      not yet executed because the owner has no GitHub push auth — see
      `docs/DECISIONS.md`.)*

## M2 — Base system and reproducible ISO

**Status: in progress** (profile added, 2026-10-15)

Deliverables
- Reproducible `archiso` profile under `iso/profile/` (UEFI / systemd-boot,
  official Arch mirrors, ArchLume branding, English live motd).
- Base packages/services (NetworkManager, PipeWire/WirePlumber, BlueZ,
  power-profiles-daemon, Mesa), systemd-boot UEFI path, hardware detection.

Acceptance criteria
- [x] ISO builds reproducibly from a documented command (`just iso`).
      Evidence (2026-10-15): `iso/out/archlume-2026.10.15-x86_64.iso`
      (1.6 GiB), label `ARCHLUME_202610`, sha256
      `d3f125d779cac0f9b052bb04ae5d58e168204c6efad803b40a8ed9c4da61cff1`.
- [x] ISO boots in QEMU (UEFI) to a working shell (`just boot-test`).
      Evidence (2026-10-15): `scripts/boot-iso-qemu.sh` PASS — the live
      environment reached a root shell in QEMU/OVMF; OCR of the captured
      framebuffer shows `archlume login: root (automatic login)` and the live
      motd.
- [ ] Installation test (installer lands in M7); boot test automated with
      evidence (done).

## M3 — Package repository and CI

**Status: planned**

Deliverables
- Package metadata, build workflow, versioning, signing design, release
  artifacts, free-resource fallback/mirroring.

Acceptance criteria
- [ ] Catalog validated by CI; artifacts signed and verifiable.
- [ ] Failure/outage handling documented and tested.

## M4 — Wayland compositor foundation

**Status: in progress** (first nested slice exists, 2026-10-15)

A minimal but real Smithay-based compositor lives under `compositor/`. It runs
on the winit backend (nested), implements xdg-shell toplevels/popups,
pointer/keyboard input, focus, and window move/resize grabs, and is verified by
`scripts/smoke-nested.sh` (a client maps a window on the reference GPU).

Window-management policy (`compositor/src/wm.rs`) implements floating-by-default
with drag-to-edge snapping: top edge maximizes, left/right edges snap to halves,
and corners snap to quarters. The zone/geometry logic is pure and unit-tested;
the move grab applies the resulting xdg_toplevel configure on drop.

Fixed workspaces (AGENTS 6.4) are implemented in `compositor/src/workspace.rs`:
ten workspaces are modelled as horizontal regions of the space and switching
moves the output's view, so windows stay mapped and frame/commit handling is
unchanged. `Super + 1..9`/`0` selects a workspace and `Shift + Super + digit`
moves the focused window to a workspace and follows it. The index/offset logic
is pure and unit-tested; the keyboard bindings compile and are wired but have
not yet been exercised at runtime (the nested winit backend cannot feed
synthetic key events to the compositor).

Still to come: native DRM/libseat startup, gestures, snap preview, and a way to
un-snap back to the previous floating geometry.

Deliverables
- Smithay-based compositor: session startup, outputs, input, surfaces, basic
  window management, safe fallback.

Acceptance criteria
- [ ] Runs nested in a host Wayland session (dev) and on DRM/libseat (native).
- [ ] Renders outputs; handles keyboard/pointer input; xdg-shell clients map.
- [ ] Meets an initial frame-time budget at 60 Hz on the reference T450.

## M5 — Desktop shell

**Status: planned**

Deliverables
- Panel, launcher, quick settings, notification center, workspaces, theming,
  keyboard action system.

Acceptance criteria
- [ ] Panel/launcher/quick settings reflect real system state.
- [ ] Action system binds commands/scripts/settings actions safely (polkit for
      privileged actions).
- [ ] Measurable frame-time/input-latency criteria met.

## M6 — Core applications

**Status: planned**

Deliverables
- Settings, terminal (via proven component), file manager, calculator,
  image/video viewer, Chromium integration, application store.

Acceptance criteria
- [ ] Each shipped app performs its advertised actions (no dead controls).
- [ ] Store supports pacman/ArchLume/AUR sources with clear labels; Flatpak/Snap
      optional and off by default.

## M7 — Installer and accounts

**Status: planned**

Deliverables
- Graphical installer with partitioning safeguards, optional LUKS2,
  users/roles (OWNER/ADMIN/USER), permission editor, guest mode.

Acceptance criteria
- [ ] Destructive operations require explicit confirmation.
- [ ] Install completes in QEMU and boots the installed system.
- [ ] Permissions enforced at the polkit/system layer, not only UI hiding.

## M8 — Update center and recovery

**Status: planned**

Deliverables
- Unified update center, snapshot/rollback, package safety, recovery
  environment, channels and policy.

Acceptance criteria
- [ ] Snapshot created before system update; rollback tested end to end.
- [ ] Offline/stale/tampered/conflict states handled with user guidance.
- [ ] ISO recovery repairs a broken boot without reinstalling where possible.

## M9 — Hardware, power, audio, multi-monitor

**Status: planned**

Deliverables
- T450 validation, Intel graphics, PipeWire, gestures, hibernation where
  supported, multi-monitor profiles and hotplug.

Acceptance criteria
- [ ] Verified behavior documented on the T450 (and limitations elsewhere).
- [ ] Unsupported display modes never lose the active session.

## M10 — Extensions, Skills, localization foundation, backup UI

**Status: planned**

Deliverables
- Stable APIs, SDK, catalogs, AI-agent skill adapters, user-data backup UI.

Acceptance criteria
- [ ] Extensions install/update with compatibility checks; no auto-execution of
      untrusted scripts.
- [ ] Backup restore is actually tested, not only backup creation.

## M11 — Polish and verified alpha

**Status: planned**

Deliverables
- Accessibility pass, performance profiling, docs, CI matrix, ISO release and
  known-issues report.

Acceptance criteria — the **verified alpha** checklist (`AGENTS.md` §23):
- [ ] Reproducible source/build workflow.
- [ ] Bootable, installable ISO verified in automated virtualized tests.
- [ ] Core desktop session and preinstalled apps functioning.
- [ ] Package/update workflows with safe failure handling.
- [ ] Tested snapshot/rollback and recovery flows (to the extent implemented).
- [ ] Security and signature checks for published artifacts.
- [ ] Generated release documentation.
- [ ] Automated tests and honest reports of remaining gaps.
- [ ] Installation and developer documentation.
- [ ] List of known limitations and unverified hardware support.
