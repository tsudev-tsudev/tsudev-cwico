# Legacy scripts

These are the original PowerShell scripts this project grew out of. They are
kept for reference - the behaviour they encode is now in the tool proper - and
are **not** used at runtime.

## `Optimize_Win11_For_Dev.ps1`

A 640-line one-click optimiser: twelve fixed groups of changes applied in
sequence with no way to inspect, choose or undo any of them.

Its content lives on as `data/tweaks.json`, where each change became an
individually selectable tweak with:

* a safety class, so `Location tracking off` and `Show file extensions` are no
  longer presented as equivalent choices;
* a revert path, so a registry write can be undone;
* a bilingual explanation of what the change costs, not just what it does.

What was deliberately **not** carried over:

| Original behaviour | Why it was dropped |
|---|---|
| `$ErrorActionPreference = "SilentlyContinue"` at the top | It swallowed every failure. The script reported success whether or not anything worked. |
| Removing `MicrosoftEdge`, `WindowsMediaPlayer` and `Cortana` unconditionally | These are `caution` in the safety database - removable, but only after the user is told what breaks. |
| Disabling `WSearch`, `SysMain` and `PcaSvc` with no warning | Also `caution`: they cost Start-menu search, prefetch tuning and app-compatibility fixes respectively. |
| `Remove-Item $env:TEMP -Recurse -Force` | Now `cwico:clean-temp`, which empties the folder's *contents* and skips files in use, rather than deleting the folder other software expects to exist. |
| Restarting Explorer at the end | Surprising in a tool the user may run mid-work. |
| No restore point | The current engine refuses to run at all if it cannot create one. |

## `RunOptimize.bat`

A UAC self-elevation wrapper. Replaced by the app's own
`relaunch_as_admin` command and, for the CLI, by the installer's elevation
requirement.

---

If you want the old behaviour, the scripts still run. Read them first - that
was always the right advice for a script that reconfigures your machine, and
it is the reason the tool that replaced them shows you a plan before it acts.
