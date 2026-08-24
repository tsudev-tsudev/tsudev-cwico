# Microsoft Store submission notes

## Why this package needs `runFullTrust` and `allowElevation`

Store reviewers ask about both. The honest answer:

| Capability | Used for | Why nothing narrower works |
|---|---|---|
| `runFullTrust` | Reading `HKLM\SOFTWARE\...\Uninstall` in both registry views; `EnumServicesStatusExW` and `ChangeServiceConfigW`; `ITaskService`; `SRSetRestorePointW` | An AppContainer cannot read HKLM, cannot open the service control manager, and cannot create a restore point. There is no brokered API for any of them. |
| `allowElevation` | Relaunching with a UAC prompt when the user started the app unelevated | Removing machine-wide software and creating a restore point both require an elevated token. The app runs unelevated by default and only asks when the user chooses to act. |

## What the app does **not** do

Worth stating in the submission notes, because a debloater looks alarming
from the outside:

* **No network access of its own.** No telemetry, no update check, no
  analytics. The only outbound navigation is the user clicking the tsudev
  logo, which opens `https://tsudev.com` in their default browser.
* **Nothing is removed without an explicit action.** There is no "optimise
  now" button, no default selection, and no background operation.
* **Critical system components cannot be removed at all.** 18 rules in
  `data/safety-db.json` classify Defender, the shell, core services, shared
  runtimes and drivers as Critical, and `RemovalPlan::build` refuses to plan
  them. This is not a warning the user can click through.
* **Every run is reversible.** A System Restore Point is created first, and
  the run aborts if it cannot be. Every registry key touched is exported to a
  `.reg` file alongside a `restore-registry.cmd` the user can run without
  this app installed.

## Building the MSIX

`cargo tauri build` produces the MSI and NSIS installers. For the Store
package:

```powershell
# 1. Build the release binary and front end
cargo tauri build --no-bundle

# 2. Stage the package layout
mkdir -Force packaging/msix/staging/Assets
Copy-Item target/release/cwico.exe        packaging/msix/staging/
Copy-Item -Recurse data                   packaging/msix/staging/data
Copy-Item app/src-tauri/icons/Square*.png packaging/msix/staging/Assets/
Copy-Item app/src-tauri/icons/StoreLogo.png packaging/msix/staging/Assets/
Copy-Item packaging/msix/AppxManifest.xml packaging/msix/staging/

# 3. Pack and sign
makeappx pack /d packaging/msix/staging /p build/tsudev-cwico-v26.8.19-x64.msix
signtool sign /fd SHA256 /a /f cert.pfx /p $env:CERT_PASSWORD `
  build/tsudev-cwico-v26.8.19-x64.msix
```

`Publisher` in `AppxManifest.xml` must match the certificate subject exactly,
and for a Store submission it must match the publisher identity Partner Center
assigns - replace `CN=tsudev` with that value before packing.

## Age rating and category

* Category: **Utilities & tools**
* Age rating: 3+ - no user-generated content, no network, no purchases.
