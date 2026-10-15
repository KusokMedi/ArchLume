# ArchLume Architecture

> Status: living document. Written during Milestone 1. Sections marked
> **planned** or **open** are not implemented yet; they record the intended
> design so implementation can follow in dependency order. Do not read planned
> items as delivered functionality.

## 1. Goals and constraints

ArchLume is a rolling-release Arch Linux distribution whose defining feature is
a coherent, Android-inspired desktop built on an **ArchLume-owned Wayland
compositor and shell**.

Hard constraints from [`AGENTS.md`](../AGENTS.md):

- Base: official Arch repos, `pacman`, `systemd`, `yay` (no `paru`).
- `x86_64` only initially; UEFI first-class; official `linux` + `linux-lts`.
- No existing desktop environment or compositor as the actual desktop.
- Reference hardware: Lenovo ThinkPad T450 (i5-5300U, HD 5500, 8 GB RAM,
  1366x768). Baseline target: any x86_64 with 8 GB RAM.
- Free infrastructure only; never incur charges.
- English-first, localization-ready (strings separated from logic).
- License undecided; no public distribution until the owner decides.

Primary quality bar: **smoothness and consistency**. Frame budget at 60 Hz is
~16.67 ms; at 144 Hz ~6.94 ms. Heavy work (package ops, filesystem scans,
network) must stay off the compositor/shell UI thread.

## 2. Component map

```
+---------------------------------------------------------------+
|  First-party applications (settings, terminal, files, ...)     |
+---------------------------------------------------------------+
|  Desktop shell  (panel, launcher, quick settings, notifications|
|                  lock/login surfaces, OSD)                     |
+---------------------------------------------------------------+
|  Design system / theme engine (tokens, components, dynamic)    |
+---------------------------------------------------------------+
|  ArchLume Wayland compositor  (Rust + Smithay)                 |
+---------------------------------------------------------------+
|  Wayland / session plumbing (libseat, logind, xdg-desktop-portal)|
+---------------------------------------------------------------+
|  Base system (systemd, PipeWire/WirePlumber, NetworkManager,   |
|  BlueZ, power-profiles-daemon, Mesa, Btrfs, snapper)           |
+---------------------------------------------------------------+
|  Boot (systemd-boot, optional LUKS2, Btrfs subvolumes)         |
+---------------------------------------------------------------+
```

Cross-cutting systems:

- **Update center** — one UI for system, application, and AI-agent updates.
- **Recovery** — Btrfs snapshots/rollback, built-in tools, ISO recovery.
- **Extensions/theme SDK** — versioned public APIs, catalog, publishing.
- **Installer** — graphical, hardware-adaptive, safe by default.

## 3. Technology choices

See [`DECISIONS.md`](DECISIONS.md) for the record and rationale. Summary:

| Area | Choice | Notes |
| --- | --- | --- |
| Language | Rust | compositor, shell core, most first-party tooling |
| Compositor foundation | [Smithay](https://github.com/Smithay/smithay) | proven Wayland server library; not a full graphics stack rewrite |
| Bootloader | `systemd-boot` | default for installed systems and ISO (UEFI) |
| Filesystem | Btrfs | subvolumes `@`, `@home`, `@snapshots`, `@var_log`, `@var_cache_pacman_pkg` |
| Snapshots | Snapper + `snap-pac` | rollback coordinated with bootloader |
| Audio | PipeWire + WirePlumber | |
| Network / BT | NetworkManager / BlueZ | |
| Power | `power-profiles-daemon` | |
| Graphics | Mesa (Intel/AMD); NVIDIA where feasible | capability-detected |

## 4. Compositor (planned)

- Foundation: Smithay. ArchLume owns window management, workspaces, gestures,
  and shell integration.
- Backends:
  - **winit** — nested development (runs as a client inside the current
    Wayland session) for fast iteration.
  - **udev/DRM (libseat)** — native session with DRM master.
- Renderer: **GLES2/EGL first** for the widest Intel/AMD compatibility (this is
  what the reference HD 5500 needs); a Vulkan renderer may be added later where
  measured to help.
- Input: libinput through Smithay; keyboard via xkbcommon.
- Protocols (initial set): `xdg-shell`, `wlr-layer-shell`, `xdg-decoration`,
  `xdg-output`/`wlr-output-management`, `primary-selection`,
  `idle-inhibit`, `linux-dmabuf`, `fractional-scale`.
- XWayland: later, never the primary session.
- Session model: compositor runs as a user process holding DRM via libseat;
  `logind` manages the session.

Window management rules (from the brief) are enforced at compositor level:
floating by default; drag-to-top maximizes; drag-to-left/right snaps to half;
10 fixed shared workspaces; `Super+1..9`, `Super+0`; `Shift+Super+digit` moves
and follows; `Super` alone is a no-op.

## 5. Desktop shell (planned)

Panel (top only by default) with optional dock; launcher; quick settings;
notification center/history; lock and login surfaces; OSD. Implementation stack
for shell surfaces is **open** — candidates are a Rust/GTK4 shell process or a
custom Smithay client. This decision will be recorded before Milestone 5.

Only the top panel is enabled by default. Disabling the dock must not reserve
screen area.

## 6. Design system (planned)

- Semantic color roles and states; typography scale; spacing scale; radii.
- Reusable components with focus/hover/active/disabled/loading/error states.
- Motion guidelines with a reduced-motion mode.
- Light/dark themes, contrast/accessibility modes, token files.
- Dynamic color from wallpaper (Material You-inspired) is a goal, not a
  guarantee, and must not be claimed for third-party apps that ignore it.

## 7. Update, snapshot, and recovery (planned)

- Unified update center using `pacman`/`yay`; AI agents are updated through the
  same center, not a separate updater.
- Preflight checks, metadata/dependency verification, Btrfs snapshot before
  system updates, rollback on failure.
- Three channels: Stable (default), Testing, Development.
- Signed, versioned update policies; fail-safe behavior when sources are
  offline, stale, or tampered. Never install unverified artifacts.
- ISO recovery detects installed systems, mounts/repairs, unlocks LUKS, and
  requires explicit confirmation before any destructive action.

Signing uses separate keys for packages, ISOs, and policies; private keys are
never committed. Where a robust offline root-of-trust cannot be automated on
free CI, the limitation is documented and the safest feasible workflow used.

## 8. Extensions, themes, and SDK (planned)

- Versioned public APIs; compatibility checked on update; incompatible
  extensions disabled with an explanation and rollback.
- Catalog distinguishes official vs. external content.
- Theme packs get no system privilege by mere installation.
- The project is code/SDK-first, not a visual extension builder.

## 9. Repository and infrastructure

Single monorepo during early development (ADR-0001). `AGENTS.md` additionally
anticipates `ArchLume-Web`, `ArchLume-Extensions`, and `ArchLume-Repository`;
these will be split out when concrete need arises and when publishing is
authorized.

Infrastructure must stay on free tiers and be portable across providers.
Existing releases must never be silently deleted to hide capacity problems.

## 10. Risks and open questions

- **Compositor scope** is the largest risk: a production Wayland compositor is a
  multi-year effort. Mitigation: stand on Smithay and deliver a thin, working
  vertical slice early, then expand.
- **Shell stack** is open (section 5).
- **Free CI/ISO capacity**: full ISO builds and QEMU tests are heavy; strategy
  must cache aggressively and queue when quotas are exhausted.
- **Signing root of trust** on free CI (section 7).
- **NVIDIA specifics** are unverified; no universal support will be claimed.
- **Licensing** blocks public distribution until decided.
