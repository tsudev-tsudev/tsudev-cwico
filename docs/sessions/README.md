# Session log

This directory exists so that work on this project survives an interruption —
a machine that shuts down mid-task, a session that runs out of context, or
simply a gap of several weeks between one person picking it up and the next.

## Read this first

**[`STATE.md`](STATE.md)** is the canonical answer to *where is this project
right now*. It is the only file that must always be current. If you are
starting a session, read it before anything else; if you are ending one,
update it before you stop.

Everything else in this directory is history. History is useful for
understanding *why* something is the way it is, but `STATE.md` is what tells
you what to do next.

## Layout

```
docs/sessions/
├── README.md                    this file — the conventions
├── STATE.md                     where the project is. Always current.
└── 2026-08-19-01.md             one file per working session, in order
```

Session files are named `YYYY-MM-DD-NN.md`, where `NN` counts sessions on that
date starting at `01`. They are append-only once written: a later session
corrects the record in `STATE.md`, not by rewriting yesterday's log.

## What goes where

| | `STATE.md` | a session log |
|---|---|---|
| Purpose | What is true now | What happened, and why |
| Tense | Present | Past |
| Lifetime | Rewritten every session | Written once, never edited |
| Answers | "What do I do next?" | "Why is this like this?" |

## Starting a session

1. Read `STATE.md`, top to bottom. It is deliberately short enough to read.
2. Run the verification block it lists. If something fails that `STATE.md`
   claims passes, that is your first task — and it means the previous session
   ended without verifying, which is itself worth recording.
3. Create `docs/sessions/YYYY-MM-DD-NN.md` from the template below.

## Ending a session — including an unplanned one

Update `STATE.md` **first**, then the session log. If you only have time for
one, make it `STATE.md`: a missing session log costs the next person context,
a stale `STATE.md` costs them a wrong decision.

Record work in progress honestly. "Half-finished, the tests do not compile"
is far more useful to the next session than silence, and infinitely more
useful than a claim that something is done when it is not.

## Session log template

```markdown
# Session YYYY-MM-DD-NN

**Started:** <state at the beginning — the commit, what STATE.md said>
**Goal:** <what this session set out to do>

## What happened

<Chronological. Decisions and their reasons, not just a list of commits —
`git log` already has the commits.>

## Decisions

<Anything a future session would otherwise re-litigate. Include what was
rejected and why; that is usually the more valuable half.>

## Problems found

<Bugs, wrong assumptions, things that turned out harder than expected.>

## Left undone

<Be specific and honest. "The updater UI renders but downloadAndInstall is
untested on a real machine" beats "updater mostly done".>

## Verification at the end of this session

<Exact commands and their results.>
```
