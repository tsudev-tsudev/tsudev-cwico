#!/usr/bin/env python3
"""
The single place that knows how tsudev-cwico versions work.

Releases are named by date: the release published on 19 August 2026 is
`tsudev-cwico-v26.8.19`, and a second release the same day is
`tsudev-cwico-v26.8.19.2`.

That four-component name is not valid semver, and Cargo, the MSI bundler and
the Tauri updater all require semver - the updater in particular *compares*
versions to decide whether an update exists, so the encoding has to sort
correctly. The name therefore maps to a semver where the patch field carries
both the day and the release-of-day counter:

    patch = day × 100 + n            (n starts at 1)

    tsudev-cwico-v26.8.19     <->  26.8.1901
    tsudev-cwico-v26.8.19.2   <->  26.8.1902
    tsudev-cwico-v26.8.20     <->  26.8.2001
    tsudev-cwico-v26.9.1      <->  26.9.101

Ordering holds in every direction: 1901 < 1902 < 2001, and a September release
sorts above any August one on the minor field. Up to 99 releases a day.

Usage
    tools/version.py current              print the version the project claims
    tools/version.py next                 compute today's next release name
    tools/version.py to-semver <name>     tsudev-cwico-v26.8.19 -> 26.8.1901
    tools/version.py to-name <semver>     26.8.1901 -> tsudev-cwico-v26.8.19
    tools/version.py set <name|semver>    write it to every manifest
    tools/version.py check                verify every manifest agrees
"""
from __future__ import annotations

import datetime as dt
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
PRODUCT = "tsudev-cwico"

#: Files that carry the version, and how to read/write it.
CARGO_TOML = ROOT / "Cargo.toml"
TAURI_CONF = ROOT / "app" / "src-tauri" / "tauri.conf.json"
UI_PACKAGE = ROOT / "ui" / "package.json"

SEMVER_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")
NAME_RE = re.compile(
    rf"^(?:{re.escape(PRODUCT)}-)?v?(\d+)\.(\d+)\.(\d+)(?:\.(\d+))?$"
)

MAX_RELEASES_PER_DAY = 99


class VersionError(ValueError):
    """A version string that cannot be interpreted."""


# ---------------------------------------------------------------------------
# Conversion
# ---------------------------------------------------------------------------


def parse_name(name: str) -> tuple[int, int, int, int]:
    """`tsudev-cwico-v26.8.19.2` -> (26, 8, 19, 2). The counter defaults to 1."""
    match = NAME_RE.match(name.strip())
    if not match:
        raise VersionError(
            f"`{name}` is not a release name; expected {PRODUCT}-vYY.M.D[.N]"
        )
    year, month, day, counter = match.groups()
    n = int(counter) if counter else 1
    if not 1 <= n <= MAX_RELEASES_PER_DAY:
        raise VersionError(f"release counter {n} is outside 1..{MAX_RELEASES_PER_DAY}")
    if not 1 <= int(month) <= 12:
        raise VersionError(f"month {month} is out of range")
    if not 1 <= int(day) <= 31:
        raise VersionError(f"day {day} is out of range")
    return int(year), int(month), int(day), n


def to_semver(name: str) -> str:
    """Release name -> the semver Cargo, the MSI and the updater use."""
    year, month, day, n = parse_name(name)
    return f"{year}.{month}.{day * 100 + n}"


def parse_semver(semver: str) -> tuple[int, int, int, int]:
    """`26.8.1902` -> (26, 8, 19, 2)."""
    match = SEMVER_RE.match(semver.strip())
    if not match:
        raise VersionError(f"`{semver}` is not a three-component semver")
    year, month, patch = (int(g) for g in match.groups())
    day, n = divmod(patch, 100)
    if not 1 <= day <= 31:
        raise VersionError(
            f"patch {patch} decodes to day {day}, which is not a calendar day - "
            "was this version produced by something other than tools/version.py?"
        )
    if n == 0:
        raise VersionError(
            f"patch {patch} decodes to release-of-day 0; the counter starts at 1"
        )
    return year, month, day, n


def to_name(semver: str) -> str:
    """The semver -> the release name users see."""
    year, month, day, n = parse_semver(semver)
    suffix = "" if n == 1 else f".{n}"
    return f"{PRODUCT}-v{year}.{month}.{day}{suffix}"


# ---------------------------------------------------------------------------
# Reading and writing the manifests
# ---------------------------------------------------------------------------


def read_cargo() -> str:
    match = re.search(
        r'^\[workspace\.package\](?:.|\n)*?^version\s*=\s*"([^"]+)"',
        CARGO_TOML.read_text(),
        re.MULTILINE,
    )
    if not match:
        raise VersionError("no [workspace.package] version in Cargo.toml")
    return match.group(1)


def read_tauri() -> str:
    return json.loads(TAURI_CONF.read_text())["version"]


def read_ui() -> str:
    return json.loads(UI_PACKAGE.read_text())["version"]


def write_cargo(semver: str) -> None:
    text = CARGO_TOML.read_text()
    new, count = re.subn(
        r'(^\[workspace\.package\](?:.|\n)*?^version\s*=\s*")[^"]+(")',
        rf"\g<1>{semver}\g<2>",
        text,
        count=1,
        flags=re.MULTILINE,
    )
    if count != 1:
        raise VersionError("could not rewrite the version in Cargo.toml")
    CARGO_TOML.write_text(new)


def write_tauri(semver: str) -> None:
    # Rewritten as text rather than re-serialised, to keep key order and the
    # comments-in-strings this file relies on intact.
    text = TAURI_CONF.read_text()
    new, count = re.subn(r'("version"\s*:\s*")[^"]+(")', rf"\g<1>{semver}\g<2>", text, count=1)
    if count != 1:
        raise VersionError("could not rewrite the version in tauri.conf.json")
    json.loads(new)  # fail loudly rather than writing broken JSON
    TAURI_CONF.write_text(new)


def write_ui(semver: str) -> None:
    text = UI_PACKAGE.read_text()
    new, count = re.subn(r'("version"\s*:\s*")[^"]+(")', rf"\g<1>{semver}\g<2>", text, count=1)
    if count != 1:
        raise VersionError("could not rewrite the version in ui/package.json")
    json.loads(new)
    UI_PACKAGE.write_text(new)


SOURCES = [
    ("Cargo.toml", read_cargo, write_cargo),
    ("app/src-tauri/tauri.conf.json", read_tauri, write_tauri),
    ("ui/package.json", read_ui, write_ui),
]


# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------


def cmd_current() -> int:
    semver = read_cargo()
    print(f"{to_name(semver)}   (semver {semver})")
    return 0


def existing_tags() -> list[str]:
    try:
        out = subprocess.run(
            ["git", "tag", "--list", f"{PRODUCT}-v*"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    except (subprocess.CalledProcessError, FileNotFoundError):
        return []
    return [line.strip() for line in out.splitlines() if line.strip()]


def cmd_next(today: dt.date | None = None) -> int:
    """The next release name for today, accounting for releases already cut."""
    today = today or dt.date.today()
    year, month, day = today.year % 100, today.month, today.day

    used = set()
    for tag in existing_tags():
        try:
            t_year, t_month, t_day, n = parse_name(tag)
        except VersionError:
            continue
        if (t_year, t_month, t_day) == (year, month, day):
            used.add(n)

    n = 1
    while n in used:
        n += 1
    if n > MAX_RELEASES_PER_DAY:
        print(
            f"error: {MAX_RELEASES_PER_DAY} releases already published today",
            file=sys.stderr,
        )
        return 1

    suffix = "" if n == 1 else f".{n}"
    name = f"{PRODUCT}-v{year}.{month}.{day}{suffix}"
    print(name)
    return 0


def cmd_set(value: str) -> int:
    semver = value if SEMVER_RE.match(value) else to_semver(value)
    parse_semver(semver)  # reject anything the decoder cannot read back
    for label, _read, write in SOURCES:
        write(semver)
        print(f"  {label:<34} {semver}")
    print(f"\n{to_name(semver)}")
    return 0


def cmd_check() -> int:
    values = {}
    problems = []
    for label, read, _write in SOURCES:
        try:
            values[label] = read()
        except Exception as error:  # noqa: BLE001 - report, do not crash
            problems.append(f"{label}: {error}")

    distinct = set(values.values())
    if len(distinct) > 1:
        problems.append("the manifests disagree:")
        for label, value in values.items():
            problems.append(f"    {label:<34} {value}")

    for label, value in values.items():
        try:
            parse_semver(value)
        except VersionError as error:
            problems.append(f"{label}: {error}")

    if problems:
        print("version check failed:", file=sys.stderr)
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        print(
            "\nRun `tools/version.py set <release-name>` to bring them back in line.",
            file=sys.stderr,
        )
        return 1

    semver = next(iter(distinct))
    print(f"  every manifest agrees: {to_name(semver)}  (semver {semver})")
    return 0


def main(argv: list[str]) -> int:
    if not argv:
        print(__doc__.strip())
        return 1

    command, *rest = argv
    try:
        if command == "current":
            return cmd_current()
        if command == "next":
            return cmd_next()
        if command == "to-semver":
            print(to_semver(rest[0]))
            return 0
        if command == "to-name":
            print(to_name(rest[0]))
            return 0
        if command == "set":
            return cmd_set(rest[0])
        if command == "check":
            return cmd_check()
    except VersionError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    except IndexError:
        print(f"error: `{command}` needs an argument", file=sys.stderr)
        return 1

    print(f"error: unknown command `{command}`", file=sys.stderr)
    print(__doc__.strip(), file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
