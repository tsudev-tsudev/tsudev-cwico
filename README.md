<div align="center">

<a href="https://tsudev.com">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/brand/tsudev-wordmark-dark.png">
    <img src="assets/brand/tsudev-wordmark.png" alt="tsudev" width="320">
  </picture>
</a>

<h1>cwico</h1>

**Deep Windows debloater &amp; software removal toolkit**<br>
*Bộ công cụ rà quét và gỡ bỏ phần mềm Windows chuyên sâu*

[![CI](https://github.com/tsudev-tsudev/tsudev-cwico/actions/workflows/ci.yml/badge.svg)](https://github.com/tsudev-tsudev/tsudev-cwico/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-2482bd)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.82%2B-ef6d18)](https://www.rust-lang.org)
[![Safety rules](https://img.shields.io/badge/safety%20rules-58-2482bd)](data/safety-db.json)
[![Tests](https://img.shields.io/badge/tests-136-2482bd)](#tests)

[tsudev.com](https://tsudev.com) · [Quick start](#quick-start) · [Safety model](#the-safety-model) · [Tiếng Việt](docs/README.vi.md)

</div>

---

## What it does

Windows hides installed software in at least six unrelated places. `cwico`
reads all of them, classifies everything it finds, and lets you remove what you
do not need — without breaking the operating system.

| Pass | What it reads | Win32 / WinRT API |
|---|---|---|
| **Programs** | `DisplayName`, `DisplayVersion`, `Publisher`, `UninstallString`, `QuietUninstallString`, `InstallLocation`, `EstimatedSize`, `InstallDate` | `RegOpenKeyExW` over `HKLM`/`HKCU` `…\Uninstall`, **both** the 64-bit and `WOW6432Node` views |
| **UWP / AppX** | Package full name, family name, publisher, architecture, install path, framework and system flags | `Windows.Management.Deployment.PackageManager` |
| **Provisioned packages** | Packages staged on the image that reinstall for every new user | `FindProvisionedPackages` |
| **Services** | Display name, state, start type, `ImagePath` | `EnumServicesStatusExW`, `QueryServiceConfigW` |
| **Scheduled tasks** | Full path, enabled state, run state | `ITaskService` / `ITaskFolder` COM |
| **Autostart** | `Run`, `RunOnce` (both hives, both views) and the Startup folders | Registry + filesystem |

Removal runs a four-step flow per item:

1. **Terminate** the software's processes — `CreateToolhelp32Snapshot`,
   `OpenProcess`, `TerminateProcess`. Shared hosts (`svchost.exe`) and
   boot-critical processes are never touched.
2. **Run the vendor's uninstaller**, silently. `QuietUninstallString` is
   preferred; otherwise the silent switches are inferred for MSI, Inno Setup
   and NSIS, and only for those.
3. **Remove the package** — `RemovePackageWithOptionsAsync` with
   `RemoveForAllUsers`, then `DeprovisionPackageForAllUsersAsync` so it does
   not come back for the next user account.
4. **Deep clean** — leftover folders and registry keys, each one validated
   against the deletion guard first.

---

## What it looks like

<div align="center">

<img src="docs/screenshots/scan-light-vi.png" alt="The inventory, showing each item's safety class and the reason for it" width="820">

*Every row carries its classification and **why**. A user deciding about
Microsoft Edge should not have to hover to learn that Windows renders PDF
previews with it.*

<br>

<img src="docs/screenshots/plan-light-en.png" alt="The removal plan, listing the backup steps and the exact steps per item" width="820">

*Nothing happens until you have seen the plan: the restore point and `.reg`
export that run first, then the exact steps for each item — and anything the
engine refused, with the reason.*

<br>

<img src="docs/screenshots/caution-dark-vi.png" alt="The per-item confirmation dialog for a Caution item" width="820">

*Ticking a checkbox is not the acknowledgement. `Caution` and `Unknown` items
open this, and it is the only thing in the interface that sets the `confirmed`
flag the engine requires.*

<br>

<img src="docs/screenshots/tweaks-dark-vi.png" alt="The tweaks catalogue" width="820">

*The old PowerShell script's twelve fixed steps, now 36 individually
selectable changes — each with a safety class, a revert path and an
explanation of what it costs.*

<br>

<img src="docs/screenshots/update-gate-vi.png" alt="The mandatory update screen" width="620">

*Publishing a release pushes a mandatory update. There is no dismiss and no
"later" — only Update — because a user on an old build is running an old idea
of what is safe to delete on their machine. A check that fails, though, lets
the app start normally: an outage must not lock everyone out at once.*

<br>

<img src="docs/screenshots/about-light-en.png" alt="The about panel showing safety database rule counts" width="820">

*How much protection is actually loaded, and where your rollback artefacts go.*

<sub>Rendered from the fixture backend (`cwico-core`'s `MockBackend`), not a
live scan — which is also how the interface is developed and reviewed without a
Windows machine in the loop.</sub>

</div>

---

## The safety model

A debloater's failure mode is an unbootable machine, so three independent
layers stand between a user and that outcome.

### 1. Classification — `data/safety-db.json`

Every discovered item is matched against a rule set of **58 rules**:

| Class | Meaning | Count | Examples |
|---|---|---|---|
| **Safe** | No functional impact on Windows | 29 | OneDrive, Xbox app, Candy Crush, Bing News, Skype, telemetry services |
| **Caution** | Removal works, but costs a secondary feature | 11 | Microsoft Edge, Windows Camera, Photos, Media Player, Cortana, Microsoft Store |
| **Critical** | Removal breaks boot, logon, security or the shell | 18 | Defender, File Explorer, Settings, RPC/DCOM/WMI, VC++ and .NET runtimes, device drivers, licensing |
| *Unknown* | Matched no rule | — | Any third-party software the database has not seen |

An item that matches nothing is **`Unknown`, never `Safe`** — the database
fails closed. When an item matches both a `Safe` and a `Critical` rule,
`Critical` wins: severity beats specificity.

### 2. The planning gate — `RemovalPlan::build`

* `Critical` items **cannot be planned at all.** Not with a flag, not with a
  confirmation, not from the CLI. The plan type's constructor drops them and
  records why, and the engine re-checks the invariant before executing.
* `Caution` and `Unknown` items need an **explicit per-item confirmation**.
  A bulk "select all safe" action cannot supply one.
* Everything refused is reported back with a reason. Nothing is dropped
  silently.

### 3. The deletion guard — `cwico_core::guard`

Every filesystem path and registry key is validated immediately before
deletion, because residue paths come from the registry and vendors get them
wrong. Rejected outright:

* drive roots, `C:\Windows`, `System32`, `WinSxS`, `Program Files`,
  `ProgramData`, and anything that is a *parent* of a protected directory
* user profile roots and the user's own folders — `Documents`, `Desktop`,
  `Downloads`, `OneDrive`
* shared containers — `AppData`, `AppData\Local`, `Packages`, `Temp`,
  `Start Menu\Programs`
* every registry hive root, `HKLM\SOFTWARE`, `HKLM\SYSTEM\CurrentControlSet`,
  the services database
* any path still containing an unexpanded `%VARIABLE%`, a `..` traversal, or a
  wildcard

The distinction the guard is careful about: `C:\Users\me\OneDrive` is the
user's synced files and is untouchable, while
`C:\Users\me\AppData\Local\Microsoft\OneDrive` is the client's own state
folder and is exactly what deep clean is for.

### Rollback

Before the first destructive step of any run:

* a **System Restore Point** via `SRSetRestorePointW`
  (`APPLICATION_UNINSTALL`, bracketed by `BEGIN_`/`END_SYSTEM_CHANGE`). If it
  cannot be created, the run **aborts** rather than proceeding unprotected —
  a rollback you cannot perform is not a rollback.
* a **`.reg` export** of every key the run will touch, in both registry views,
  plus a generated `restore-registry.cmd` the user can run without this tool
  installed.
* a **transaction log** (JSON) recording every step, its outcome and its
  artefacts.

---

## Quick start

### Use it

Download the installer from [releases](https://github.com/tsudev-tsudev/tsudev-cwico/releases),
or:

```powershell
winget install tsudev.cwico
```

Run it as Administrator. Without elevation the scan is incomplete and nothing
can be removed.

### Build it

```bash
git clone https://github.com/tsudev-tsudev/tsudev-cwico
cd tsudev-cwico

# Engine tests — run on any host, no flags needed
cargo test

# Type-check the Windows backend from Linux/macOS
rustup target add x86_64-pc-windows-gnu
cargo check -p cwico-win --target x86_64-pc-windows-gnu

# Desktop app (on Windows)
npm --prefix ui install
cargo tauri dev

# Ship it
cargo tauri build            # -> MSI + NSIS installers
```

### Command line

```bash
cwico info                                   # platform, elevation, rule counts
cwico scan --safety safe --locale en         # what is safe to remove
cwico scan --json > inventory.json           # machine-readable audit
cwico plan --safe-only --deep-clean          # what would happen
cwico remove --name OneDrive --deep-clean --apply
```

Everything is a dry run until `--apply`. `--name` matches at word boundaries,
so `--name Edge` selects *Microsoft Edge* and not *Acme Ledger Desktop*.

---

## Architecture

```
tsudev-cwico/
├── crates/
│   ├── cwico-core/          Platform-independent engine. No OS calls at all.
│   │   ├── model.rs           SoftwareItem, SafetyClass, Action
│   │   ├── safety.rs          The classifier
│   │   ├── plan.rs            The planning gate  ← Critical items die here
│   │   ├── guard.rs           The deletion guard ← unsafe paths die here
│   │   ├── engine.rs          The executor
│   │   ├── tweaks.rs          System tweak catalogue
│   │   └── mock.rs            Fixture backend for CI on any host
│   ├── cwico-win/           Windows backend
│   │   ├── registry.rs        RAII RegKey, both 32/64-bit views
│   │   ├── appx.rs            WinRT PackageManager
│   │   ├── services.rs        Service control manager
│   │   ├── tasks.rs           Task Scheduler COM
│   │   ├── process.rs         Toolhelp32 + TerminateProcess
│   │   ├── restore.rs         SRSetRestorePointW
│   │   ├── regbackup.rs       .reg export + rollback script
│   │   ├── deepclean.rs       Guarded deletion
│   │   ├── cmdline.rs         UninstallString parsing   (host-independent)
│   │   ├── naming.rs          Package/task name transforms (host-independent)
│   │   └── protected.rs       Process deny-list          (host-independent)
│   └── cwico-cli/           Headless interface
├── app/src-tauri/           Tauri v2 shell — a thin IPC layer, no logic
├── ui/                      React 19 + TypeScript + Tailwind 4
├── data/
│   ├── safety-db.json         58 classification rules
│   └── tweaks.json            36 system tweaks
└── assets/brand/            Logo and generated icon sets
```

**Why the split.** Everything that decides whether an action is safe lives in
`cwico-core`, which makes no OS calls and therefore runs its full test suite on
a Linux CI runner. The Windows crate is adapters. The Tauri layer is transport.
A bug in the UI cannot talk the engine into doing something dangerous, because
the UI does not get a say.

The three host-independent modules inside `cwico-win` are there for the same
reason: `UninstallString` parsing and package-name derivation are where the
subtle bugs live, so they are pure `std` and covered by tests that run
everywhere.

---

## Tests

```
cargo test
```

**136 tests.** The ones that matter most:

* `selecting_everything_still_cannot_remove_a_critical_component` — ticks every
  row, confirms every prompt, presses go, and asserts Defender and RPC survive.
* `a_required_restore_point_that_fails_aborts_before_anything_is_touched`
* `a_failing_uninstaller_stops_that_item_but_not_the_run` — and specifically
  that deep clean does *not* run after a failed uninstall.
* `a_folder_named_like_a_known_folder_is_still_residue_when_deep_in_appdata`
* `shared_host_processes_are_never_terminated`
* `an_unquoted_program_path_containing_spaces_is_not_split`
* `nsis_uninstall_exe_is_not_mistaken_for_inno_setup`

---

## Roadmap

- [x] Registry, AppX, provisioned, services, tasks, autostart scanning
- [x] Safety database with hard-blocked Critical class
- [x] Restore point + `.reg` rollback + transaction log
- [x] Deep clean with guard rails
- [x] Bilingual desktop app (Vietnamese / English)
- [x] Headless CLI
- [x] MSI and NSIS installers, built and verified on a Windows runner
- [x] MSIX manifest and Store submission notes
- [x] `winget` manifest generated automatically at release time

Auto-update is in place: installed copies check GitHub Releases on startup and
block until a confirmed newer release is installed — see
[`docs/SIGNING.md`](docs/SIGNING.md) for how update payloads are signed.

Ready, but waiting on something only a human can supply:

- [ ] **Runtime testing on real Windows.** Everything compiles and its tests
      pass on a Windows runner, but no version of this has yet driven
      `SRSetRestorePointW` or the service control manager on a live machine.
      Start with `cwico plan`, which changes nothing.
- [ ] **Code signing (Authenticode).** Unsigned installers work, but
      SmartScreen warns on first run — a poor first impression for a tool that
      then asks for Administrator, and it trains users to click through the
      warning that protects them. Needs a certificate from a commercial CA;
      [`docs/SIGNING.md`](docs/SIGNING.md) has the workflow ready to fill in.
      *Update signing is separate and already done.*
- [ ] **Microsoft Store submission.** Needs a Partner Center account; the
      manifest and the reviewer notes are in [`packaging/msix/`](packaging/msix/).
- [ ] **`winget` publication.** Needs a tagged release, then a pull request to
      `microsoft/winget-pkgs` with the generated manifest.

Genuinely future work:

- [ ] Linux port (`cwico-linux`: apt/dnf/flatpak/snap inventory)
- [ ] macOS port

The `PlatformBackend` trait is the seam the other platforms plug into; the
engine, the safety model and the entire UI are already portable.

---

## Contributing to the safety database

`data/safety-db.json` is the most valuable file in this repository, and the
easiest to contribute to. A rule looks like:

```json
{
  "id": "vendor.product",
  "class": "safe",
  "match": { "exact": ["product name"], "kinds": ["registry_uninstall"] },
  "reason": { "en": "Why it is this class.", "vi": "Lý do bằng tiếng Việt." },
  "processes": ["Product.exe"],
  "leftovers": {
    "paths": ["%LOCALAPPDATA%\\Vendor\\Product"],
    "registry": ["HKCU\\Software\\Vendor\\Product"]
  }
}
```

Both `reason` translations are required — a test enforces it. Classify
conservatively: `Caution` costs a user one confirmation click, while a wrong
`Safe` costs them a feature they did not agree to lose.

---

## Licence

MIT © [tsudev](https://tsudev.com)

<div align="center">
<br>
<a href="https://tsudev.com">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/brand/tsudev-wordmark-dark.png">
    <img src="assets/brand/tsudev-wordmark.png" alt="tsudev" width="180">
  </picture>
</a>
</div>
