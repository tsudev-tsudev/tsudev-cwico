# Project state

> **This is the file to read first.** It says what is true right now, what is
> in flight, and what to do next. Everything else in `docs/sessions/` is
> history. See [`README.md`](README.md) for the conventions.

**Last updated:** 2026-08-19 · session `2026-08-19-03`
**Branch:** `main` · **Remote:** https://github.com/tsudev-tsudev/tsudev-cwico

---

## In one paragraph

`tsudev-cwico` is a deep Windows debloater. The engine (`cwico-core`) makes no
OS calls and holds all the safety logic; `cwico-win` is Win32/WinRT adapters;
`app/src-tauri` is IPC plumbing; `ui/` is React. It scans registry uninstall
keys, AppX packages, provisioned packages, services, scheduled tasks and
autostart entries, classifies everything against a 58-rule safety database,
and refuses outright to remove anything classified `Critical`. Installed
copies check for updates on startup and block until a confirmed newer release
is installed. The current version is `tsudev-cwico-v26.8.19` (semver
`26.8.1901`). It builds, tests and packages successfully on Windows via CI.
**Nobody has yet run it on a live Windows machine.**

---

## Verify the project is where this file says it is

Run these before trusting anything below. They take about three minutes.

```bash
cargo fmt --all -- --check          # expect: silent
cargo clippy --all-targets -- -D warnings
cargo test                          # expect: 136 passing
cargo check -p cwico-win --target x86_64-pc-windows-gnu
npm --prefix ui run build
python3 tools/check_docs.py --with-tests
python3 tools/test_version.py
```

The last two are self-checking: they fail if the numbers quoted in the
READMEs, the version encoding, or the three manifests have drifted.

---

## What is done and verified

| Area | State | How it was verified |
|---|---|---|
| Engine, safety database, planner, guard | Done | 136 tests, incl. adversarial ones |
| Windows backend (`cwico-win`) | Compiles, unit tests pass | CI Windows runner, MSVC |
| Desktop app + CLI | Done | CI + headless-Chromium screenshots |
| MSI / NSIS installers | Done | Release workflow; artefacts downloaded and inspected |
| winget manifest generation | Done | `tools/winget-manifest.ps1`, verified twice |
| Docs, CI, issue templates, security policy | Done | CI green |
| CalVer release naming | Done | `tools/test_version.py` + Rust tests on shared vectors |
| Updater signing key | Done | In `~/.tsudev-cwico/` and repository secrets |
| Mandatory update gate | Done | CI Windows build; UI screenshots of both states |
| Signed update payloads | Done | Signature cryptographically verified against the app's public key |

## What is *not* verified

* **The application has never run on a live Windows machine.**
  `SRSetRestorePointW`, `EnumServicesStatusExW`, `ITaskService` and
  `RemovePackageWithOptionsAsync` compile and are covered by unit tests of
  their pure logic, but no code path has executed against a real registry or a
  real service control manager.

  The one exception, and it is worth knowing: winget's validation pipeline
  installed the MSI in a clean Windows VM and passed. So the *installer* works
  on real Windows. Nothing beyond that is evidenced.
  First thing to do on a Windows box: `cwico info`, then `cwico scan`, then
  `cwico plan --safe-only` — none of which change anything.
* Installers are **unsigned**. SmartScreen will warn on first run.

---

## What to do next

Everything planned so far is finished and released. The work now splits three
ways, and **the first group is the only part a session working in this
repository can actually do** — the other two need the maintainer or a Windows
machine.

### A · Available now, no Windows machine and no permissions needed

Ordered by value.

1. **Add rules to `data/safety-db.json`.** This is the highest-value work left
   and it is pure data. Every item that matches no rule resolves to `Unknown`,
   which pushes the judgement onto a user who has less context than the
   database could. Gaps worth closing, roughly in order of how often they
   appear on a real machine:
   * OEM preloads by vendor — HP, Dell, Lenovo, Acer, Asus, MSI each ship a
     dozen named utilities, and the current rules only catch the generic ones.
   * Common third-party software that people *do* want to remove and that has
     residue worth sweeping — Adobe Creative Cloud components, Java, Zoom,
     Discord, Steam, Epic, iTunes, Nvidia GeForce Experience.
   * Windows 11 24H2/25H2 packages this database predates.

   `CONTRIBUTING.md` has the rule format. Classify one step stricter than you
   think: a wrong `caution` costs a click, a wrong `safe` costs a feature the
   user never agreed to lose. Run `cargo test -p cwico-core --features mock
   safety::` after.

2. **Extract more pure logic out of `cwico-win`.** The three host-independent
   modules (`cmdline`, `naming`, `protected`) each turned up a real bug the
   moment their tests could run. `registry.rs` value decoding, `startup.rs`
   target resolution and `scanner.rs` item-id construction are the same shape
   of code and are currently only type-checked.

3. **Accessibility in the item table.** It is the surface users spend their
   time in, it is a checkbox list that drives destructive actions, and it has
   never been driven by keyboard alone or read by a screen reader.

4. **Grow `data/tweaks.json`.** Same shape of work as the safety database,
   lower stakes.

5. **Start `cwico-linux`.** `PlatformBackend` is the seam; `MockBackend` shows
   the shape. apt/dnf/flatpak/snap inventory would come first. Large, and
   nothing else depends on it.

### B · Blocked on the maintainer

1. **Check the CLA state on winget PR #420321.** The `license/cla` check says
   requirements are met but the `Needs-CLA` label is still applied. If it is
   genuinely unsigned, only the account owner can sign it.
2. **Submit the SignPath application** (deferred deliberately: internal testing
   first). Text is written; see below.

### C · Blocked on a Windows machine

1. **Run it.** `cwico info`, `cwico scan`, `cwico plan --safe-only` — none
   change anything. This is the single largest gap in the project.
2. **Test the update path end to end.** Needs two published releases: install
   `v26.8.19`, publish a second, watch the gate appear and the installer hand
   over. The UI states and the signatures are verified; the handoff is not.

---

### Release `tsudev-cwico-v26.8.19` — published

https://github.com/tsudev-tsudev/tsudev-cwico/releases/tag/tsudev-cwico-v26.8.19

The mandatory-update mechanism is now live: the endpoint installed copies poll
returns HTTP 200 with a `latest.json` carrying version `26.8.1901`, three
platform entries and 4,982 characters of release notes. Both payload signatures
verify against the public key compiled into the application.

The release notes state plainly that this has never run on a live Windows
machine, and point at the three commands that change nothing.

**The next release is what tests the update path.** Publishing it will make
every installed copy of `v26.8.19` show the blocking gate. Before doing that,
read `docs/RELEASING.md`, and ideally install this release on a Windows machine
first so there is something to update *from*.

### winget — submitted, automated validation passed

[microsoft/winget-pkgs#420321](https://github.com/microsoft/winget-pkgs/pull/420321)

Labels as of close of session: `Azure-Pipeline-Passed`, `Policy-Test-2.7`,
`Needs-CLA`, `New-Package`, `Validation-Guide`.

**`Azure-Pipeline-Passed` matters.** Their pipeline installs a new package in a
clean Windows VM. It is the closest thing to a real-Windows smoke test this
project has had — the MSI installs. It does *not* say the application was
launched, that it read a registry, or that anything was removed.

The other labels, read carefully:

* **`Needs-CLA`** — but the `license/cla` check on the head commit reports
  *"All CLA requirements met"*. The label and the check disagree; the label is
  probably just stale. **Confirm on the pull request page** before acting on
  either.
* **`Policy-Test-2.7`** is the *adult content* review policy, applied in the
  same second as `Azure-Pipeline-Passed`. Almost certainly routine routing for
  a new package rather than a finding — the manifests were checked for the
  usual false-positive substrings and contain only `do**cum**entations`,
  `cl**ass**ified` and `s**hell**`.
* **`Validation-Guide`** came with a generic bot comment ("there was an issue
  validating") and no specifics. Nothing actionable was stated.

**Next action:** watch the pull request and respond to whatever a moderator
asks. Two checklist boxes were deliberately left unchecked — `winget validate`
and `winget install` cannot be run from this development host — with an
explanation in the pull request body of what was verified instead. If a
moderator asks for them, that needs a Windows machine.

### SignPath — prepared, not submitted

`packaging/signpath/APPLICATION.md` has the application text, including the
argument for why this is not a hacking tool (their terms exclude those, and a
summary of this tool's capabilities reads like one at a glance).
`docs/CODE-SIGNING-POLICY.md` is the publicly-visible policy they require.

Deferred by the maintainer: internal testing first, code signing later.

### Next

**On a Windows machine, the things nothing has verified:**

1. That the scanner reads a real registry, service control manager and task
   scheduler correctly. Start with `cwico info`, `cwico scan`, then
   `cwico plan --safe-only` — none of which change anything.
2. That `download_and_install` genuinely replaces the running process and
   restarts into the new version. The UI states are verified; the handoff to
   the Windows installer is not. This needs two releases to test — install
   `v26.8.19`, then publish a second one.
3. Sign the Microsoft CLA so winget#420321 can proceed.

### Decisions taken for this work — do not re-litigate

1. **Version naming.** Release name `tsudev-cwico-v26.8.19`; a second release
   the same day is `…-v26.8.19.2`. Cargo, MSI and the updater all require
   three-component semver, so the display name maps to a semver where
   `patch = day × 100 + release-of-day`:

   | Release name | semver |
   |---|---|
   | `tsudev-cwico-v26.8.19` | `26.8.1901` |
   | `tsudev-cwico-v26.8.19.2` | `26.8.1902` |
   | `tsudev-cwico-v26.8.20` | `26.8.2001` |
   | `tsudev-cwico-v26.9.1` | `26.9.101` |

   Ordering is correct in every case. `tools/version.py` converts both ways
   and is the only thing that should ever compute a version.

2. **Update failures fail open.** The app blocks only when a newer version is
   *confirmed*. Network error, DNS failure, GitHub outage → the app starts
   normally and shows a quiet "could not check" line. A server outage must
   never brick the tool on every user's machine at once.

3. **Update metadata lives on GitHub Releases** (`latest.json`), which the
   Tauri updater reads directly. No separate hosting.

4. **The updater signing key** is generated by the maintainer and stored in
   repository secrets. It is *not* the same thing as an Authenticode
   certificate: it signs update payloads so the app can verify them, and does
   nothing about SmartScreen.

---

## Environment notes — things that waste an hour if you do not know them

* **`cargo test` needs no feature flags.** A non-Windows build pulls
  `cwico-core/mock` in through a `[target.'cfg(not(windows))'.dependencies]`
  entry, so the fixture backend is automatic.
* **`cargo test --workspace` fails on Linux/macOS** — it drags in the Tauri
  crate, which needs webkit2gtk. `default-members` deliberately excludes it.
  Use plain `cargo test`.
* **The Tauri crate can no longer be cross-checked from Linux.** Since the
  updater plugin was added it pulls in `ring`, whose build script needs a C
  compiler for the target (`x86_64-w64-mingw32-gcc`), and before that it
  needed `x86_64-w64-mingw32-windres`. Install both to restore it:
  `apt install binutils-mingw-w64-x86-64 gcc-mingw-w64-x86-64`. Without them,
  **CI's Windows job is the only thing that type-checks `cwico-app`** — push
  and read the result rather than assuming. `cwico-core`, `cwico-win` and
  `cwico-cli` still cross-check fine, and they hold all the logic.
* **Paths in `tauri.conf.json` have two different bases.** `frontendDist` is
  relative to `tauri.conf.json` (`app/src-tauri/`); the `before*Command` hooks
  run from the *app directory* one level up (`app/`). That is why one says
  `../../ui/dist` and the other `../ui`. Getting it wrong fails only under
  `cargo tauri build`, which CI's Windows job does not run — the release
  workflow does.
* **Tauri builds are not reproducible.** Two runs of the same commit produced
  different MSI hashes *and* different ProductCodes, so a winget manifest is
  only valid for the exact build it was generated from. The one checked in for
  `26.8.1901` carries the real values from the published release;
  `tools/apply_winget_manifest.py` folds a new build's in and refuses a
  placeholder hash.
* **The release workflow can be run manually** (`gh workflow run release.yml`)
  to build installers without drafting a release. Use it to check packaging
  without cutting a version.

---

## Repository map

```
crates/cwico-core/     engine — no OS calls, all the safety logic, 136 tests
  safety.rs              the classifier
  plan.rs                the planning gate — Critical items die here
  guard.rs               the deletion guard — unsafe paths die here
crates/cwico-win/      Win32/WinRT adapters
  cmdline.rs naming.rs protected.rs   host-independent, tested everywhere
crates/cwico-cli/      headless interface
app/src-tauri/         IPC only, no logic
ui/                    React 19 + TS + Tailwind 4
data/safety-db.json    58 rules — the most consequential file here
data/tweaks.json       36 system tweaks
tools/
  version.py             the ONLY thing that computes a version
  test_version.py        + Rust tests, both on data/version-cases.json
  check_docs.py          fails CI when the READMEs' numbers drift
  changelog_section.py   release notes -> the update dialog
  verify_update_signature.py   will installed copies accept this update?
  apply_winget_manifest.py     folds a build's manifest in, checks URL + hash
  winget-manifest.ps1    reads SHA256 and ProductCode from the MSI itself
  gen_icons.py gen_wordmark.py   brand assets from assets/brand/
packaging/             MSIX manifest + Store notes, winget manifests
docs/sessions/         you are here
```

---

## If you are picking this up cold

1. Read this file (done).
2. Run the verification block above.
3. Read [`docs/SAFETY.md`](../SAFETY.md) — it is the design rationale for
   everything that stops this tool breaking someone's machine, and changing
   the safety layer without reading it is how that protection gets eroded.
4. Pick something from **What to do next → A**. That section is ordered by
   value, and everything in it can be done from this repository alone.
5. Before you stop — including if you are running out of context — update this
   file first, then write the session log. If there is only time for one, make
   it this file: a missing log costs the next session context, a stale
   `STATE.md` costs them a wrong decision.
