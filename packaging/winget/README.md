# winget manifests

These are the manifests submitted to
[`microsoft/winget-pkgs`](https://github.com/microsoft/winget-pkgs) so that
users can install with:

```powershell
winget install tsudev.cwico
```

## The installer manifest is generated, not written

`tsudev.cwico.installer.yaml` in each version directory carries an
`InstallerSha256` and a `ProductCode`. **Both change with every build** -
Tauri's builds are not reproducible, and two runs of the same commit have
produced different values for each. Transcribing them by hand is how a
manifest ends up pointing at a different binary than the one it names.

So they are produced by the release build and folded in with a tool:

```bash
gh run download <release-run-id> -n installers -D /tmp/release
python3 ../../tools/apply_winget_manifest.py /tmp/release/winget-installer-manifest.yaml
```

That creates the version directory if it is new, copying the locale manifests
forward and rewriting the version and URLs in them.

## Submitting

1. Publish the GitHub release first - winget's validation downloads the
   installer from the URL in the manifest, so it has to be reachable.
2. Verify the manifests locally on a Windows machine if you can:

   ```powershell
   winget validate --manifest packaging\winget\manifests\t\tsudev\cwico\<version>
   winget install --manifest packaging\winget\manifests\t\tsudev\cwico\<version>
   ```

3. Fork `microsoft/winget-pkgs`, copy the version directory to
   `manifests/t/tsudev/cwico/<version>/`, and open a pull request.

   `wingetcreate submit` does the same thing in one command if you have it.

## What their validation does

The winget pipeline installs the package in a clean Windows VM and checks it
launches. That is genuinely useful here: it is an automated smoke test on real
Windows, which is the one thing this project's own CI cannot do.

It also means a broken installer fails validation rather than reaching users -
but it consumes maintainer attention, so it is worth having installed the MSI
by hand at least once first.

## Why `winget` matters more than usual for this project

Installers are currently unsigned, so a browser download triggers SmartScreen.
Installing through `winget` is expected to avoid that prompt, because winget
does not apply the Mark of the Web that SmartScreen keys off. Until a code
signing certificate is in place (see [`../../docs/SIGNING.md`](../../docs/SIGNING.md)),
winget is the least alarming way for someone to install this.
