<div align="center">
<a href="https://tsudev.com"><picture><source media="(prefers-color-scheme: dark)" srcset="../assets/brand/tsudev-wordmark-dark.png"><img src="../assets/brand/tsudev-wordmark.png" alt="tsudev" height="24"></picture></a>

# Releasing

<a href="https://tsudev.com">tsudev.com</a>
</div>

---

Publishing a release here is not only publishing a download. Every installed
copy checks GitHub Releases on startup and **blocks until it has installed
whatever is newest**. Publishing pushes a mandatory update to everyone.

That is deliberate - see [`SAFETY.md`](SAFETY.md) for why a stale safety
database is worth interrupting people over - but it makes the publish step the
one that deserves care.

---

## Version naming

Releases are named for the day they ship:

| Situation | Release name |
|---|---|
| First release on 19 August 2026 | `tsudev-cwico-v26.8.19` |
| A second release the same day | `tsudev-cwico-v26.8.19.2` |
| A third | `tsudev-cwico-v26.8.19.3` |
| The next day | `tsudev-cwico-v26.8.20` |

Internally each maps to a semver whose patch field carries the day and the
release counter (`26.8.1901`), because Cargo, the MSI bundler and the updater
all require three components - and the updater *compares* that number to
decide whether a user is out of date.

Never write a version by hand. `tools/version.py` owns the mapping:

```bash
tools/version.py current      # what the project claims right now
tools/version.py next         # the next release name for today
tools/version.py check        # do all three manifests agree?
```

---

## Cutting a release

### 1. Confirm the tree is green

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
npm --prefix ui run build
python3 tools/check_docs.py --with-tests
python3 tools/test_version.py
```

### 2. Set the version

```bash
tools/version.py set "$(tools/version.py next)"
tools/version.py check
```

This rewrites `Cargo.toml`, `app/src-tauri/tauri.conf.json` and
`ui/package.json`. Nothing else should carry a version.

### 3. Write the changelog entry

Add a section to `CHANGELOG.md` under the new release name. Its body becomes
the release notes users read **inside the mandatory update screen**, so write
it for them rather than for other developers: what changed about what the tool
will and will not remove, and anything they should know before it installs.

The build fails if the section is missing. A blocking screen with no
explanation is worse than no blocking screen - check what it will look like:

```bash
tools/changelog_section.py "$(tools/version.py current | awk '{print $1}')"
```

### 4. Commit and tag

```bash
git add -A
git commit -m "Release $(tools/version.py current | awk '{print $1}')"
git tag "$(tools/version.py current | awk '{print $1}')"
git push origin main --tags
```

The workflow refuses to build a tag whose version disagrees with the
manifests, so a forgotten step 2 fails in CI rather than shipping a build that
reports the previous version - which, since the updater compares exactly that
number, would tell every user they were already current.

### 5. Check the draft

The workflow drafts a release; it does not publish one. The updater endpoint
resolves `releases/latest`, and **GitHub does not count drafts as latest**, so
nothing reaches users yet.

Before publishing, confirm on the draft:

- [ ] `latest.json` is attached - without it no update is ever delivered
- [ ] The MSI and NSIS installers are attached and their sizes look sane
- [ ] The release notes read the way you want them to, in a blocking dialog
- [ ] **The signature verifies against the key this build carries:**

  ```bash
  gh run download <run-id> -n installers -D /tmp/release
  python3 tools/verify_update_signature.py /tmp/release
  ```

  This is the check worth not skipping. A payload signed by the wrong key -
  a rotation applied to the repository secrets but not to `tauri.conf.json` -
  produces a release that every installed copy shows as a mandatory update and
  then refuses to install. Users are stuck behind a wall with a broken button,
  and the only way out is downloading a fresh installer by hand.

- [ ] Ideally: install the MSI on a Windows machine and run it once

### 6. Publish

Publishing is the moment every installed copy starts blocking on this version.
There is no staged rollout - press it when you are confident.

### 7. Update winget

The build attaches `winget-installer-manifest.yaml` with the real SHA256 and
ProductCode. Copy it over
`packaging/winget/manifests/t/tsudev/cwico/<version>/tsudev.cwico.installer.yaml`,
update the other two manifests in that directory, and open a pull request
against [`microsoft/winget-pkgs`](https://github.com/microsoft/winget-pkgs).

---

## Checking packaging without releasing

```bash
gh workflow run release.yml
```

A manual run builds the installers and uploads them as artefacts without
drafting a release or touching any user. Use it after changing anything about
packaging, the bundler configuration or the build hooks - `cargo build` does
not exercise those, so CI's Windows job will not catch a break there.

---

## If a release turns out to be broken

Because updates are mandatory, a broken release reaches everyone who opens the
app. In order of preference:

1. **Fix forward.** Cut the next release the same day -
   `tsudev-cwico-v26.8.19.2` - and publish it. Users who have not yet updated
   go straight to the fixed build; users who did get the fix on next start.
   This is almost always the right answer, and it is what the same-day
   counter exists for.
2. **Unpublish**, if the release is minutes old and the damage is contained.
   Reverting it to a draft removes it from `releases/latest`, so copies that
   have not yet checked stop being told to update. Copies that already
   installed it are not helped.
3. **Never delete a published release's assets** while leaving the release
   itself up. Installed copies then get a `latest.json` pointing at a file
   that no longer exists, and the mandatory gate becomes a wall with a broken
   button behind it.

The blast radius of a bad release is the reason the workflow drafts rather
than publishes, and the reason step 5 asks you to install it once by hand.
