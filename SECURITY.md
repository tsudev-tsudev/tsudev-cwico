<div align="center">
<a href="https://tsudev.com"><img src="assets/brand/tsudev-logo.png" alt="tsudev" width="64" height="81"></a>

# Security policy

<a href="https://tsudev.com"><picture><source media="(prefers-color-scheme: dark)" srcset="assets/brand/tsudev-wordmark-dark.png"><img src="assets/brand/tsudev-wordmark.png" alt="tsudev" height="24"></picture></a> · <a href="https://tsudev.com">tsudev.com</a>
</div>

---

## Supported versions

| Version | Supported |
|---|---|
| 1.0.x | ✅ |

## What counts as a security issue here

This tool runs elevated and deletes things. The threat model is unusual, so it
is worth being explicit about what we treat as a vulnerability:

**High priority — please report privately**

* Any way to make the tool remove an item classified `Critical`.
* Any way to make the deletion guard accept a path it should reject — a drive
  root, a system directory, a user's documents, a registry hive root.
* Any way to make the tool execute a program that is not the vendor's own
  uninstaller or an entry on the tweak allow-list. In particular, a crafted
  `UninstallString` that escapes the command parser.
* A path in which the engine performs a destructive step without having first
  created a restore point and the registry backups, when those are enabled.
* Privilege escalation: a way for an unelevated user to make the elevated
  process act on their behalf.
* Anything that lets a malicious `safety-db.json` or `tweaks.json` achieve
  code execution. These files are data and must stay data.

**Also welcome, lower urgency**

* A misclassification in `data/safety-db.json` — something marked `safe` that
  costs the user a feature, or something removable that is over-classified.
  These are correctness bugs rather than vulnerabilities, but they matter.

**Not security issues**

* The tool requiring Administrator rights. That is inherent.
* The tool being able to remove software the user selected. That is the
  product.

## Reporting

Please **do not open a public issue** for anything in the high-priority list.

* GitHub: [private vulnerability reporting](https://github.com/tsudev-tsudev/tsudev-cwico/security/advisories/new)
* Email: security@tsudev.com

Include the item's `id` and classification if the report involves a specific
piece of software, and the exact path or registry key if it involves the guard.
A failing test case is the most useful thing you can send.

Expect an acknowledgement within 72 hours.

## Design commitments

These are properties the project intends to keep. A change that breaks one is
a bug, not a trade-off:

1. **No network access.** The application makes no outbound connections. It
   does not check for updates, report telemetry or fetch rule updates at
   runtime. The only navigation is the user clicking the tsudev logo, which
   opens the product site in their own browser.
2. **`Critical` is absolute.** No flag, configuration file, command-line
   switch or confirmation dialog removes an item classified `Critical`.
3. **Data cannot execute.** `runCommand` tweak effects are restricted to a
   fixed allow-list (`powercfg`, `dism`, `tzutil`, `netsh`) plus internal
   actions implemented in Rust. A poisoned catalogue cannot run arbitrary code.
4. **The web view has no capabilities of its own.** The Tauri capability set
   grants window controls and nothing else — no filesystem, shell or HTTP
   plugin. Everything the front end can do goes through this project's own
   commands, which are individually reviewable.
5. **`open_product_site` takes no URL.** It always opens `https://tsudev.com`.
   A command that accepted an arbitrary URL would be a phishing primitive if
   the web view were ever compromised.
6. **Every destructive run is recoverable.** A restore point and `.reg`
   exports precede the first destructive step, and the run aborts if the
   restore point cannot be created.

## Verifying a release

Release binaries are built by
[GitHub Actions](https://github.com/tsudev-tsudev/tsudev-cwico/blob/main/.github/workflows/release.yml)
from a tagged commit, and each release publishes `sha256sums.txt`.

```powershell
Get-FileHash .\tsudev-cwico_1.0.0_x64_en-US.msi -Algorithm SHA256
```
