#!/usr/bin/env python3
"""
Check that the numbers quoted in the documentation still match the project.

READMEs rot quietly. "58 safety rules" stays in the README long after someone
adds the 59th, and the first person to notice is a reader who now distrusts
everything else on the page. This runs in CI so the drift fails a build
instead.

Run:  python3 tools/check_docs.py
"""
from __future__ import annotations

import collections
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent


def read_json(relative: str):
    return json.loads((ROOT / relative).read_text())


def main() -> int:
    rules = read_json("data/safety-db.json")["rules"]
    by_class = collections.Counter(rule["class"] for rule in rules)
    tweaks = read_json("data/tweaks.json")["tweaks"]

    workspace = (ROOT / "Cargo.toml").read_text()
    msrv_match = re.search(r'rust-version\s*=\s*"([^"]+)"', workspace)
    msrv = msrv_match.group(1) if msrv_match else None

    # (label, value, files that must mention it)
    expectations: list[tuple[str, object, list[str]]] = [
        ("safety rules", len(rules), ["README.md", "docs/README.vi.md", "docs/SAFETY.md"]),
        ("safe rules", by_class["safe"], ["README.md", "docs/README.vi.md"]),
        ("caution rules", by_class["caution"], ["README.md", "docs/README.vi.md"]),
        ("critical rules", by_class["critical"], ["README.md", "docs/README.vi.md"]),
        ("tweaks", len(tweaks), ["README.md", "docs/README.vi.md", "CHANGELOG.md"]),
        ("MSRV", msrv, ["README.md"]),
    ]

    failures: list[str] = []
    for label, value, files in expectations:
        if value is None:
            failures.append(f"{label}: could not be determined from the project")
            continue
        for name in files:
            text = (ROOT / name).read_text()
            if str(value) not in text:
                failures.append(f"{name} does not mention {label} = {value}")
        if not any(f"{label}: could not" in f for f in failures):
            print(f"  ok   {label:<16} {value}")

    # Category coverage: a tweak category with no entries means the UI renders
    # an empty section.
    categories = collections.Counter(t["category"] for t in tweaks)
    empty = [c for c, n in categories.items() if n == 0]
    if empty:
        failures.append(f"tweak categories with no entries: {empty}")

    # Every safety rule needs both translations; the Rust tests check this too,
    # but failing here names the rule without compiling anything.
    for rule in rules:
        reason = rule.get("reason", {})
        if not reason.get("en", "").strip() or not reason.get("vi", "").strip():
            failures.append(f"safety rule `{rule['id']}` is missing a translation")

    if failures:
        print("\ndocumentation is out of date:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        print(
            "\nUpdate the affected files, or the numbers here, so the two agree.",
            file=sys.stderr,
        )
        return 1

    print("  documentation matches the project")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
