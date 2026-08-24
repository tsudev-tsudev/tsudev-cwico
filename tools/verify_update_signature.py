#!/usr/bin/env python3
"""
Verify that a release's update payload will actually be accepted.

This checks the one property the whole update mechanism rests on: an installed
copy only installs a payload signed by the private key matching the public key
compiled into it. If the signature does not verify - wrong key, unsigned build,
corrupted download - every user is shown a mandatory update they can never
install, and the only way out is downloading a fresh installer by hand.

The release workflow already fails when no `.sig` is produced. This goes
further and checks the signature is one *this* build will accept, which
catches the case that matters most: a key rotation that was applied to the
secrets but not to `tauri.conf.json`.

Run against a downloaded release, or against the artefacts of a build:

    gh run download <run-id> -n installers -D /tmp/release
    python3 tools/verify_update_signature.py /tmp/release

Requires PyNaCl:  pip install pynacl
"""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import pathlib
import sys

try:
    from nacl.exceptions import BadSignatureError
    from nacl.signing import VerifyKey
except ImportError:  # pragma: no cover
    sys.exit("PyNaCl is required:  pip install pynacl")

ROOT = pathlib.Path(__file__).resolve().parent.parent
TAURI_CONF = ROOT / "app" / "src-tauri" / "tauri.conf.json"

#: minisign's prehashed algorithm: the signature covers BLAKE2b-512 of the
#: file rather than the file itself. Tauri uses this for anything large.
PREHASHED = b"ED"


def load_public_key() -> tuple[bytes, bytes]:
    """The key id and Ed25519 public key the application was built with."""
    conf = json.loads(TAURI_CONF.read_text())
    try:
        pubkey_b64 = conf["plugins"]["updater"]["pubkey"]
    except KeyError:
        sys.exit(
            "no updater public key in tauri.conf.json - is the updater configured?"
        )
    # Tauri stores the whole minisign public-key *file*, base64-encoded.
    key_file = base64.b64decode(pubkey_b64).decode()
    raw = base64.b64decode(key_file.strip().splitlines()[-1])
    return raw[2:10], raw[10:42]


def verify(payload: pathlib.Path, signature: pathlib.Path, key_id: bytes,
           public_key: bytes) -> str | None:
    """Return None on success, or a description of what went wrong."""
    sig_file = base64.b64decode(signature.read_text()).decode()
    lines = sig_file.strip().splitlines()
    if len(lines) < 2:
        return "the signature file is malformed"
    raw = base64.b64decode(lines[1])
    algorithm, sig_key_id, sig = raw[:2], raw[2:10], raw[10:74]

    if sig_key_id != key_id:
        return (
            f"signed by key {sig_key_id.hex()}, but this build expects "
            f"{key_id.hex()} - the signing secret and tauri.conf.json disagree"
        )

    data = payload.read_bytes()
    signed = (
        hashlib.blake2b(data, digest_size=64).digest()
        if algorithm == PREHASHED
        else data
    )
    try:
        VerifyKey(public_key).verify(signed, sig)
    except BadSignatureError:
        return "the signature does not verify - the payload was modified after signing"
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.strip().splitlines()[0])
    parser.add_argument(
        "directory",
        type=pathlib.Path,
        help="a directory containing the installers and their .sig files",
    )
    args = parser.parse_args()

    if not args.directory.is_dir():
        sys.exit(f"not a directory: {args.directory}")

    key_id, public_key = load_public_key()
    print(f"  application expects key {key_id.hex()}\n")

    signatures = sorted(args.directory.rglob("*.sig"))
    if not signatures:
        sys.exit(
            f"no .sig files under {args.directory} - this release would deliver "
            "no update at all"
        )

    failures = []
    for signature in signatures:
        payload = signature.with_suffix("")
        if not payload.exists():
            failures.append(f"{signature.name}: no payload alongside it")
            print(f"  FAIL {signature.name}: no payload alongside it")
            continue
        problem = verify(payload, signature, key_id, public_key)
        if problem:
            failures.append(f"{payload.name}: {problem}")
            print(f"  FAIL {payload.name}: {problem}")
        else:
            size = payload.stat().st_size
            print(f"  ok   {payload.name}  ({size:,} bytes)")

    print()
    if failures:
        print("installed copies would REJECT this update:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print(f"  installed copies will accept this update ({len(signatures)} payload(s))")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
