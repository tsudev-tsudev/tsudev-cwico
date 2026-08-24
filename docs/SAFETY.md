<div align="center">
<a href="https://tsudev.com"><img src="../assets/brand/tsudev-logo.png" alt="tsudev" width="72" height="91"></a>

# The safety model

<a href="https://tsudev.com"><picture><source media="(prefers-color-scheme: dark)" srcset="../assets/brand/tsudev-wordmark-dark.png"><img src="../assets/brand/tsudev-wordmark.png" alt="tsudev" height="24"></picture></a> · <a href="https://tsudev.com">tsudev.com</a>
</div>

---

A debloater's failure mode is a machine that will not boot. This document is
the design rationale for everything that stands between a user and that
outcome, so that changes to the safety layer are made deliberately rather than
by accident.

## The three layers

They are independent on purpose. Any one of them failing still leaves two.

```
        user selects items
                │
   ┌────────────▼─────────────┐
   │ 1. SafetyDatabase        │  classify: Safe / Caution / Critical / Unknown
   │    safety.rs             │  fails closed - unmatched is Unknown, not Safe
   └────────────┬─────────────┘
                │
   ┌────────────▼─────────────┐
   │ 2. RemovalPlan::build    │  Critical → refused, always
   │    plan.rs               │  Caution/Unknown → needs per-item confirmation
   └────────────┬─────────────┘
                │
   ┌────────────▼─────────────┐
   │ 3. guard::validate_*     │  every path and key, immediately before deletion
   │    guard.rs              │  drive roots, system dirs, user data, hive roots
   └────────────┬─────────────┘
                │
          restore point
          .reg backups
          transaction log
                │
             execute
```

## Layer 1 - classification

`data/safety-db.json` holds 58 rules. Each carries a class, matching criteria,
a bilingual reason, and optional knowledge about the product's processes,
services, tasks and residue.

### The four classes

| Class | Test | User experience |
|---|---|---|
| `safe` | Windows boots, logs in and works identically without it | Selectable in bulk |
| `caution` | Windows still works, but a feature the user might rely on is gone | Needs a per-item confirmation dialog that names what is lost |
| `critical` | Windows fails to boot, fails to log in, loses security, or loses the shell | Not selectable. No checkbox is rendered. |
| `unknown` | No rule matched | Treated as `caution` for gating, but labelled distinctly |

### Two invariants

**Fail closed.** An item that matches nothing is `Unknown`. It is tempting to
default unmatched third-party software to `Safe` - most of it is - but the one
time it is a line-of-business application, the user finds out afterwards.

**Severity beats specificity.** If an item matches a broad `Safe` rule and a
narrow `Critical` rule, the verdict is `Critical`. The classifier collects
every match, takes the highest severity, and only then uses specificity to
pick which rule's *explanation* to show. A more specific rule can never
downgrade a protection.

### Choosing a class

The question is not "does this break anything" but **"what does the user lose,
and would they have predicted it?"**

* Microsoft Edge is `caution`, not `safe`: Windows renders PDF previews, help
  pages and some Settings panes with it. It is also not `critical` - the
  machine boots and works fine, and people remove it deliberately every day.
* Windows Camera is `caution`: other apps still reach the webcam, but Windows
  Hello enrolment stops working. That is a surprise worth one dialog.
* The Xbox Game Bar is `safe`, but the Xbox **Identity Provider** is `caution`
  - Game Pass titles fail to sign in without it. Splitting a product family
  across two classes is normal and correct.
* Shared runtimes (VC++, .NET, WebView2) are `critical` even though removing
  one does not stop the machine booting. They break *other* software, silently
  and later, which is worse than an obvious failure.

When in doubt, classify one step stricter. A wrong `caution` costs the user a
click. A wrong `safe` costs them something they never agreed to lose.

## Layer 2 - the planning gate

`RemovalPlan::build` is the only way to produce a plan, and it is where
selections become work. Three things happen there:

1. **`Critical` items are dropped.** Not warned about - dropped, with a
   `protected_component` rejection the UI displays. There is no flag, no CLI
   switch and no confirmation that overrides this. `assert_no_protected_items`
   re-checks the invariant immediately before execution, so a future refactor
   that reintroduced the hole would abort the run rather than proceed.
2. **`Caution` and `Unknown` need `confirmed: true`**, which only the
   per-item dialog sets. "Select all safe" cannot set it, by construction.
3. **Steps are ordered.** Services and scheduled tasks are quiesced before the
   programs that own them; AppX removal precedes deprovisioning; deep clean is
   last.

### Services are disabled, never deleted

A service's registry key *is* the service. `Uninstall` on a service means stop
and set `StartType = Disabled` - one call to undo. Deep clean deliberately
does not apply to services, scheduled tasks or autostart entries; the guard
also refuses `HKLM\SYSTEM\CurrentControlSet\Services` at every depth, so the
two layers cover each other.

### A failed step stops that item

If the vendor's uninstaller fails, the deep-clean steps for that item do not
run. Sweeping a product's folders after a failed uninstall is how a machine
ends up with a half-removed application that can neither run nor be
reinstalled.

## Layer 3 - the deletion guard

Residue paths come from the registry, which means they come from vendors,
which means some of them are wrong. `InstallLocation = C:\` ships in real
products. `guard::validate_delete_path` runs on every path immediately before
deletion and rejects:

* drive roots and the system directories - `C:\Windows`, `System32`,
  `SysWOW64`, `WinSxS`, `Program Files`, `ProgramData`, `WindowsApps`
* **any path that is a parent of a protected directory**, which catches
  intermediates nobody thought to list
* user profile roots, and everything under `Documents`, `Desktop`,
  `Downloads`, `Pictures`, `OneDrive`
* shared containers - `AppData`, `AppData\Local`, `Packages`, `Temp`,
  `Start Menu\Programs`
* anything with an unexpanded `%VARIABLE%`, a `..` traversal, a wildcard, or a
  UNC prefix

The unexpanded-variable rule is load-bearing: if `%LOCALAPPDATA%` failed to
expand, `%LOCALAPPDATA%\Vendor\App` must not quietly become `\Vendor\App`.
Environment expansion deliberately leaves unknown variables intact so the
guard trips on them.

### The distinction that matters

```
C:\Users\me\OneDrive                            <- the user's synced files: never
C:\Users\me\AppData\Local\Microsoft\OneDrive    <- the client's own state: yes
```

A folder *named* OneDrive is not the thing to protect; a known folder directly
under a profile is. Getting this wrong in either direction is a real bug - too
strict and deep clean does nothing useful, too loose and it deletes someone's
documents.

## Rollback

Before the first destructive step:

1. **System Restore Point** - `SRSetRestorePointW` with
   `APPLICATION_UNINSTALL`, bracketed by `BEGIN_`/`END_SYSTEM_CHANGE`.
   If it fails and `require_restore_point` is set (the default), the run is
   **cancelled**. Windows also throttles restore points to one per 24 hours by
   default; a throttled call returns success with sequence number 0, which the
   code detects and reports rather than treating as protection.
2. **`.reg` export** of every key the run will touch, in both registry views,
   via `reg.exe export` - a text file the user can read and re-import from
   Explorer, rather than a binary hive only this tool understands. A generated
   `restore-registry.cmd` re-imports them all.
3. **Transaction log** - JSON, one file per run, recording every step, its
   status, its duration and the artefacts it touched.

## Processes that are never terminated

Step one of removal terminates the software's processes. Two groups are exempt
regardless of what any rule says:

* **Shared hosts** - `svchost.exe`, `dllhost.exe`, `RuntimeBroker.exe`,
  `taskhostw.exe`. Killing `svchost.exe` stops a dozen unrelated services.
* **Boot and session critical** - `lsass.exe`, `csrss.exe`, `wininit.exe`,
  `winlogon.exe`, `services.exe`, `smss.exe`, `dwm.exe`, `explorer.exe`.
* **Security** - `MsMpEng.exe`, `SecurityHealthService.exe`. Terminating these
  disables protection mid-run.

A telemetry service running inside `svchost.exe` is stopped through the
service control manager instead, which is the correct mechanism anyway.

## Testing the safety layer

The tests that matter are the adversarial ones:

| Test | What it proves |
|---|---|
| `selecting_everything_still_cannot_remove_a_critical_component` | Ticking every row, confirming every prompt and pressing go leaves Defender and RPC untouched |
| `critical_beats_safe_when_both_match` | A specific rule cannot downgrade a protection |
| `unmatched_software_is_unknown_not_safe` | The database fails closed |
| `a_required_restore_point_that_fails_aborts_before_anything_is_touched` | No rollback means no run |
| `a_failing_uninstaller_stops_that_item_but_not_the_run` | Deep clean never follows a failed uninstall |
| `deep_clean_never_deletes_a_service_definition` | Services survive deep clean |
| `a_folder_named_like_a_known_folder_is_still_residue_when_deep_in_appdata` | Both directions of the OneDrive distinction |
| `shared_host_processes_are_never_terminated` | `svchost.exe` survives |

Run them with:

```bash
cargo test
```

---

<div align="center">
<a href="https://tsudev.com"><img src="../assets/brand/tsudev-logo.png" alt="tsudev" width="40" height="50"></a><br>
<a href="https://tsudev.com"><picture><source media="(prefers-color-scheme: dark)" srcset="../assets/brand/tsudev-wordmark-dark.png"><img src="../assets/brand/tsudev-wordmark.png" alt="tsudev" height="24"></picture></a>
</div>
