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
| Engine, safety database, planner, guard | Done | 128 tests, incl. adversarial ones |
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

* **Nothing has ever run on a live Windows machine.** `SRSetRestorePointW`,
  `EnumServicesStatusExW`, `ITaskService` and `RemovePackageWithOptionsAsync`
  compile and are covered by unit tests of their pure logic, but no code path
  has executed against a real registry or a real service control manager.
  First thing to do on a Windows box: `cwico info`, then `cwico scan`, then
  `cwico plan --safe-only` — none of which change anything.
* Installers are **unsigned**. SmartScreen will warn on first run.

---

## In flight — the current work

Restructuring for release management and auto-update. Tasks, in order:

- [x] 11 · Session log system (`docs/sessions/`)
- [x] 12 · CalVer versioning + `tools/version.py`
- [x] 13 · Updater signing keypair into GitHub Secrets
- [x] 14 · `tauri-plugin-updater` integration + IPC commands
- [x] 15 · Mandatory update screen in the UI
- [x] 16 · Release workflow publishing a signed `latest.json`
- [x] 17 · Authenticode documentation, docs refresh

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

### winget — submitted, waiting on a signature

[microsoft/winget-pkgs#420321](https://github.com/microsoft/winget-pkgs/pull/420321),
labelled `Needs-CLA`.

**Only the account owner can clear it:** Microsoft's bot requires
[@tsudev-tsudev](https://github.com/tsudev-tsudev) to sign the
[Contributor License Agreement](https://cla.opensource.microsoft.com). Nothing
else in the pull request can proceed until that is done — their validation
pipeline does not run beforehand.

Two checklist boxes were deliberately left unchecked, because `winget validate`
and `winget install` cannot be run from this development host. What was
verified instead is written into the pull request body.

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
  different MSI hashes *and* different ProductCodes. The checked-in winget
  manifest is therefore a placeholder; the real one is generated per build.
* **The release workflow can be run manually** (`gh workflow run release.yml`)
  to build installers without drafting a release. Use it to check packaging
  without cutting a version.

---

## Repository map

```
crates/cwico-core/     engine — no OS calls, all the safety logic, 128 tests
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
tools/                 icon/wordmark generation, doc checks, winget manifest
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
4. Pick up the first unchecked task in **In flight**.
5. Before you stop, update this file.
