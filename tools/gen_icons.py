#!/usr/bin/env python3
"""
Generate the full tsudev-cwico icon set from assets/brand/tsudev-logo.png.

Outputs:
  app/src-tauri/icons/*        -> Tauri bundler (MSI / NSIS / MSIX / Linux / macOS)
  ui/public/brand/*            -> icons consumed by the web UI
  assets/brand/*               -> square + store-ready derivatives

Run:  python3 tools/gen_icons.py
"""
from __future__ import annotations

import pathlib
import sys

try:
    from PIL import Image
except ImportError:  # pragma: no cover
    sys.exit("Pillow is required:  pip install Pillow")

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "assets" / "brand" / "tsudev-logo.png"

TAURI_ICONS = ROOT / "app" / "src-tauri" / "icons"
UI_BRAND = ROOT / "ui" / "public" / "brand"
BRAND = ROOT / "assets" / "brand"

# Tauri's standard icon set.
PNG_SIZES = [32, 128, 256, 512]
# MSIX / Microsoft Store tile assets (square, transparent).
STORE_TILES = {
    "Square30x30Logo.png": 30,
    "Square44x44Logo.png": 44,
    "Square71x71Logo.png": 71,
    "Square89x89Logo.png": 89,
    "Square107x107Logo.png": 107,
    "Square142x142Logo.png": 142,
    "Square150x150Logo.png": 150,
    "Square284x284Logo.png": 284,
    "Square310x310Logo.png": 310,
    "StoreLogo.png": 50,
}
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]


def squarify(im: Image.Image, pad_ratio: float = 0.06) -> Image.Image:
    """Centre the logo on a transparent square canvas with a little breathing room."""
    w, h = im.size
    side = int(max(w, h) * (1 + pad_ratio * 2))
    canvas = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    canvas.paste(im, ((side - w) // 2, (side - h) // 2), im)
    return canvas


def resize(sq: Image.Image, size: int) -> Image.Image:
    return sq.resize((size, size), Image.LANCZOS)


def main() -> int:
    if not SRC.exists():
        sys.exit(f"missing source logo: {SRC}")

    logo = Image.open(SRC).convert("RGBA")
    square = squarify(logo)

    for d in (TAURI_ICONS, UI_BRAND, BRAND):
        d.mkdir(parents=True, exist_ok=True)

    # Canonical square master, reused by the UI and the docs.
    square.save(BRAND / "tsudev-logo-square.png")
    written = 1

    # --- Tauri PNG set -------------------------------------------------------
    for size in PNG_SIZES:
        resize(square, size).save(TAURI_ICONS / f"{size}x{size}.png")
        written += 1
    resize(square, 256).save(TAURI_ICONS / "128x128@2x.png")
    resize(square, 512).save(TAURI_ICONS / "icon.png")
    written += 2

    # --- Windows .ico (multi-resolution) -------------------------------------
    resize(square, 256).save(
        TAURI_ICONS / "icon.ico",
        format="ICO",
        sizes=[(s, s) for s in ICO_SIZES],
    )
    written += 1

    # --- MSIX / Microsoft Store tiles ----------------------------------------
    for name, size in STORE_TILES.items():
        resize(square, size).save(TAURI_ICONS / name)
        written += 1

    # --- Web UI assets -------------------------------------------------------
    for size in (32, 64, 128, 256, 512):
        resize(square, size).save(UI_BRAND / f"tsudev-logo-{size}.png")
        written += 1
    resize(square, 256).save(
        UI_BRAND / "favicon.ico", format="ICO", sizes=[(16, 16), (32, 32), (48, 48)]
    )
    resize(square, 180).save(UI_BRAND / "apple-touch-icon.png")
    # The UI's <img> source; 512px keeps it crisp on HiDPI at any rendered size.
    resize(square, 512).save(UI_BRAND / "tsudev-logo.png")
    written += 3

    print(f"generated {written} icon files from {SRC.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
