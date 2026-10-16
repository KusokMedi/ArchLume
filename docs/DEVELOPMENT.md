# Development Guide

This document explains how to build, run, and test ArchLume, and the rules that
keep development safe and reproducible.

## 0. Non-negotiable: do not disturb the host

ArchLume is **developed on a live host machine that is also someone's daily
driver**. Treat the host desktop as untouchable:

- Do **not** modify, disable, or reconfigure the host compositor, shell,
  session, system config, bootloader, or disks.
- Do **not** run destructive disk operations (partitioning, formatting, `dd` to
  a block device, `mkfs`, installers) on the host. All installation testing
  happens inside QEMU with a virtual disk.
- Test the compositor **nested** inside the host Wayland session, or inside a
  QEMU VM. Never take over the host's DRM/logind session.
- Installing development packages (toolchains, QEMU, archiso) is allowed; see
  §1.

## 1. Toolchain

Target platform: Arch Linux (`x86_64`). Install prerequisites:

```sh
# Rust toolchain (version pinned by rust-toolchain.toml)
sudo pacman -S --needed rustup
rustup toolchain install stable --profile minimal --component rustfmt --component clippy

# Build/run dependencies for the Smithay compositor
sudo pacman -S --needed base-devel pkgconf \
  wayland wayland-protocols libxkbcommon libinput libdrm \
  mesa libglvnd seatd libdisplay-info hwdata

# ISO build + virtualized testing
sudo pacman -S --needed archiso qemu-desktop edk2-ovmf

# Optional convenience runner
# sudo pacman -S --needed just
```

> On Omarchy hosts, direct `pacman -Syu` (system upgrade) is blocked by an
> Omarchy guard on purpose; use `omarchy update` for system upgrades and plain
> `pacman -S <pkg>` (no `-u`) for adding packages. This applies only to the
> developer's host, never to an installed ArchLume system.

## 2. Build, format, lint, test

With [`just`](https://github.com/casey/just) installed, the canonical commands
are:

```sh
just build     # cargo build --workspace
just test      # cargo test --workspace
just fmt       # cargo fmt --all
just fmt-check # cargo fmt --all -- --check
just lint      # cargo clippy --workspace --all-targets -- -D warnings
just check     # fmt-check + lint + test
```

Equivalent raw commands (usable before a Cargo workspace exists):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

Lint is **deny-warnings**: warnings are treated as failures.

## 3. Running the compositor (nested)

During development the compositor runs as a client of the host Wayland session
(winit backend). This never touches the host's display session:

```sh
just run-nested     # cargo run -p archlume-compositor
just smoke          # nested smoke test: a client maps a window and exits
just control-test   # drive workspace state over the control socket
just preview-test   # verify the snap-preview overlay with real pixels
just panel-test     # run archlume-shell against it and verify the panel pixels
```

The compositor exposes a line-based control socket at
`$XDG_RUNTIME_DIR/archlume-control.sock` (path also in `$ARCHLUME_CONTROL`); see
`compositor/src/control.rs` and ADR-0007. Commands: `ping`, `workspace`,
`switch <n>`, `move <n>`, `windows`, `window <i>`, `close <i>`, `stats`,
`preview <zone|off>`, `gesture <fingers> <dx> <dy>`, `layers`.
`just control-test` uses it to verify workspace switching, window counting,
focus-on-open, moving a window, cascade placement, close and frame stats at
runtime without needing to inject input events.

The desktop shell is the separate `archlume-shell` crate. It connects to a
running compositor and places a top panel with `wlr-layer-shell`; run it against
the nested compositor with:

```sh
cargo build -p archlume-compositor -p archlume-shell
just panel-test   # starts both, then verifies the bar with grim + ImageMagick
```

(Native DRM/libseat operation is a Milestone 4 deliverable and is only tested on
dedicated hardware or in a VM, never on the development host's session. See
`docs/NATIVE-BACKEND.md`.)

## 4. ISO build and QEMU testing

The `archiso` profile lives in `iso/profile/` (derived from the upstream
`releng` profile, UEFI/`systemd-boot` only, English live environment). It uses
the official Arch mirror CDN for reproducible builds.

```sh
just iso            # sudo mkarchiso -v -w iso/work -o iso/out iso/profile
just qemu           # boot the newest ISO with a disposable virtual disk (GPU)
just boot-test      # headless boot check: capture the framebuffer + OCR it
```

`just boot-test` boots the newest ISO under UEFI/OVMF with `-display none`,
dumps the emulated framebuffer through the QEMU monitor (`screendump`), and
verifies via `tesseract` that the live environment reached a shell. It requires
KVM and `edk2-ovmf`. Override `ISO_FILE`, `BOOT_WAIT` (default 180s), or
`MARKER` (an extended regex) via the environment.

- `iso/work/` and `iso/out/` are build artifacts and are git-ignored.
- For reproducible metadata, export `SOURCE_DATE_EPOCH` before building; the
  profile derives the ISO version/label from it via `profiledef.sh`.
- Building downloads packages from the mirrors in `iso/profile/pacman.conf`
  and requires root (`mkarchiso`), so `just iso` uses `sudo`.

QEMU uses a **disposable virtual disk**; the host disks are never passed
through. UEFI is the first-class target (`edk2-ovmf`).

## 5. Conventions

- **Language:** all repository artifacts (code, comments, docs, commit
  messages, UI strings) are **English**. Chat with the owner may be in Russian.
- **Formatting:** `rustfmt` (see `rust-toolchain.toml`), `-D warnings` clippy.
- **Commits:** small and frequent, one logical change each; imperative mood
  ("add", "fix", "refactor"). Do not push unless explicitly authorized.
- **No generated artifacts** in commits (build outputs, ISOs, caches).
- **No secrets**: never commit keys, tokens, or credentials.
- **Strings for UI** live in translation resources, separate from logic, even
  though only English ships initially.
- Add tests alongside behavior, not in a later phase.

## 6. Evidence, not claims

`AGENTS.md` is explicit: never claim a feature, test, or release succeeded
without concrete evidence. Record the exact command and its output when
reporting progress. If something cannot be verified (missing hardware, CI
quota), say so and document the limitation.
