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
| State here | **Done** | **Not done** — needs a certificate |
| Key location | `~/.tsudev-cwico/`, and repository secrets | — |

---

## 1. Update signing — already configured

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

The release workflow fails if no `.sig` files were produced, because an
unsigned release is one that installed copies will refuse — a failure that
would otherwise only surface on a user's machine, days later.

---

## 2. Code signing — what is left to do

Unsigned installers work. What the user sees on first run is:

> **Windows protected your PC**
> Microsoft Defender SmartScreen prevented an unrecognised app from starting.

with the publisher shown as *Unknown*, and a "Run anyway" hidden behind *More
info*. For a tool that then asks for Administrator rights, that is a poor
first impression — and it trains users to click through exactly the warning
that protects them from something worse.

### Choosing a certificate

| Type | Reputation | Cost/year | Notes |
|---|---|---|---|
| **OV** (Organisation Validation) | Builds over time and downloads | lower | SmartScreen still warns until reputation accrues |
| **EV** (Extended Validation) | Immediate | higher | No SmartScreen warning from the first release; requires a hardware token or a cloud HSM |

EV is worth it for this kind of tool: the warning it removes is the one users
will otherwise be told to ignore.

Since June 2023 all publicly trusted code-signing keys must live in certified
hardware, so "download a `.pfx` and put it in a secret" is no longer how this
works. In practice that means a cloud signing service — Azure Trusted Signing,
DigiCert KeyLocker, SSL.com eSigner — driven from CI.

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
  validating the day the certificate expires — including on releases already
  in users' hands.

### Order matters

Code signing must come **after** the Tauri build and **before** the update
signature is used, or the `.sig` will not match the file that ships. The
simplest correct order is: build → Authenticode-sign the artefacts → have
`tauri-action` produce `latest.json` from the signed files.

If you add code signing later, re-run the whole release rather than signing an
already-published artefact in place.

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
- [ ] Code-signing certificate obtained
- [ ] Signing step added to `release.yml`
- [ ] Timestamping configured
- [ ] A test release downloaded on a clean Windows install to confirm no
      SmartScreen warning appears
