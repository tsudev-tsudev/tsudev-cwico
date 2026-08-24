# SignPath Foundation application

Prepared text for the application at <https://signpath.org/apply>, kept in the
repository so that what was claimed is on the record and a future maintainer
can see what was agreed to.

**Status:** not yet submitted. See the checklist at the end.

---

## Project

**Name:** tsudev-cwico
**Repository:** https://github.com/tsudev-tsudev/tsudev-cwico
**Website:** https://tsudev.com
**Licence:** MIT (OSI-approved, no commercial dual-licensing)
**Language:** Rust, TypeScript
**Platform:** Windows 10 2004+ / Windows 11

## What it does

A Windows debloater and software-removal tool. It reads the six places Windows
keeps installed software - the registry uninstall keys in both the 32- and
64-bit views, UWP/MSIX packages, packages provisioned on the image, services,
scheduled tasks and autostart entries - classifies everything it finds against
a bundled safety database, and removes what the machine's owner selects.

## Why this is not a hacking tool

The Foundation's terms exclude *"hacking tools and active vulnerability
scanning features"*. This project contains neither, and the distinction is
worth stating plainly because a summary of its capabilities -
*terminates processes, deletes registry keys, reconfigures services* - reads
like one at a glance.

* **It operates only on the machine it runs on**, on software the machine's
  owner has selected in a list, with Administrator rights that the owner
  granted. There is no remote capability of any kind. It has no network
  functionality beyond checking for its own updates.
* **It scans for installed software, not for vulnerabilities.** It reads
  inventory the operating system already exposes through documented APIs
  (`RegOpenKeyExW`, `EnumServicesStatusExW`, `PackageManager`, `ITaskService`).
  There is no port scanning, no fingerprinting, no exploit database, no
  network discovery.
* **It refuses to do the damaging thing.** 18 rule groups classify Windows
  Defender, the shell, core services, shared runtimes, device drivers, the
  boot loader and the licensing stack as `Critical`, and the removal planner
  will not plan a `Critical` item at all - not with a flag, not from the
  command line, not with a confirmation dialog. Every filesystem path and
  registry key is validated before deletion against a guard that rejects drive
  roots, system directories, user data folders and hive roots.
* **It is designed to be reversible.** A System Restore Point and `.reg`
  exports precede the first destructive step, and the run is cancelled rather
  than continued if the restore point cannot be created.

The design rationale for all of this is documented at
[`docs/SAFETY.md`](../../docs/SAFETY.md), which is the most useful thing to
read when assessing whether this project is what it says it is.

The nearest comparable published tools are Microsoft's own
`Remove-AppxPackage`, the Sysinternals `Autoruns` utility, and third-party
uninstallers such as Revo or BCUninstaller. This is in that category.

## Eligibility

| Condition | Evidence |
|---|---|
| OSI-approved licence, no dual-licensing | [MIT](../../LICENSE); GitHub reports SPDX `MIT` |
| No proprietary components | All dependencies are open source; `Cargo.lock` and `package-lock.json` are committed |
| Actively maintained | Public commit history |
| Released product | Release `tsudev-cwico-v26.8.19` |
| Functionality documented on the download page | [README](../../README.md), with screenshots |
| Team owns the repository | Yes; not a fork |
| Signs only its own binaries | The release workflow builds from a tag on `main` and signs nothing else |
| Build scripts reviewed like source | Stated in the [code-signing policy](../../docs/CODE-SIGNING-POLICY.md) |
| MFA on repository and signing service | Enabled |
| Author / reviewer / approver roles named publicly | [code-signing policy](../../docs/CODE-SIGNING-POLICY.md) |
| Code-signing policy published | [`docs/CODE-SIGNING-POLICY.md`](../../docs/CODE-SIGNING-POLICY.md) |
| Consistent product name and version metadata | Set from `tauri.conf.json`; the release workflow refuses a tag whose version disagrees with the manifests |
| No hacking tools or vulnerability scanning | See above |

## Why signing matters unusually much here

Two reasons beyond the usual:

1. **This tool asks for Administrator rights.** Training its users to click
   through "Windows protected your PC" is training them to dismiss exactly the
   warning that would protect them from something malicious wearing the same
   name.
2. **SmartScreen reputation for an unsigned binary is tracked per file hash.**
   This project uses date-based versioning and expects to ship corrections to
   its safety database promptly, so an unsigned build never accumulates enough
   reputation before it is superseded. Unsigned, the warning is permanent
   rather than a first-release inconvenience - and because updates are
   mandatory, users cannot opt out of receiving the new hash.

## Build pipeline

GitHub Actions, `.github/workflows/release.yml`, on `windows-latest`.
Triggered by a tag matching `tsudev-cwico-v*`. Produces an MSI (WiX) and an
NSIS installer via the Tauri bundler, plus a CLI binary.

A signing step would be inserted after the build and before the artefacts are
uploaded; [`docs/SIGNING.md`](../../docs/SIGNING.md) describes where.

---

## Before submitting

- [ ] Confirm MFA is enabled on the GitHub account
      (Settings → Password and authentication)
- [ ] Confirm the release is published, not a draft
- [ ] Re-read [`docs/CODE-SIGNING-POLICY.md`](../../docs/CODE-SIGNING-POLICY.md)
      and correct anything that has drifted
- [ ] Submit at <https://signpath.org/apply>

## After acceptance

- [ ] Add the SignPath Foundation attribution to the code-signing policy
- [ ] Add the signing step to `release.yml`
- [ ] Cut a release and confirm no SmartScreen warning on a clean Windows
      install
