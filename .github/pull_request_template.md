## What this changes

<!-- One or two sentences. The effect, not the mechanism. -->

## Why

<!-- What was wrong, or what became possible. -->

## Checks

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all-targets` - no new warnings
- [ ] `cargo test` - passing
- [ ] `npm --prefix ui run build` - if the front end changed

## Safety

<!-- Delete the lines that do not apply. -->

- [ ] This does not weaken any classification in `data/safety-db.json`.
- [ ] This does not widen what `cwico_core::guard` accepts as a delete target.
- [ ] This does not add a way to act on a `Critical` item.
- [ ] This does not add a new program to the tweak `runCommand` allow-list.

<!-- If any of the above *is* the point of the change, say so here and explain
     the reasoning. Those are reviewable changes, not forbidden ones - they
     just need to be deliberate. -->
