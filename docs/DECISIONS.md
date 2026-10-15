# Architecture Decision Records

Short, dated records of decisions that shape ArchLume. Newest decisions are
appended. Format: context → decision → consequences → status.

---

## ADR-0001 — Use a monorepo for initial development

- **Date:** 2026-10-15
- **Status:** accepted

**Context.** `AGENTS.md` §4 describes four repositories: `ArchLume`,
`ArchLume-Web`, `ArchLume-Extensions`, `ArchLume-Repository`. Only `ArchLume`
exists, and early development needs rapid, cross-cutting changes with a single
reviewable history.

**Decision.** Develop everything in the `ArchLume` monorepo for now, organized
by directory (`compositor/`, `shell/`, `iso/`, ...). Split into dedicated
repositories later when a concrete need justifies it and when publishing is
authorized.

**Consequences.** Simpler atomic changes and CI early on. The split will require
a migration step later; directory boundaries are kept clean to ease that.

---

## ADR-0002 — Build the compositor on Smithay (Rust)

- **Date:** 2026-10-15
- **Status:** accepted

**Context.** ArchLume must own a Wayland compositor and must not embed an
existing desktop. Writing a graphics/wayland stack from scratch is infeasible;
`AGENTS.md` §6.1 permits libraries and evaluates Smithay and GTK4.

**Decision.** Use **Smithay** (Rust) as the compositor foundation. ArchLume owns
window management, workspaces, gestures, and shell integration; Smithay provides
the Wayland server, input, and rendering plumbing. Initial renderer is GLES2/EGL
for broad Intel/AMD support (the reference HD 5500 requires it).

**Consequences.** Requires a Rust toolchain and a real learning/implementation
curve. Full compositor maturity is a long effort — mitigated by an early thin
working slice. Rejected: building on an existing compositor (would violate the
"own compositor" requirement).

---

## ADR-0003 — Default bootloader is systemd-boot

- **Date:** 2026-10-15
- **Status:** accepted

**Context.** `AGENTS.md` §5.3 proposes `systemd-boot` after verifying
constraints. The reference machine (T450) currently runs Limine under Omarchy,
but that is the *development host*, not an installed ArchLume system.

**Decision.** Use `systemd-boot` as the default bootloader for installed
ArchLume systems and the ISO. UEFI is the first-class target. The installer must
explain unsupported configurations rather than pretend they work.

**Consequences.** Simple, well-supported UEFI path. Bootloader/Btrfs-snapshot
rollback integration must be designed and tested explicitly (systemd-boot has no
built-in snapshot menu).

---

## ADR-0004 — English-first repository artifacts

- **Date:** 2026-10-15
- **Status:** accepted

**Context.** `AGENTS.md` §5.1 and §21 require English-only interface and
documentation initially, with a localization-ready architecture. The project
owner communicates in Russian.

**Decision.** Code, comments, documentation, UI strings, and commit messages are
written in **English**. The architecture keeps strings in translation resources.
Conversation with the owner may be in Russian.

**Consequences.** Ready for future localization; requires discipline to avoid
hard-coded strings in UI logic.

---

## ADR-0005 — Isolate development from the host desktop

- **Date:** 2026-10-15
- **Status:** accepted

**Context.** Development happens on the owner's only laptop, running a live
desktop the owner depends on. An accidental change could make it unusable.

**Decision.** The host desktop/session/system/disks are off-limits for
modification. ArchLume is tested **nested** in the host Wayland session or in
QEMU with a disposable virtual disk. Installing dev packages is permitted.

**Consequences.** Some features (native DRM/libseat, boot, installer) cannot be
exercised directly on the dev host and are validated in VMs. Slower for
system-level work but safe.

---

## ADR-0006 — On Omarchy hosts, upgrade via `omarchy update`

- **Date:** 2026-10-15
- **Status:** accepted

**Context.** The development host runs Omarchy, which installs a pacman hook
(`omarchy-update-pacman-guard`) that blocks direct `pacman -Syu` system
upgrades, redirecting to `omarchy update` (which snapshots first). A detached
`pacman -Syu` failed its transaction hooks for this reason.

**Decision.** On Omarchy hosts, perform *system upgrades* with `omarchy update`
(not direct `pacman -Syu`). Adding individual packages with `pacman -S <pkg>`
(no `-u`) is allowed by the guard and is used for dev tooling. This is a
host-only operational rule; ArchLume itself uses normal `pacman`.

**Consequences.** Host updates gain Omarchy's snapshot safety and migrations.
Detached runs of `omarchy update` need `OMARCHY_PATH=/usr/share/omarchy` set and
stdin redirected from `/dev/null`.

---

## ADR-0007 — Line-based control socket for shell/test IPC

- **Date:** 2026-10-15
- **Status:** accepted

**Context.** The shell (M5) must drive the compositor (switch workspaces, list
windows, ...), and automated tests need a way to observe compositor state
without injecting input events. A full compositor IPC protocol is a large design
effort that would block earlier progress.

**Decision.** Provide a minimal, dependency-free, line-based protocol over a
UNIX socket at `$XDG_RUNTIME_DIR/archlume-control.sock` (also exported as
`ARCHLUME_CONTROL`). Commands return single-line `ok ...` / `err ...` responses
(see `compositor/src/control.rs`). It is explicitly **unversioned** and may be
replaced by a richer, versioned protocol once the shell's needs are clearer.
Requests are handled on the event-loop thread via a calloop channel, so handlers
mutate compositor state directly.

**Consequences.** Enables the shell and runtime tests immediately, and is
verified by `scripts/control-test.sh` (`just control-test`). Because it can
mutate state, access should later be restricted (e.g. socket permissions,
per-user runtime dir). Cross-process state dumps are limited to what the
protocol exposes; expand deliberately.
