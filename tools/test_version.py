#!/usr/bin/env python3
"""
Tests for tools/version.py.

The version encoding is load-bearing in a way that is easy to overlook: the
updater *compares* versions to decide whether a user is out of date, so a
mapping that round-trips but sorts wrongly would silently stop delivering
updates — or push users backwards. Ordering is therefore tested explicitly,
not just conversion.

Run:  python3 tools/test_version.py
"""
from __future__ import annotations

import importlib.util
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("version", ROOT / "tools" / "version.py")
version = importlib.util.module_from_spec(spec)
spec.loader.exec_module(version)

#: Shared with cwico-core's version tests. Changing the mapping in one language
#: without the other fails a test in the language that was not changed.
CASES = json.loads((ROOT / "data" / "version-cases.json").read_text())

failures: list[str] = []


def check(label: str, condition: bool) -> None:
    print(f"  {'ok  ' if condition else 'FAIL'} {label}")
    if not condition:
        failures.append(label)


def semver_key(s: str) -> tuple[int, int, int]:
    major, minor, patch = (int(part) for part in s.split("."))
    return major, minor, patch


def test_round_trip() -> None:
    for case in CASES["roundTrip"]:
        name, semver = case["name"], case["semver"]
        check(
            f"{name} <-> {semver}",
            version.to_semver(name) == semver and version.to_name(semver) == name,
        )


def test_ordering() -> None:
    """The updater compares these, so they must sort in release order."""
    semvers = [version.to_semver(name) for name in CASES["ascendingOrder"]]
    check("release order survives the mapping", semvers == sorted(semvers, key=semver_key))

    # The specific trap: the tenth release of a day must sort above the second,
    # which naive string comparison gets wrong.
    check(
        "the 10th release of a day sorts above the 2nd",
        semver_key(version.to_semver("tsudev-cwico-v26.8.19.10"))
        > semver_key(version.to_semver("tsudev-cwico-v26.8.19.2")),
    )
    # And a new day must beat any number of same-day releases.
    check(
        "a new day beats the 99th release of the previous one",
        semver_key(version.to_semver("tsudev-cwico-v26.8.20"))
        > semver_key(version.to_semver("tsudev-cwico-v26.8.19.99")),
    )


def test_accepted_input_forms() -> None:
    for short in ["v26.8.19", "26.8.19", "tsudev-cwico-v26.8.19"]:
        check(f"accepts `{short}`", version.to_semver(short) == "26.8.1901")


def test_rejected_names() -> None:
    for bad in CASES["rejectedNames"]:
        try:
            version.to_semver(bad)
            check(f"rejects `{bad}`", False)
        except version.VersionError:
            check(f"rejects `{bad}`", True)


def test_rejected_semvers() -> None:
    """A semver from outside this scheme must fail loudly, not mis-decode."""
    for bad in CASES["rejectedSemvers"]:
        try:
            version.to_name(bad)
            check(f"rejects semver `{bad}`", False)
        except version.VersionError:
            check(f"rejects semver `{bad}`", True)


def test_manifests_agree() -> None:
    check("every manifest carries the same version", version.cmd_check() == 0)


def main() -> int:
    for test in [
        test_round_trip,
        test_ordering,
        test_accepted_input_forms,
        test_rejected_names,
        test_rejected_semvers,
        test_manifests_agree,
    ]:
        print(f"\n{test.__name__}")
        test()

    print()
    if failures:
        print(f"{len(failures)} check(s) failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("all version checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
