# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning follows [SemVer](https://semver.org/spec/v2.0.0.html).

## [1.0.0] — 2026-08-19

First release. The project was previously a single 640-line PowerShell script
(`legacy/Optimize_Win11_For_Dev.ps1`) that applied twelve fixed groups of
changes with no inspection, no selection and no way back.

### Added — scanning

* Registry uninstall keys across `HKLM`, `HKCU` and **both** the 64-bit and
  `WOW6432Node` views, reading `DisplayName`, `DisplayVersion`, `Publisher`,
  `UninstallString`, `QuietUninstallString`, `InstallLocation`,
  `EstimatedSize` and `InstallDate`.
* UWP/MSIX packages via the WinRT `PackageManager`, including framework and
  system-signature flags.
* Provisioned packages — the ones that reinstall themselves for every new user
  account, which is why removed bloatware "comes back".
* Windows services via `EnumServicesStatusExW` / `QueryServiceConfigW`.
* Scheduled tasks via the `ITaskService` COM interfaces.
* Autostart entries: `Run` and `RunOnce` in both hives and views, plus the
  per-user and all-users Startup folders, with detection of entries pointing
  at programs that no longer exist.

### Added — safety

* **Safety database** (`data/safety-db.json`): 58 rules classifying software
  as Safe (29), Caution (11) or Critical (18), each with a bilingual
  explanation. Unmatched items are `Unknown`, never `Safe`.
* **Hard block on Critical items.** `RemovalPlan::build` refuses to plan them;
  no flag, confirmation or CLI switch overrides it, and the engine re-asserts
  the invariant before executing.
* **Per-item confirmation** for Caution and Unknown items. Bulk selection
  cannot supply it.
* **Deletion guard** (`cwico_core::guard`): validates every filesystem path
  and registry key immediately before deletion. Rejects drive roots, system
  directories, parents of protected directories, user profile roots and
  personal folders, shared containers, every registry hive root, the service
  control database, and any path with an unexpanded `%VARIABLE%`, a `..`
  traversal or a wildcard.
* **Process deny-list**: shared hosts (`svchost.exe`), boot-critical
  processes and Defender are never terminated.

### Added — rollback

* System Restore Point via `SRSetRestorePointW` before the first destructive
  step. If it cannot be created the run is cancelled rather than proceeding
  unprotected, and Windows' 24-hour throttle is detected rather than mistaken
  for success.
* `.reg` export of every registry key a run will touch, in both views, plus a
  generated `restore-registry.cmd` that re-imports them without this tool.
* JSON transaction log per run.

### Added — removal

Four-step flow: terminate the software's processes, run the vendor's own
uninstaller silently, remove and deprovision the package, then sweep residue.
A failed step stops that item without stopping the run, and deep clean never
follows a failed uninstall.

### Added — interfaces

* Desktop application (Tauri v2, React 19, TypeScript, Tailwind 4) in
  Vietnamese and English, with light and dark themes.
* Headless CLI (`cwico`) with JSON output for auditing and scripting.
  Everything is a dry run until `--apply`.
* System tweaks catalogue (`data/tweaks.json`): 36 individually selectable
  changes ported from the legacy script, each with a safety class and — where
  possible — a revert path.

### Packaging

* MSI and NSIS installers via the Tauri bundler.
* MSIX manifest and Microsoft Store submission notes.
* `winget` manifests in English and Vietnamese.
* GitHub Actions for CI (engine tests on Linux, Windows type-checking from
  Linux, front-end build, Windows build) and releases.

### Notes

Behaviour deliberately **not** carried over from the legacy script:
`$ErrorActionPreference = "SilentlyContinue"` swallowing every failure;
unconditional removal of Edge, Media Player and Cortana; silently disabling
Windows Search, SysMain and the Program Compatibility Assistant; deleting the
`TEMP` directory rather than its contents; and restarting Explorer without
asking. See [legacy/README.md](legacy/README.md).
