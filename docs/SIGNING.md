<div align="center">
<a href="https://tsudev.com"><picture><source media="(prefers-color-scheme: dark)" srcset="../assets/brand/tsudev-wordmark-dark.png"><img src="../assets/brand/tsudev-wordmark.png" alt="tsudev" height="24"></picture></a>

# Signing

<a href="https://tsudev.com">tsudev.com</a>
</div>

---

There are **two unrelated signatures** in this project, and confusing them
wastes a lot of time. One is already in place; the other costs money.

| | Update signing | Code signing (Authenticode) |
|---|---|---|
| What it protects | Installed copies only accept updates you published | Windows and SmartScreen recognise the publisher |
| What it costs | Nothing | An OV or EV certificate from a commercial CA |
| Failure symptom | Updates silently rejected on users' machines | "Windows protected your PC" on first run |
| State here | **Done** | **Not done** - needs a certificate |
| Key location | `~/.tsudev-cwico/`, and repository secrets | - |

---

## 1. Update signing - already configured

The Tauri updater will not install a payload that is not signed by the key
whose public half is baked into the application at build time
(`plugins.updater.pubkey` in `app/src-tauri/tauri.conf.json`).

The keypair was generated with:

```bash
npx @tauri-apps/cli@2 signer generate -w ~/.tsudev-cwico/updater.key
```

* Private key → repository secret `TAURI_SIGNING_PRIVATE_KEY`
* Passphrase → repository secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
* Public key → committed in `tauri.conf.json`
* Both private halves → `~/.tsudev-cwico/`, which is **the only recoverable
  copy**: GitHub secrets are write-only.

### Back it up

If the private key is lost, updates signed with a replacement are rejected by
every copy already installed. Those users are stranded on their current
version permanently; the only remedy is asking each of them to download and
run a new installer by hand. `~/.tsudev-cwico/README.md` says the same thing
next to the key itself.

### Verifying a release actually got signed

Two layers:

1. The release workflow fails if no `.sig` files were produced at all.
2. `tools/verify_update_signature.py` checks the signature verifies against
   the public key **this build carries**, which the first check cannot:

   ```bash
   gh run download <run-id> -n installers -D /tmp/release
   python3 tools/verify_update_signature.py /tmp/release
   ```

The second catches the failure that matters most - a key rotation applied to
the repository secrets but not to `tauri.conf.json`. That produces a
perfectly-signed release that every installed copy shows as a mandatory update
and then refuses to install, which is worse than no release at all.

---

## 2. Code signing - what is left to do

Unsigned installers work. What the user sees on first run is:

> **Windows protected your PC**
> Microsoft Defender SmartScreen prevented an unrecognised app from starting.

with the publisher shown as *Unknown*, and a "Run anyway" hidden behind *More
info*. For a tool that then asks for Administrator rights, that is a poor
first impression - and it trains users to click through exactly the warning
that protects them from something worse.

### Why this matters more here than for most projects

SmartScreen reputation for an **unsigned** binary is tracked *per file hash*.
Every release produces a new hash and starts from zero. This project uses
date-based versioning and expects to ship often, so an unsigned build never
accumulates enough downloads to stop warning before it is superseded - the
warning is effectively permanent rather than a first-release inconvenience.

Signing moves reputation to the *certificate*, so it carries across releases.
That is the actual reason to sign, more than any single warning.

### Getting a certificate for free - the route for this project

**[SignPath Foundation](https://signpath.org/)** issues free OV code-signing
certificates to qualifying open-source projects, with the key held in an HSM
and signing driven from CI. `tsudev-cwico` meets the substantive conditions -
OSI-approved licence (MIT), public repository owned by the maintainers, no
proprietary components - with three things to do first:

1. **Cut a release.** They require an existing released product to sign.
2. **Enable MFA** on the GitHub account, which they require for both SignPath
   and repository access.
3. **Publish a code-signing policy** on the project site and README: SignPath
   Foundation attribution, and the named people in the author / reviewer /
   approver roles.

One condition is worth reading carefully before applying: they exclude
*"hacking tools and active vulnerability scanning features"*. This is a
system-maintenance utility - it removes software the machine's owner selects,
on their own machine, and hard-blocks the components that would make it
dangerous - but a reviewer seeing "terminates processes and deletes registry
keys" may reasonably ask. The safety model in [`SAFETY.md`](SAFETY.md) is the
answer to that question, and worth linking in the application.

### Paid alternatives, if that route does not work out

| Option | Cost | Notes |
|---|---|---|
| [Azure Artifact Signing](https://azure.microsoft.com/en-us/pricing/details/trusted-signing/) (formerly Trusted Signing) | ~$9.99/month, Basic tier, 5,000 signatures | Individual developers accepted; needs 3 years of verifiable identity history |
| Commercial OV certificate | ~$200-400/year | Reputation builds over time |
| Commercial EV certificate | ~$400-700/year | No SmartScreen warning from the first release |

Since June 2023 all publicly trusted code-signing keys must live in certified
hardware, so "download a `.pfx` and put it in a repository secret" is no longer
how any of this works - every option above is a cloud signing service or a
hardware token.

### Wiring it into the release workflow

Once you have a certificate, add the credentials as repository secrets and put
a signing step after the build, before the artefacts are uploaded. With Azure
Trusted Signing:

```yaml
      - name: Sign the installers
        if: startsWith(github.ref, 'refs/tags/')
        uses: azure/trusted-signing-action@v0
        with:
          azure-tenant-id: ${{ secrets.AZURE_TENANT_ID }}
          azure-client-id: ${{ secrets.AZURE_CLIENT_ID }}
          azure-client-secret: ${{ secrets.AZURE_CLIENT_SECRET }}
          endpoint: ${{ secrets.TRUSTED_SIGNING_ENDPOINT }}
          trusted-signing-account-name: ${{ secrets.TRUSTED_SIGNING_ACCOUNT }}
          certificate-profile-name: ${{ secrets.TRUSTED_SIGNING_PROFILE }}
          files-folder: app/src-tauri/target/release/bundle
          files-folder-filter: msi,exe
          file-digest: SHA256
          timestamp-rfc3161: http://timestamp.acs.microsoft.com
          timestamp-digest: SHA256
```

Two things to get right:

* **Sign the `.exe` inside the bundle as well as the installers.** Signing
  only the MSI leaves the application itself unsigned, and SmartScreen
  evaluates what actually runs.
* **Always timestamp.** Without `timestamp-rfc3161`, every signature stops
  validating the day the certificate expires - including on releases already
  in users' hands.

### Order matters

Code signing must come **after** the Tauri build and **before** the update
signature is used, or the `.sig` will not match the file that ships. The
simplest correct order is: build → Authenticode-sign the artefacts → have
`tauri-action` produce `latest.json` from the signed files.

If you add code signing later, re-run the whole release rather than signing an
already-published artefact in place.

---

### What is *not* affected by being unsigned

The SmartScreen warning appears when a file carries the Mark of the Web - the
tag a *browser* attaches to a download. The in-app updater fetches the
installer over HTTP from Rust and runs it, which does not apply that tag, so
an update should install without a warning even while the first manual
download from a browser gets one.

That bounds the problem to first install rather than every release - but it is
reasoning about how Mark of the Web works, not something this project has
observed. It is on the list of things to confirm during the first run on a real
Windows machine.

Installing through `winget` is likewise expected to avoid the prompt, which is
a practical reason to prioritise the winget submission over waiting for a
certificate.

---

## 3. MSIX and the Microsoft Store

Store submissions are signed by Microsoft after review, so no certificate of
your own is needed for the Store build specifically. Sideloaded MSIX packages
do need one. See [`../packaging/msix/README.md`](../packaging/msix/README.md).

---

## Checklist for a signed release

- [x] Updater keypair generated and stored
- [x] `TAURI_SIGNING_PRIVATE_KEY` and `..._PASSWORD` in repository secrets
- [x] Public key committed to `tauri.conf.json`
- [x] Release workflow fails when no `.sig` is produced
- [x] `tools/verify_update_signature.py` checks the signature against the
      key the build carries
- [ ] Code-signing certificate obtained
      (try [SignPath Foundation](https://signpath.org/) first - free for OSS)
- [ ] Signing step added to `release.yml`
- [ ] Timestamping configured
- [ ] A test release downloaded on a clean Windows install to confirm no
      SmartScreen warning appears
