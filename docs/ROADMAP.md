# ArchLume Roadmap

Milestones follow `AGENTS.md` §24 (dependency order). Each milestone lists
**deliverables** and **acceptance criteria**. A milestone is `done` only when its
acceptance criteria have been verified with concrete evidence — not when code
compiles or a screenshot looks right.

Legend: `done` · `in progress` · `planned`

---

## M1 — Repository and build foundation

**Status: in progress**

Deliverables
- Repository layout, README, architecture/roadmap/development docs, decision log.
- Pinned Rust toolchain; formatting/lint configuration.
- Build/lint/test entry points (`just` targets).
- CI skeleton.

Acceptance criteria
- [ ] Layout, docs, and ADRs committed.
- [ ] Rust toolchain pinned and reproducible.
- [ ] `just fmt`, `just lint`, `just test`, `just build` defined.
- [ ] CI workflow runs and is honest about what it checks.

## M2 — Base system and reproducible ISO

**Status: planned**

Deliverables
- Reproducible `archiso` profile under `iso/`.
- Base packages/services (NetworkManager, PipeWire/WirePlumber, BlueZ,
  power-profiles-daemon, Mesa), systemd-boot UEFI path, hardware detection.

Acceptance criteria
- [ ] ISO builds reproducibly from a documented command.
- [ ] ISO boots in QEMU (UEFI) to a working shell.
- [ ] Installation/boot test automated or documented with evidence.

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

Still to come: native DRM/libseat startup, workspaces, gestures, snap preview,
and a way to un-snap back to the previous floating geometry.

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
