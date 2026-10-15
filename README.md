# ArchLume

An Arch Linux-based distribution with an ArchLume-owned Wayland compositor and a
coherent, Android-inspired desktop experience.

> **Status: early development.**
> The project is at **Milestone 1 (repository and build foundation)**. There is
> no installable ISO yet. The acceptance target is a **verified alpha**, not a
> mock-up. See [`docs/ROADMAP.md`](docs/ROADMAP.md). Nothing in this repository
> should be treated as complete, released, or production-ready.

## What ArchLume is

- A rolling-release Arch Linux system (`pacman`, `systemd`, `yay`) with its own
  Wayland compositor and desktop shell.
- A coherent first-party desktop: top panel, application launcher, quick
  settings, notification center, lock/login screens, settings, and core
  applications built on a single design system.
- A graphical installer, a unified update center, Btrfs snapshot/rollback, and
  an ISO recovery workflow.
- A code/SDK-first extension and theme ecosystem.

## What ArchLume is not

- It is **not** a preconfigured KDE, GNOME, Hyprland, Sway, or labwc setup.
  ArchLume owns its compositor and shell; it does not embed another desktop
  environment. Underlying libraries and protocols (Smithay, `wayland-server`,
  Mesa, PipeWire, ...) are used as foundations.
- It is **not** a mock-up. A component is only considered done when it runs and
  has been tested. A successful compilation or a screenshot is not evidence of
  completion.

## Repository layout

This repository is a **monorepo** for the initial development phase. Dedicated
repositories (`ArchLume-Web`, `ArchLume-Extensions`, `ArchLume-Repository`) will
be split out later when a concrete need justifies it (see
[`docs/DECISIONS.md`](docs/DECISIONS.md), ADR-0001).

| Path | Purpose | State |
| --- | --- | --- |
| `compositor/` | ArchLume Wayland compositor (Rust / Smithay) | planned |
| `shell/` | Desktop shell: panel, launcher, quick settings, notifications | planned |
| `apps/` | First-party core applications (settings, terminal, files, ...) | planned |
| `installer/` | Graphical installer and initial setup | planned |
| `recovery/` | Recovery tools and ISO recovery workflow | planned |
| `iso/` | Reproducible `archiso` profile | planned |
| `packaging/` | Package metadata and PKGBUILDs | planned |
| `extensions/` | Extension/theme SDK, templates, examples | planned |
| `docs/` | Architecture, roadmap, developer docs, decision log | active |
| `.github/workflows/` | CI | active |

## Documentation

- [`AGENTS.md`](AGENTS.md) — the primary project brief and operating rules.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — system architecture and
  component boundaries.
- [`docs/ROADMAP.md`](docs/ROADMAP.md) — milestones, status, and acceptance
  criteria.
- [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) — how to build, run, and test.
- [`docs/DECISIONS.md`](docs/DECISIONS.md) — architecture decision records.

## Development

See [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md).

## License

**Undecided.** ArchLume currently carries no license and must not be published
under one until the project owner makes an explicit licensing decision
(`AGENTS.md` §20). Do not add a `LICENSE` file or license headers without that
decision. Third-party components retain their own licenses and attributions.
