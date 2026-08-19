#!/usr/bin/env python3
"""
Fold a release build's generated winget manifest into the repository.

The release workflow produces `winget-installer-manifest.yaml` carrying the
real SHA256 and ProductCode — both change with every build, because Tauri's
builds are not reproducible. This copies it into the versioned manifest
directory and creates that directory's other two manifests if the version is
new, so submitting to winget is not an exercise in careful transcription.

    gh run download <run-id> -n installers -D /tmp/release
    python3 tools/apply_winget_manifest.py /tmp/release/winget-installer-manifest.yaml

Then review the diff and open a pull request against microsoft/winget-pkgs.
"""
from __future__ import annotations

import argparse
import importlib.util
import pathlib
import re
import shutil
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
MANIFEST_ROOT = ROOT / "packaging" / "winget" / "manifests" / "t" / "tsudev" / "cwico"

spec = importlib.util.spec_from_file_location("version", ROOT / "tools" / "version.py")
version = importlib.util.module_from_spec(spec)
spec.loader.exec_module(version)


def read_field(text: str, field: str) -> str | None:
    match = re.search(rf"^\s*{field}:\s*'?\"?([^'\"\n]+)'?\"?\s*$", text, re.MULTILINE)
    return match.group(1).strip() if match else None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.strip().splitlines()[0])
    parser.add_argument(
        "manifest",
        type=pathlib.Path,
        help="the winget-installer-manifest.yaml a release build produced",
    )
    args = parser.parse_args()

    if not args.manifest.is_file():
        sys.exit(f"not a file: {args.manifest}")

    generated = args.manifest.read_text()
    semver = read_field(generated, "PackageVersion")
    sha = read_field(generated, "InstallerSha256")
    product_code = read_field(generated, "ProductCode")

    if not semver or not sha or not product_code:
        sys.exit("the generated manifest is missing PackageVersion, InstallerSha256 or ProductCode")
    if sha.strip("0") == "":
        sys.exit("the generated manifest still carries the placeholder hash")

    release = version.to_name(semver)
    target_dir = MANIFEST_ROOT / semver

    if not target_dir.exists():
        # A new version: start from the most recent existing directory so the
        # locale manifests keep their descriptions and tags.
        existing = sorted(
            (d for d in MANIFEST_ROOT.iterdir() if d.is_dir()),
            key=lambda d: d.name,
        )
        if not existing:
            sys.exit(f"no existing manifest directory to base {semver} on")
        source = existing[-1]
        shutil.copytree(source, target_dir)
        print(f"  created {target_dir.relative_to(ROOT)} from {source.name}")
        for path in target_dir.glob("*.yaml"):
            text = path.read_text()
            text = re.sub(
                r"^PackageVersion:.*$", f"PackageVersion: {semver}", text, flags=re.MULTILINE
            )
            # The release name appears in download and release-notes URLs.
            text = re.sub(
                r"releases/(download|tag)/tsudev-cwico-v[\d.]+",
                lambda m: f"releases/{m.group(1)}/{release}",
                text,
            )
            text = re.sub(
                r"tsudev-cwico_[\d.]+_x64_en-US\.msi",
                f"tsudev-cwico_{semver}_x64_en-US.msi",
                text,
            )
            path.write_text(text)

    installer = target_dir / "tsudev.cwico.installer.yaml"
    installer.write_text(generated)

    print(f"  {installer.relative_to(ROOT)}")
    print(f"    release      {release}")
    print(f"    version      {semver}")
    print(f"    sha256       {sha}")
    print(f"    productCode  {product_code}")
    print()
    print("  Review the diff, then open a pull request against microsoft/winget-pkgs")
    print(f"  with the contents of {target_dir.relative_to(ROOT)}/")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
