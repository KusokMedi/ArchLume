# AGENTS.md: ArchLume

## Who you are working with

The project owner communicates in Russian. In chat, always reply to the owner in Russian. All code, comments, commit messages, documentation files (README, docs/), and package metadata are written in English.

The owner is an experienced Linux user (three or more years, currently on Omarchy, Arch-based, Hyprland on Wayland). The owner is not a full-time developer and plans to use this project as a daily driver and possibly as material for a YouTube channel. The owner has a limited time budget per week, so work should be broken into small, testable steps.

The owner does not want to write code by hand. You write the code, the owner reviews the plan and tests the results. Because of this, your explanations to the owner must be clear and in plain Russian, without unnecessary jargon. When you must use a technical term, add a short explanation in parentheses.

## Project: ArchLume

ArchLume is a lightweight, visually polished Linux distribution based on Arch Linux. It has its own desktop environment written from scratch. Name rules: "ArchLume" in documentation and titles, "archlume" for commands, package names, repository names, and file paths.

The goal is a daily-usable system that feels smooth out of the box: fluid animations, clean visuals, sensible defaults, and good performance on modest hardware.

## Non-negotiable goals

1. Arch compatibility. ArchLume uses official Arch repositories and pacman as-is. Users can install any Arch package and any AUR package through `paru`. ArchLume packages follow standard PKGBUILD conventions so they feel identical to Arch packages to the user.
2. Architecture: x86_64 only.
3. Boot: UEFI only at first. Secure Boot is a later task.
4. Minimum system requirement: 8 GB RAM.
5. Primary test hardware: Lenovo ThinkPad T450 (Intel Core i5-5300U, Intel HD Graphics 5500, 8 GB RAM, 1366x768 display, 14 inch, 221 GiB SSD with btrfs). Every feature must work on this machine. Newer hardware must also work, but the T450 is the reference.
6. No existing desktop environment or compositor as a dependency. Do not use KDE Plasma, GNOME, Hyprland, Sway, labwc, or similar as the desktop. Building blocks such as wlroots, Smithay, GTK4, Qt6, and libraries for rendering, input, and IPC are allowed, but the DE itself (compositor logic, panel, launcher, notifications, lock screen, wallpaper, settings) is ours.
7. Wayland only. No X11 session. XWayland support is a later task and must be discussed before implementation.
8. Preinstalled software is minimal: Chromium (browser), an image viewer, a video player, a file manager, a settings app, and a calculator. Everything else is installed by the user.
9. Default language: English. Russian locale must be installable and selectable at install time.
10. Visual quality: smooth animations (target 60 fps on the T450 for window open, close, move, and workspace switch), consistent theme, readable typography, sensible default fonts.

## Technical decisions (fixed, do not change without asking the owner)

- Base: Arch Linux, official repositories, pacman.
- AUR: `paru` preinstalled and configured.
- Init: systemd.
- Kernel: stock `linux` package. `linux-lts` installed as a fallback and offered in the boot menu.
- Filesystem: btrfs.
  - Subvolumes: `@` (root), `@home`, `@snapshots`, `@var_log`, `@var_cache_pacman_pkg`.
  - Mount options: `noatime,compress=zstd:1,space_cache=v2` as the default.
- Snapshots: snapper for management, snap-pac so every pacman transaction creates pre and post snapshots.
- Rollback: a documented way to boot into an older snapshot. Use grub-btrfs or the systemd-boot equivalent, depending on the bootloader decision.
- Bootloader: systemd-boot is the default for UEFI. Confirm with the owner in the open decisions list before implementing.
- Audio: PipeWire with WirePlumber.
- Networking: NetworkManager.
- Bluetooth: BlueZ.
- Power: power-profiles-daemon. Do not use TLP unless discussed.
- Graphics: Mesa. Intel drivers (`intel-media-driver` for newer GPUs, `libva-intel-driver` for older ones such as the HD Graphics 5500 on the T450, `vulkan-intel`) are the default. AMD and Nvidia drivers are optional packages, not installed by default.
- ISO build: archiso with a custom profile stored in `iso/`.
- Package repository: an ArchLume pacman repository, built by GitHub Actions, published through GitHub Releases or GitHub Pages.
- Hosting: GitHub only. No self-hosted servers.
- Updates: rolling release, identical to Arch. Users run `pacman -Syu` or use `paru`.

## Open decisions (ask the owner; do not decide them yourself)

Collect these, present them in one list, and wait for answers. Ask once, not one at a time. For each decision, propose 2 or 3 options with a short trade-off explanation in Russian.

1. Compositor stack: wlroots (C), Smithay (Rust), or another option. Explain the trade-offs in 2-3 sentences each.
2. Shell and UI toolkit: GTK4, Qt6, custom rendering (for example, using a graphics library directly), or another option.
3. Installer: a graphical front-end over a script, a Calamares-style flow, or another approach. Propose 2 options.
4. Bootloader: systemd-boot (default proposal) or GRUB.
5. Image viewer and video player: which ones to ship (for example, Loupe, Showtime, mpv, Celluloid).
6. Settings app and file manager: write from scratch or build on existing libraries.
7. License: MIT or GPL-3.0.
8. Name and logo: confirm "ArchLume" and decide whether a logo is needed now or later.
9. Programming language for the DE components, if different from the compositor stack.

## How we work

### Language rules

- Chat with the owner: Russian, always.
- Code, comments, commit messages, README, docs, PKGBUILDs, and user-facing strings in the software: English.
- Plans, questions, and test instructions to the owner: Russian.

### Workflow

1. **Plan first.** Before writing any code, produce a written plan in Russian. The plan must include:
   - The phases listed below, with concrete milestones for each.
   - Repository structure (directory tree).
   - An estimate of the work per phase, in hours and in number of sessions.
   - The open decisions list.
   - Known risks for the T450 and for the DE work.
2. **Wait for approval.** Do not write code until the owner approves the plan.
3. **One phase at a time.** Finish a phase, verify it, then stop and report.
4. **Testable milestones.** At the end of every phase, give the owner exact steps to test it: commands to build, commands to run in a VM, and what the expected result is.
5. **No guessing.** If something is unclear, do not assume. Gather all questions into one list, ask the owner once, and wait.
6. **Small commits.** Use Conventional Commits (`feat:`, `fix:`, `docs:`, `chore:`, `build:`). One logical change per commit.
7. **Document as you go.** Keep README.md, docs/build.md, docs/testing.md, and docs/architecture.md current. Update them in the same change as the code they describe.
8. **Report honestly.** If a test fails or a plan is not feasible, say so immediately in Russian and propose options. Do not hide problems.
9. **No external services without approval.** Do not add network calls, telemetry, analytics, or third-party APIs without asking the owner first.

### Code standards

- Split code into multiple files with one responsibility each. Avoid files over about 400 lines. If a file grows past that, split it.
- Every module has a short header comment describing its purpose.
- Pin dependency versions in PKGBUILDs and lock files.
- Write tests for logic that can be tested without a display (parsers, config handling, layout math, snapshot naming, and similar).
- Shell scripts: `#!/usr/bin/env bash`, `set -euo pipefail`, and run `shellcheck` on them.
- Follow the language's standard formatter and linter (for example, `rustfmt` and `clippy` for Rust, `ruff` for Python).
- No hardcoded paths to the owner's machine. Use configuration and environment variables.

### Repository structure (initial proposal, confirm in the plan)

```
archlume/
├── AGENTS.md
├── README.md
├── LICENSE
├── docs/
│   ├── architecture.md
│   ├── build.md
│   └── testing.md
├── iso/                  # archiso profile
│   ├── profiledef.sh
│   ├── packages.x86_64
│   ├── pacman.conf
│   ├── airootfs/         # files copied into the live system
│   └── efiboot/ syslinux/ grub/ (as needed)
├── packages/             # ArchLume PKGBUILDs, one folder per package
├── desktop/              # the DE: compositor, shell, panel, etc.
├── apps/                 # core apps, if written by us
├── installer/            # graphical installer
├── config/               # default configs (shipped in /etc/skel or /etc/archlume)
├── scripts/              # build, test, and helper scripts
└── .github/workflows/    # CI: ISO build, package build, repo publish
```

## Phases

Create a detailed plan for each phase. The outline below is the starting point for the plan. Do not start coding a phase until the plan is approved.

### Phase 1: Base ISO and installed system

Goal: a bootable UEFI ISO that installs a plain Arch-based system with btrfs, snapper, paru, and Chromium.

- Create the archiso profile in `iso/`.
- Add a basic installation script (without the graphical installer yet) that partitions a disk, creates btrfs subvolumes, installs base packages with pacstrap, sets up systemd-boot, and creates a user.
- Configure snapper and snap-pac.
- Install Chromium and paru.
- Test in VirtualBox or QEMU with OVMF firmware. Provide exact commands for both.
- Milestone: a VM boots the ISO, the install script completes, the installed system boots, `pacman -Syu` works, `paru -S` works, and Chromium launches.

### Phase 2: ArchLume package repository

Goal: ArchLume's own packages, built and published automatically.

- Create PKGBUILDs for ArchLume packages (for example, `archlume-base`, `archlume-settings`, `archlume-theme`).
- Set up a local build with `makepkg` and a local repository with `repo-add`.
- Add GitHub Actions workflows to build packages and publish the repository.
- Add the repository to `pacman.conf` on installed systems.
- Milestone: `pacman -Syu` on a test VM pulls ArchLume packages from the published repository.

### Phase 3: Compositor (the core of the DE)

Goal: a working Wayland compositor with window management and basic animations.

- Implement the compositor using the stack chosen by the owner.
- Features in this phase: start a session, tiling and floating window modes, workspaces, keyboard shortcuts, multi-monitor basics, and window open, close, move, and workspace-switch animations.
- Performance target: 60 fps animations on the T450 with Intel HD Graphics 5500. Report measured frame times, not impressions.
- Test in a VM first (virtio-gpu with 3D acceleration if available), then on the T450.
- Milestone: the owner can log in to the session, open two terminals, tile them, switch workspaces with an animation, and log out.

### Phase 4: Shell

Goal: the basic desktop shell around the compositor.

- Panel with clock, workspace indicator, system tray (network, audio, battery).
- Application launcher with search.
- Notifications.
- Wallpaper.
- Lock screen with password authentication through PAM.
- Milestone: a full session that a user can run day to day without a terminal.

### Phase 5: Core apps

Goal: the minimum set of apps shipped with the system.

- File manager.
- Settings app: display, network, audio, power, keyboard, language, and theme.
- Calculator.
- Image viewer and video player: use the owner's choice from the open decisions list. Integrate them with the theme and with file associations.
- Chromium: default browser, with ArchLume defaults applied through policy or profile files.
- Milestone: all core apps open, respect the theme, and work with the file associations.

### Phase 6: Graphical installer

Goal: a user can install ArchLume from the live ISO without a terminal.

- Build the installer following the approach chosen by the owner.
- Steps: language, keyboard, timezone, disk selection (with clear warnings before erasing data), btrfs layout, user creation, optional Russian locale, and install progress.
- Milestone: a fresh VM installs ArchLume fully through the GUI and boots into the desktop.

### Phase 7: Update and rollback workflow

Goal: safe updates with a way back.

- Verify snap-pac creates snapshots on every pacman transaction.
- Add a tool or documented procedure to list snapshots and roll back.
- Configure the bootloader to offer snapshot boot entries.
- Milestone: install a broken package on purpose, reboot into the previous snapshot, and confirm the system returns to the working state.

### Phase 8: Hardware support on the T450

Goal: everything works on the reference hardware.

- Intel graphics: correct driver selection, hardware video acceleration, and working brightness control.
- Wi-Fi: Intel wireless card support with NetworkManager.
- Bluetooth: pairing and audio.
- Audio: PipeWire with working headphone and speaker switching.
- Power: battery status, suspend and resume, power profiles, and lid behavior.
- Keyboard: function keys, ThinkPad-specific keys where possible.
- Milestone: a checklist in `docs/hardware-t450.md` where every item is tested on the physical laptop and marked with the date.

### Phase 9: Polish, documentation, and alpha release

Goal: a public alpha that another person can install.

- Finalize README with install instructions, screenshots placeholders, and known issues.
- Write docs/architecture.md for contributors.
- Tag an alpha release on GitHub with the ISO attached.
- Milestone: a person who has never seen the project can download the ISO, install it, and use it.

## Testing

### Virtual machines

- VirtualBox: enable EFI in VM settings, allocate at least 4 GB RAM and 2 CPUs, enable 3D acceleration for GUI tests.
- QEMU: use OVMF firmware. Example: `qemu-system-x86_64 -enable-kvm -m 4096 -smp 2 -drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2/x64/OVMF_CODE.4m.fd -cdrom archlume.iso -device virtio-vga-gl -display gtk,gl=on`.
- Always document the exact command and VM settings used for each test in `docs/testing.md`.

### Real hardware

- Test on the ThinkPad T450 only after a phase is stable in a VM.
- Record results in `docs/hardware-t450.md`: date, ArchLume version, what worked, what failed.

### Automated checks

- CI runs shellcheck, formatter checks, linters, unit tests for logic, and an ISO build on every pull request.
- Package builds run in CI for every PKGBUILD change.

## Risks to report early

- The compositor and shell are the largest parts of the project and the most likely to slip. Report progress honestly against the milestones.
- Intel HD Graphics 5500 is old. Some Wayland features and performance paths may be limited. Test early in Phase 3.
- Writing a compositor from scratch is a large task. If the owner wants a faster route, propose building on wlroots or Smithay as libraries rather than writing the protocol layer ourselves.
- Installers that erase disks are dangerous. Every destructive step needs a confirmation and a clear warning.

## What to do first

1. Read this file completely.
2. Reply to the owner in Russian with a short summary of your understanding and a list of any questions that block planning.
3. Produce the Phase 1 plan in Russian, including the open decisions list, the repository structure, the work estimate, and the risks.
4. Stop and wait for the owner's approval before writing any code.
