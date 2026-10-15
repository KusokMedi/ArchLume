# ArchLume — Autonomous Development Instructions

## 1. Mission and Operating Rules

Build **ArchLume**, an Arch Linux-based distribution with its own Wayland compositor, desktop shell, system settings, core applications, installer, package/update infrastructure, and recovery tools.

This file is the primary project brief. Read it completely before making changes. Then begin implementation autonomously. Do not repeatedly ask the project owner to approve routine engineering decisions. Choose sound architectures, libraries, implementation details, and development order yourself, document important decisions, implement incrementally, and test continuously.

Ask the owner only when a decision is genuinely necessary and cannot be safely inferred, especially for:
- destructive or irreversible operations;
- publication to an external/public destination when authorization is not already established;
- paid services, subscriptions, or actions that could incur charges;
- a major product decision explicitly reserved for the owner;
- a security or licensing ambiguity that prevents safe distribution.

Never initiate paid services, paid upgrades, or automatic charges. Never claim that a feature, test, release, file change, deployment, or publication succeeded unless there is concrete evidence.

The owner has completed a requirements interview through question 83. Treat the decisions in this document as accepted requirements. Do not repeat the interview. If a minor detail is unspecified, make a conservative, documented engineering decision and continue.

The project is intended to be developed by AI coding agents. Prefer clear structure, reproducible commands, explicit acceptance criteria, meaningful tests, and concise progress reports. Keep this document and the implementation synchronized as decisions evolve.

## 2. Product Definition

ArchLume is a rolling-release Linux distribution based on Arch Linux. Its defining feature is a coherent, Android-inspired desktop experience implemented using an ArchLume-owned Wayland compositor and shell—not by embedding an existing desktop environment.

The product includes:
- Arch-based system and installation ISO;
- a custom Wayland compositor;
- a desktop shell/panel, application launcher, quick settings, notifications, lock and login screens;
- a settings application and core desktop applications;
- package management and a unified application store;
- a unified update center for the system, applications, and supported AI coding agents;
- Btrfs snapshots, rollback, diagnostics, and recovery;
- a graphical installer and ISO recovery workflow;
- a documented extension/theme ecosystem and SDK;
- a public website/documentation and independently maintained package/release infrastructure.

The goal is a **verified alpha**, not a mock-up. A component must not be described as complete merely because it compiles or renders a screenshot.

## 3. Autonomy, Planning, and Reporting

Work through milestones autonomously:
1. Inspect the repository and existing code before changing anything.
2. Create or update a practical implementation plan and architecture notes.
3. Establish build, lint, formatting, and test commands.
4. Implement a small vertical slice, then test it.
5. Expand functionality in dependency order.
6. Keep CI, documentation, and acceptance criteria aligned with the implementation.
7. Report completed work, evidence, known limitations, and the next concrete task.

Do not attempt to build the entire distribution in one enormous change. Use small, reversible increments. Prefer working, testable foundations over placeholder screens and unimplemented buttons. UI controls must perform their advertised actions or clearly be marked unavailable.

Do not silently change the product scope. If a feature is infeasible with the selected stack, explain the constraint, implement the safest useful fallback, document it, and continue where possible.

## 4. Infrastructure and Repository Organization

Use free resources only. The project owner has no budget for required hosting or CI. Never enable a paid plan or incur a charge. Free tiers have quotas and availability limits; do not promise infinite capacity or uninterrupted service.

The intended repository separation is:
1. `ArchLume` — distribution source, desktop components, packages, ISO profile, installer, update/recovery mechanisms.
2. `ArchLume-Web` — website, downloads/catalog links, user documentation, developer documentation, and AI-agent documentation.
3. `ArchLume-Extensions` — extension SDK, public APIs, templates, examples, and extension-related source.
4. `ArchLume-Repository` — package repository metadata, indices, and published package artifacts where suitable.

Additional repositories may be introduced when a concrete architecture need justifies them. Avoid unnecessary fragmentation. Keep the site, package repository, extension catalog, ISO releases, and update policy loosely coupled so that a failure in one does not disable all others.

Candidate infrastructure includes GitHub repositories, GitHub Actions, GitHub Releases/Pages, and compatible free services such as Vercel. Verify current limits, terms, capabilities, and security properties before relying on a service. Keep the design portable enough to change providers.

If a free quota is exhausted:
- pause affected builds or publications;
- retain already published artifacts wherever the service permits;
- queue work and resume when resources become available;
- use independent free providers or mirrors only where technically and legally permitted;
- never fall back to a paid service automatically;
- never publish an artifact that skipped required checks just to keep a schedule.

Existing releases should remain accessible if CI stops, to the extent supported by the hosting providers. Retention or publication failure must not silently delete existing versions.

## 5. Base System and Target Hardware

### 5.1 Base
- Use official Arch Linux repositories and `pacman`.
- Use `yay` as the intended AUR helper; do not substitute `paru`.
- Use `systemd`.
- Support `x86_64` only initially.
- UEFI is the first-class boot target.
- Official `linux` kernel plus `linux-lts` fallback.
- Rolling-release installed system, plus periodically released, versioned ISO images.
- English is the only initial interface/documentation language. However, design all first-party UI and documentation for future localization: keep translatable strings separate from logic and avoid hard-coded language assumptions. Do not implement additional translations now unless required for architecture testing.

### 5.2 Reference hardware and baseline
Reference machine:
- Lenovo ThinkPad T450;
- Intel Core i5-5300U;
- Intel HD Graphics 5500;
- 8 GB RAM;
- 1366×768 display;
- 221 GiB Btrfs SSD.

Target computers are x86_64 systems with a baseline of 8 GB RAM. Hardware detection and capability-adaptive behavior are required. Do not promise identical support on all hardware; report verified support and known limitations honestly.

### 5.3 Filesystem and boot
Preferred installation defaults:
- Btrfs;
- subvolumes `@`, `@home`, `@snapshots`, `@var_log`, `@var_cache_pacman_pkg`;
- mount options `noatime`, `compress=zstd:1`, `space_cache=v2`;
- Snapper and `snap-pac`;
- rollback integration that is tested end to end.

Use `systemd-boot` as the proposed default bootloader, after verifying hardware and installation constraints. Preserve a safe manual configuration path where practical. The installer must explain unsupported boot configurations rather than pretending they work.

Support optional LUKS2 disk encryption. Encryption defaults to off but should be recommended when appropriate. Integrate the encryption passphrase with login authentication only through a technically sound, supported mechanism. Password changes must update the relevant unlock mechanism safely where supported. Do not give extra users access to encryption keys automatically. Provide recovery guidance and explain recovery-key limitations. Never weaken disk or account security to make login easier.

### 5.4 System services and graphics
Use and verify:
- NetworkManager;
- PipeWire and WirePlumber;
- BlueZ;
- power-profiles-daemon;
- Mesa and Intel graphics support;
- compatible AMD and NVIDIA support where feasible;
- required system services for audio, networking, Bluetooth, power, and session management.

Use standard Arch graphics components with intelligent GPU detection and setup. Detect capabilities, select compatible drivers/settings, provide safe fallbacks, and report diagnostics. Do not promise that every NVIDIA configuration or feature will work without testing.

### 5.5 Arch ISO
Use `archiso` and maintain a reproducible ISO profile. The ISO should support installation and recovery workflows. Build it in CI when free quota allows; do not make successful CI publication a prerequisite for preserving source code or already released artifacts.

## 6. Desktop Architecture

### 6.1 Fundamental requirements
Create an ArchLume-owned Wayland compositor and desktop shell. Existing desktop environments/compositors must not be required or used as the actual desktop, including KDE Plasma, GNOME, Hyprland, Sway, or labwc. Underlying libraries and protocol implementations are allowed; evaluate options such as Smithay and GTK4 based on technical suitability.

Do not write a complete graphics stack from scratch if proven, compatible libraries can provide the foundation. Keep compositor-specific behavior and shell integration under ArchLume control. Support XWayland later or wherever feasible, without making X11 the primary session.

### 6.2 Visual identity and design system
The entire first-party desktop should have a consistent, modern Android-inspired visual language. Modern Android applications such as Seal may be studied for general interaction patterns, layout, color, cards, dialogs, navigation, and microanimations. Create an original ArchLume identity; do not copy branding, assets, or proprietary design wholesale. Do not require strict Material 3 conformity.

Define a reusable design system:
- semantic color roles and states;
- typography and text hierarchy;
- spacing scale and corner radii;
- reusable buttons, cards, dialogs, menus, toggles, lists, and navigation;
- focus, hover, active, disabled, loading, and error states;
- motion and transition guidelines;
- light/dark themes, contrast/accessibility modes, and theme tokens.

All first-party applications should share this design system. Avoid inconsistent one-off styling.

### 6.3 Window management
- Floating windows by default.
- Dragging a window to the top edge maximizes it.
- Dragging to the left or right edge snaps it to half the screen.
- Corner snapping and a visual snap preview may be added where practical.
- Window behavior must be implemented at compositor level, not faked solely in application UI.
- Provide focus, stacking, resize, move, minimize/maximize/close, and keyboard operations with consistent behavior.

### 6.4 Workspaces
- Exactly 10 fixed workspaces.
- `Super + 1` through `Super + 9` select workspaces 1–9.
- `Super + 0` selects workspace 10.
- `Shift + Super + digit` moves the active window to the corresponding workspace and switches to it.
- Workspaces are shared across monitors, not independent per monitor.
- Pressing `Super` alone does nothing; it must not open a launcher or overview.

### 6.5 Keyboard shortcuts and action system
Provide a detailed, expandable keyboard shortcut editor:
- search and filtering;
- recording key combinations;
- conflict detection and warnings;
- reset to defaults;
- profiles and import/export;
- manual configuration;
- readable errors and logs;
- action parameters and reusable actions.

Users may bind shortcuts to permitted commands, scripts, installed programs, sound actions, settings/system actions, or sequences of actions, including scripts or programs without visible windows. Integrate with the same action system where possible.

Do not implement unrestricted privilege escalation. System-level actions must go through appropriate authorization and polkit/system permission boundaries. Explain failed commands, log results, and do not silently execute arbitrary privileged scripts.

### 6.6 Touchpad gestures
Support configurable three- and four-finger gestures where hardware and Wayland input support allow:
- workspace switching;
- overview;
- app menu;
- user-defined actions.

Provide direction, sensitivity, and action settings. Reuse the general action system where possible. Detect capabilities and provide a fallback if hardware or drivers do not support a gesture. Clearly report actual support.

### 6.7 Panel and optional dock
Only the top panel is enabled by default. The dock is optional at installation and can be enabled or disabled in settings.

Top panel can include:
- system menu/application access;
- Wi-Fi/network state;
- audio;
- power/battery;
- notification indicators;
- clock/calendar;
- quick settings;
- workspace and active-window indicators.

Allow configuring panel elements, order/layout, transparency, and behavior.

Optional dock supports position, size, pinned applications, autohide, and animations. Disabling it must not reserve an empty screen area.

### 6.8 Application launcher
Provide an Android-inspired application menu with:
- app grid;
- categories;
- search;
- favorites and recent apps;
- entry points from the panel and configurable hotkeys.

Do not make `Super + Space` a dedicated full-screen launcher. `Super` alone must remain a no-op.

### 6.9 Quick settings and notifications
Quick settings should use a polished hybrid of Android-like cards and toggles for Wi-Fi, Bluetooth, audio, brightness, and power. Include compact dropdowns where appropriate.

Notifications require a separate center/history with grouping and controls. Provide configurable indicators and behaviors. Controls must reflect real system state and handle unavailable hardware gracefully.

### 6.10 Settings and themes
Provide a comprehensive Android-inspired settings application. Theme support should include:
- wallpaper-derived dynamic colors inspired by Material You;
- preset color schemes;
- a full theme editor;
- system-wide light/dark modes;
- contrast/accessibility options.

Do not claim that third-party applications can be forced to use ArchLume themes when they do not support the relevant mechanisms.

### 6.11 Lock and login screens
The lock screen should be customizable and Android-inspired, with a large clock, optional widgets, notifications, media controls, wallpaper, and quick actions. Respect security boundaries; lock-screen actions must not expose protected data.

The login screen should be customizable: wallpaper, clock, animations, layout, account appearance, and security settings. Effects must adapt to hardware. Never bypass authentication or disk encryption.

### 6.12 Multi-monitor
Implement compositor-level multi-monitor support:
- per-monitor resolution, refresh rate, and scaling;
- hotplug;
- saved profiles;
- moving windows between monitors;
- shared workspaces;
- mixed refresh-rate testing;
- restoration after unsupported modes or configuration failures.

Do not lose the active session merely because a requested display mode is unsupported.

## 7. Performance and Reliability

Maximum smoothness and consistency across supported devices are top priorities. Do not optimize only for visual appearance.

- Target the highest stable, hardware-supported refresh rate unless the user overrides it, while respecting power, heat, and stability.
- At 60 Hz, a frame budget is approximately 16.67 ms; at 144 Hz, approximately 6.94 ms.
- Avoid heavy work on the UI/compositor thread. Package operations, filesystem scans, and data fetching must be asynchronous.
- Optimize redraws, GPU usage, memory usage, and animation scheduling.
- Adapt effects to available hardware and accessibility settings.
- Profile every major UI feature and establish measurable performance acceptance criteria.
- Track frame times, dropped frames, input latency, CPU/GPU use, and memory.
- Test on QEMU and the reference ThinkPad T450 where available, at different refresh rates, with mixed monitor modes, and under load.
- Detect performance regressions in CI where practical. Never claim a benchmark passed without actual measurements.

Use the performance-oriented approach associated with projects such as CachyOS as inspiration, not as a reason to blindly copy patches or maximize power use. Start from official Arch, optimize the UI deeply, then adopt kernel/compiler/scheduler/power changes only when evidence shows a benefit and compatibility is maintained. Use universal x86_64 artifacts. Consider LTO/PGO or specialized builds only when measured benefit justifies them and CPU compatibility is not narrowed unintentionally.

## 8. Core Applications

Preinstall only:
- Chromium;
- file manager;
- image viewer;
- video player;
- calculator;
- settings;
- terminal;
- application store.

Other applications should be installed by the user. Include only necessary system services and dependencies.

### 8.1 Terminal
Use a proven terminal component/library; do not build a terminal emulator from scratch. Provide:
- tabs and split panes;
- profiles for Bash, Zsh, and Fish when installed;
- search and history;
- shell settings;
- custom actions and key bindings;
- themes, notifications, and settings integration.

Do not include an AI terminal assistant as a required initial feature.

### 8.2 File manager
Provide a simple Android-inspired interface with advanced features accessible when needed:
- navigation, grid/list views, search, filtering, sorting, previews;
- tabs and split panes;
- archive support;
- network resources and SFTP where practical;
- permissions and safe system-file operations.

Respect file permissions and request authorization when required. Never silently overwrite or delete important files.

### 8.3 Browser
Use Chromium. Integrate with the ArchLume visual language for ArchLume-owned UI, notifications, keyboard shortcuts, default link handling, and settings where technically possible. Preserve Chromium's security model and do not pretend to theme unsupported browser internals.

### 8.4 Audio
Use PipeWire and WirePlumber. Provide per-application volume/routing, virtual devices, profiles, advanced settings, and diagnostics without cluttering the default interface.

Offer a simple microphone-noise-suppression toggle. Verify the availability and compatibility of a filter such as RNNoise before enabling it. Avoid aggressive filtering and distortion; test speech, silence, keyboard noise, and background noise.

### 8.5 Power and hibernation
Provide configurable battery and power profiles, switching behavior on battery/AC, screen timeout, lid/button actions, low-battery behavior, supported charge limits, monitoring, and diagnostics. Support editable profiles, `.sh` scripts and events/hotkeys within safe permission boundaries. Log failures and do not silently run scripts as root.

Hibernation should work by default where hardware and configuration support it. Configure swap/resume where feasible, verify at installation, offer a power-menu entry only when supported, restore the session when possible, and provide diagnostics/fallbacks. Account for hardware, kernel, swap, and encryption constraints; never promise universal support.

## 9. Installer and Initial Setup

Build a graphical installer with an Android-inspired interface. It must:
- detect hardware and recommend kernel, drivers, filesystem, bootloader, and settings;
- allow full manual configuration;
- provide preflight checks and post-install verification;
- explain partitioning, encryption, bootloader, and recovery choices;
- require explicit confirmation before destructive disk operations;
- preserve user data unless the user explicitly authorizes an operation that may destroy it;
- report errors and provide actionable recovery instructions.

Do as much essential configuration as possible during installation. Do not require a mandatory first-run wizard.

Provide optional LUKS2 encryption with the safeguards in section 5. Explain that encryption adds recovery responsibilities.

## 10. Accounts, Ownership, and Permissions

Support multiple user accounts at installation and later. Each account has separate home files, passwords, and settings. Provide GUI account management and login account switching.

### 10.1 Roles
- **OWNER**: created during installation; full authority. Can transfer ownership to another ADMIN. Sensitive ownership operations require OWNER authentication. Other admins cannot modify ownership or bypass OWNER restrictions. After transfer, the former OWNER loses OWNER privileges unless explicitly retained as ADMIN. OWNER can restrict specific actions or applications.
- **ADMIN**: normal administrative permissions, but cannot modify ownership or bypass OWNER restrictions.
- **USER**: ordinary account without administrator privileges.

### 10.2 Permission editor
Provide a detailed GUI permission editor for ADMIN with granular categories/actions, search, profiles, history, and reset-to-defaults. Enforce permissions at the system authorization/polkit layer; never rely on UI hiding alone.

### 10.3 Account creation at login
OWNER may enable account creation from the login screen; disabled by default. Newly created accounts are USER by default. Never automatically grant ADMIN or OWNER. Do not bypass disk encryption or other security controls.

### 10.4 Guest mode
Provide a temporary guest session by default, cleared at exit as far as technically possible. Optionally allow a persistent restricted Guest account. Guest has its own home and minimal permissions, cannot become ADMIN/OWNER, and cannot access other users' private files.

## 11. Home Directories and User Scripts

Create/configure standard user directories:
- `~/Desktop`
- `~/Documents`
- `~/Downloads`
- `~/Music`
- `~/Pictures`
- `~/Videos`
- `~/Scripts`

`~/Scripts` is for user-owned `.sh` scripts used by hotkeys, system events, automations, and custom actions. Configure XDG user directories correctly and preserve existing user directories during upgrades. Explain the folder's purpose at first installation/login without blocking the user; an optional example script may be provided.

Scripts run with the user's permissions by default. Any privileged operation must use explicit authorization and appropriate system mechanisms.

## 12. Updates, Snapshots, and Recovery

### 12.1 Unified Update Center
Use one Update Center for:
- Arch and ArchLume system packages;
- applications and enabled package sources;
- supported AI coding agents such as OpenCode, Claude Code, and Codex CLI.

Do not create a separate agent updater. AI agent installation/update should be integrated into the Update Center.

Automatically check for updates and notify, but install only after the user chooses. Before system updates, create a Btrfs snapshot where technically possible. Provide configurable automatic checks, background checks, checks after reconnecting from a long offline period, and manual checks. Limit traffic and frequency.

Use a cautious rolling-release policy: CI tests critical components and may introduce a short delay where useful. Do not delay security fixes without a sound reason.

### 12.2 Update safeguards
- preflight checks;
- package metadata/dependency verification;
- retain prior package versions where feasible;
- Btrfs snapshots before system updates;
- backup important configuration;
- rollback after failed updates where safely supported;
- disable incompatible extensions when necessary and explain why;
- diagnostics and actionable recovery steps.

A Btrfs snapshot is not a backup. Verify that snapshots include the correct subvolumes and coordinate rollback with bootloader, package cache, and system state. Test rollback end to end.

### 12.3 Recovery environment
Provide:
1. Btrfs snapshots and fast rollback before updates and when appropriate; boot a previous snapshot; automatic rollback after a failed update where reliable; retention based on space.
2. Built-in recovery tools for diagnostics, bootloader repair, filesystem checks, snapshot selection/restoration, and logs.
3. ISO recovery that detects installed systems, mounts partitions, diagnoses and repairs the bootloader/system without reinstalling where possible, preserves data by default, requires explicit confirmation for formatting/reinstallation, and supports unlocking LUKS volumes.

### 12.4 Update sources and trust policy
The update system must distinguish:
- offline state;
- temporary source outage;
- stale metadata;
- invalid signature or suspected tampering.

If the central manifest is unavailable, try independent trusted sources and then the last trusted configuration where safe. Do not install unverified updates. Keep the installed system functional and offer recovery guidance.

Use unified, versioned, signed update policies. The GUI may expose user preferences, but local settings must not silently disable mandatory integrity/security checks.

Provide policy recovery:
- normal CI publication;
- accelerated critical-fix path with signatures and protections;
- rollback to a previous trusted policy;
- independent sources;
- audit log and anti-replay protections.

Do not roll back security patches merely to restore an old configuration.

### 12.5 Severity and notifications
Support configurable event severities:
- informational;
- recommended;
- critical;
- security-critical.

Events should identify affected versions, cause, deadline, deferability, required action, and recovery. A central server outage must not shut down or block ordinary work.

Only short system notifications are required for updates/problems, linking to the Update Center. Technical logs remain available internally; a rich mandatory diagnostics/export UI is not required.

Provide a configurable panel indicator for update/support status. The user may hide the indicator, but hiding it does not disable checks. Critical notifications remain subject to system policy.

## 13. Release Channels and ISO Lifecycle

### 13.1 Channels
Provide three channels:
- **Stable** — default; tested for ordinary users.
- **Testing** — release candidates and pre-release validation.
- **Development** — early, potentially risky changes.

Each channel has its own publishing/support policy. Artifacts must be signed and have checksums. Users choose channels through the Update Center.

Channel switching must be safe and GUI-driven. Moving to Testing/Development requires a warning and compatibility check. Returning to Stable requires package/dependency checks and may require a supported recovery path or reinstall when versions are incompatible. Do not promise safe downgrades where the package system cannot guarantee them.

### 13.2 Stable publication
Automate Stable publication when policy requirements pass. CI builds and tests, checks compatibility and signatures, validates critical components, and runs regression tests. Publish automatically only after required checks succeed. Stop publication on failure, suspected tampering, or incompatibility. Provide a protected accelerated path for critical fixes.

Users still choose when to install updates.

### 13.3 Rolling system and versioned ISO
The installed system is rolling release. Periodically publish versioned ISO images containing current packages, fixes, and installer changes. ISO version/date, checksums, signatures, manifest, and support status must be published. ISO support status is distinct from installed-system support status.

Trigger ISO releases based on readiness and image age:
- release when the previous image is stale or significant changes have accumulated;
- use a minimum interval to conserve free CI quota;
- allow an emergency ISO for a critical fix.

Retain all published ISO versions while free storage permits. Do not automatically delete older ISOs without assessing the consequences. If storage is insufficient, pause or relocate publication to a compatible free source rather than silently deleting releases.

### 13.4 Release documentation
Each ISO release should include:
- ISO image;
- checksum and signature;
- package/version manifest;
- release notes and changes/fixes;
- supported hardware information;
- known issues and limitations;
- installation, diagnostics, and recovery instructions.

Generate a comprehensive report automatically. Full additional documentation-completeness enforcement beyond the standard release checks is not mandatory, but the report must be generated from actual release metadata and must not invent support claims.

## 14. Package Repository, Mirrors, and CI

### 14.1 Package catalog
Manage the official package catalog through GitHub and repository structures/PRs, not a separate GUI administration panel. CI validates metadata, builds, dependencies, compatibility, and basic security, then publishes packages/catalog metadata after success.

`Official` and `Recommended` are separate project-controlled labels. Package authors cannot self-assign those trusted labels.

### 14.2 Mirror resilience
Use several independent package/release/documentation sources where feasible. Implement:
- availability checks;
- automatic fallback;
- package and metadata integrity checks;
- independent backups/mirrors;
- monitoring of source health.

Never disable signature verification during failover. If all sources are unavailable, preserve the working system and ask the user to retry later.

Mirror synchronization uses both publication events and a schedule, plus health and integrity checks. Retry failures with backoff. Preserve the last verified mirror state when synchronization fails. Do not expose incomplete or unverified metadata as a valid current repository.

### 14.3 CI policy
Use a resilient free-CI strategy:
- cache dependencies and artifacts;
- prefer incremental builds where safe;
- prioritize critical checks and security fixes;
- distribute heavy tests across independent free CI providers where technically and legally permitted;
- queue work and pause when all free capacity is exhausted;
- never bypass required checks to publish;
- never use a paid fallback automatically.

### 14.4 Signing and trust
Use a multi-layered signing model:
- separate keys/roles for packages, ISOs, and update policies;
- protected key storage and restricted publication permissions;
- signed metadata and artifacts where supported;
- verification before installation;
- replay protection and trusted metadata freshness;
- documented key rotation, revocation, and recovery procedures;
- a trust-restoration process after key compromise.

Research and implement mechanisms compatible with actual Arch Linux `pacman`, `archiso`, and the chosen CI. Do not invent security guarantees. Never commit private signing keys or plaintext secrets. If a robust offline/root-of-trust key design cannot initially be automated on free CI, document the limitation and design the safest feasible publication workflow.

### 14.5 Versioning and component releases
Use a unified versioning policy:
- versions derived from Git tags and maintained metadata where appropriate;
- component versions and dependencies checked automatically;
- migration compatibility tests;
- generated changelogs;
- prevention of accidental publication of older builds over newer ones.

Use hybrid component releases: compatible components can update independently; the compositor, shell, and critical APIs must pass integration tests as a compatible set. Block incompatible publication except through a controlled critical-fix path with explicit tests and safeguards.

## 15. Application Store

Provide a polished, simple, Android-inspired unified store UI with distinct install mechanisms:
- official Arch packages via `pacman`;
- ArchLume packages via `pacman`;
- AUR packages via `yay`;
- optional Flatpak;
- optional Snap;
- extensions/themes through their own catalog and installation mechanism.

Features:
- search, categories, package details, version, description, source, dependencies, and size where available;
- install/remove/reinstall/update;
- clear source labels distinguishing official packages, AUR, Flatpak, Snap, and extensions;
- official and recommended labels controlled by project policy;
- explicit warnings for AUR/build actions;
- safe confirmation for destructive or dangerous actions.

Check whether `yay` exists and provide a clear installation route if missing. Flatpak and Snap are disabled by default; enabling them through settings should verify or install required components. Disabled sources must not appear in ordinary default results. Clearly label source variants.

User-defined repositories/catalogs may be added, with source/compatibility/permission checks and clear warnings. Do not assume an external source is trustworthy just because it is user-added.

## 16. AI Coding Agents and Skills

Support integration with OpenCode, Claude Code, Codex CLI, and other agents through scripts, commands, and Skills. A mandatory dedicated ArchLume system API/CLI is not required.

Allow choosing a default agent. If it is not installed, tell the user and offer installation or selection of another agent.

Use a unified Skills store with agent-specific adapters:
- central skill storage and version management;
- agent-specific paths and formats;
- GUI search, install, update, remove, enable, and disable;
- dependencies, compatibility, source review, and bundles;
- user-level and system-level installation;
- inspect skills before installation;
- never automatically execute untrusted commands.

Research current documentation for supported agents before implementation. Integrate agent install/update into the unified Update Center; do not create a separate agent updater.

## 17. Extensions, Themes, and SDK

Provide an extension ecosystem with an official catalog and optional third-party repositories. Users may add external sources with warnings, compatibility checks, permission review, and clear trust status.

Define stable, versioned public APIs. Check extension compatibility during updates. Migrate automatically only when safe; otherwise disable incompatible extensions with an explanation and provide rollback where possible. Test API changes in CI.

Support community-published full ArchLume theme packs through official APIs, including themes for panel, launcher, quick settings, lock screen, and other UI surfaces. Theme packs must not gain system privileges simply by being installed.

The project is code/SDK-first, not a visual extension builder. AI agents should be able to create extensions from documented architecture, APIs, tests, and publishing instructions. Provide:
- SDK;
- templates and examples;
- API documentation;
- CI build, test, compatibility, and basic security checks;
- a documented publishing workflow.

Theme and extension catalog entries must distinguish official content from external content. Do not auto-execute untrusted install scripts.

## 18. User Data Backup

In addition to Btrfs snapshots, provide a separate user-data backup system:
- GUI selection of files/folders;
- schedules;
- external-drive targets;
- verification of backup integrity and recoverability;
- clear restore workflow and error messages.

Do not require paid cloud storage. Warn that snapshots stored on the same physical disk do not protect against disk failure. Test actual restore operations, not only backup creation.

## 19. Accessibility

Treat accessibility as a system-level requirement for all first-party components:
- keyboard navigation and visible focus;
- screen-reader/accessibility APIs where the chosen stack supports them;
- contrast themes;
- font and UI scaling;
- reduced-motion mode;
- accessible names, roles, states, and announcements;
- usable focus order and modal behavior;
- automated accessibility checks where practical.

Test accessible states as part of component acceptance. Do not claim support for a screen reader or accessibility API until it has been integrated and verified.

## 20. Licensing and Open Source

The project license is **undecided**. Do not select or invent a project license automatically, and do not publish the project under a random license. Continue private/local development as appropriate, but before public distribution, the owner must make an explicit licensing decision.

Respect all third-party licenses from the beginning. Maintain attribution and notices required by dependencies. A custom license does not remove obligations imposed by third-party components. If the unresolved licensing choice blocks a particular public publication, pause that publication and continue safe work elsewhere.

## 21. Localization

Initial interface and official documentation are English-only. Keep all first-party strings in translation resources or an equivalent localization layer, separate from application logic. Support future translation of:
- desktop and system applications;
- settings, messages, errors, and notifications;
- installer and recovery tools;
- documentation.

Use locale-aware date, time, number, and formatting APIs. Do not add other translations now unless needed to validate the localization architecture.

## 22. Accessibility of Updates, Sources, and Failure States

All major network- or package-dependent features must handle:
- no network;
- slow network;
- unavailable repository;
- stale or invalid metadata;
- insufficient disk space;
- exhausted CI/publication quota;
- interrupted downloads;
- signature mismatch;
- package conflict or dependency failure.

Give the user clear status and safe next steps. Keep ordinary desktop use available when update infrastructure is down. Do not install unverified artifacts, silently downgrade packages, or erase old releases to hide a capacity problem.

## 23. Testing Strategy and Acceptance Criteria

Use a layered test strategy:
- unit tests;
- integration tests;
- package build and dependency checks;
- security and metadata/signature checks;
- ISO build and install tests;
- QEMU boot and installation tests;
- rollback and recovery tests;
- UI behavior and accessibility tests;
- performance benchmarks and regression checks;
- hardware validation on the ThinkPad T450 where available.

Heavy tests should be scheduled and cached to fit free CI constraints, but required checks must not be bypassed for publication.

For each UI feature, define measurable acceptance criteria. At minimum assess frame times, dropped frames, input latency, CPU/GPU use, and memory where relevant. Test 60 Hz and higher supported refresh rates, mixed monitor configurations, and system load where possible.

A verified alpha must have:
- a reproducible source/build workflow;
- a bootable and installable ISO, verified in automated virtualized tests;
- the core desktop session and preinstalled applications functioning;
- package/update workflows with safe failure handling;
- tested snapshot/rollback and recovery flows to the extent implemented;
- security and signature checks for published artifacts;
- generated release documentation;
- automated tests and honest reports of remaining gaps;
- installation and developer documentation;
- a list of known limitations and unverified hardware support.

If a criterion cannot be met due to missing hardware, free infrastructure limits, or a genuine technical blocker, document the evidence and limitation. Do not mark the project fully complete without qualification.

## 24. Milestone Order

Implement in dependency order, adjusting when actual repository constraints require it:

1. **Repository and build foundation** — inspect current source, architecture, CI, test commands, packaging structure, and AGENTS.md; establish project layout and engineering standards.
2. **Base system and reproducible ISO** — Archiso profile, base packages/services, UEFI boot path, install/boot tests, hardware detection.
3. **Package repository and CI** — package metadata, build workflow, versioning, signing design, release artifacts, free-resource fallback.
4. **Wayland compositor foundation** — session startup, outputs, input, surfaces, window management, safe fallback.
5. **Desktop shell** — panel, launcher, quick settings, notifications, workspaces, theming, keyboard action system.
6. **Core applications** — settings, terminal, file manager, calculator, image/video viewer, Chromium integration, store.
7. **Installer and accounts** — graphical installation, partitioning safeguards, encryption, users/roles/permissions.
8. **Update center and recovery** — snapshots, rollback, package safety, recovery environment, channels and policy.
9. **Hardware, power, audio, and multi-monitor** — T450 validation, Intel graphics, PipeWire, gestures, hibernation where supported.
10. **Extensions, Skills, localization foundation, and backup UI** — stable APIs, SDK, catalogs, skill adapters, user-data backups.
11. **Polish and verified alpha** — accessibility, performance profiling, docs, CI matrix, ISO release and known-issues report.

Do not try to implement all desktop features before a bootable base exists. Maintain a working, testable state at each milestone.

## 25. Coding and Repository Hygiene

- Follow the existing language/style where reasonable; introduce new languages/frameworks only for a concrete reason.
- Prefer maintained upstream libraries and protocols over custom reimplementations of complex primitives.
- Keep modules small and APIs explicit.
- Add tests alongside behavior, not after a long implementation phase.
- Use static analysis, formatting, linting, and reproducible build commands where available.
- Avoid hard-coded absolute paths, credentials, private tokens, and machine-specific assumptions.
- Do not commit generated artifacts unless the repository's release process requires them.
- Keep secrets out of logs and reports.
- Use least privilege for CI tokens and system services.
- Never run destructive disk operations on the development machine without explicit authorization.
- Do not assume the reference ThinkPad is always available; use QEMU for repeatable tests and report where real-hardware testing is missing.
- Keep README, architecture documents, developer docs, tests, and this AGENTS.md consistent with actual behavior.

## 26. Final Operating Principle

The agent is authorized to **start implementing ArchLume autonomously after reading this file**. Make reasonable engineering decisions without further approval, test them, and continue through milestones. Ask the owner only for decisions or actions that are genuinely unsafe, irreversible, paid, externally publishing without authorization, or explicitly reserved for the owner.

The owner-selected acceptance target is **C — Verified Alpha**: a bootable and installable system with tested core workflows, measurable performance criteria, security checks, reproducible builds, documentation, and honest reporting of limitations. Do not confuse a prototype, mock UI, successful compilation, or generated ISO file with a verified release.
