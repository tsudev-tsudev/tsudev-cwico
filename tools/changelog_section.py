#!/usr/bin/env python3
"""
Extract one release's section from CHANGELOG.md.

The text this prints becomes the GitHub release body, which `tauri-action`
copies into `latest.json`, which the application displays **inside the
mandatory update dialog**. A blocking screen with no explanation is a worse
experience than no blocking screen at all, so this is not cosmetic.

Usage:
    tools/changelog_section.py                      # the current version
    tools/changelog_section.py tsudev-cwico-v26.8.19
"""
from __future__ import annotations

import importlib.util
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CHANGELOG = ROOT / "CHANGELOG.md"

spec = importlib.util.spec_from_file_location("version", ROOT / "tools" / "version.py")
version = importlib.util.module_from_spec(spec)
spec.loader.exec_module(version)


def section_for(release: str) -> str:
    """The body of `## <release> — ...`, up to the next `##` heading."""
    text = CHANGELOG.read_text()
    pattern = re.compile(
        rf"^##\s+{re.escape(release)}\b.*?$(.*?)(?=^##\s|\Z)",
        re.MULTILINE | re.DOTALL,
    )
    match = pattern.search(text)
    if not match:
        raise SystemExit(
            f"no `## {release}` section in CHANGELOG.md — "
            "add one before tagging, or the update dialog will block users "
            "without telling them why"
        )
    body = match.group(1).strip()
    if not body:
        raise SystemExit(f"the `## {release}` section is empty")
    return body


def main(argv: list[str]) -> int:
    if argv:
        release = argv[0]
    else:
        release = version.to_name(version.read_cargo())
    # Accept a bare tag, a full name, or a semver.
    release = version.to_name(version.to_semver(release))
    print(section_for(release))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
