#!/usr/bin/env python3
"""
Render the tsudev wordmark to PNG.

Markdown on GitHub strips inline `style` attributes, so
`<span style="color:#2482bd">tsu</span>` renders as plain uncoloured text
there. The application renders the wordmark as live text (see
`ui/src/components/Brand.tsx`); documentation needs an image to keep the same
two-colour identity — `tsu` in the brand blue, `dev` in the brand orange.

Two variants are produced because a single colour pair cannot serve both of
GitHub's themes: the light-theme blue is too dark to read on a dark page.
`<picture>` with a `prefers-color-scheme` media query selects between them.

Run:  python3 tools/gen_wordmark.py
"""
from __future__ import annotations

import pathlib
import sys

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:  # pragma: no cover
    sys.exit("Pillow is required:  pip install Pillow")

ROOT = pathlib.Path(__file__).resolve().parent.parent
LOGO = ROOT / "assets" / "brand" / "tsudev-logo.png"
OUT = ROOT / "assets" / "brand"

# Rendered at 4x and downsampled, so the edges stay clean at any display size.
SCALE = 4
FONT_SIZE = 64 * SCALE
LOGO_HEIGHT = 76 * SCALE
GAP = 18 * SCALE
PADDING = 8 * SCALE

FONT_CANDIDATES = [
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
    "/usr/share/fonts/truetype/noto/NotoSans-Bold.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
    "C:\\Windows\\Fonts\\segoeuib.ttf",
]

# (name, tsu colour, dev colour)
VARIANTS = [
    # Light pages: the 600-weight tokens, as the app uses.
    ("tsudev-wordmark.png", (36, 130, 189), (239, 109, 24)),
    # Dark pages: the 300-weight tokens, which is what the app swaps to.
    ("tsudev-wordmark-dark.png", (120, 192, 216), (255, 167, 107)),
]


def load_font() -> ImageFont.FreeTypeFont:
    for path in FONT_CANDIDATES:
        if pathlib.Path(path).exists():
            return ImageFont.truetype(path, FONT_SIZE)
    sys.exit("no bold sans-serif font found; add one to FONT_CANDIDATES")


def main() -> int:
    if not LOGO.exists():
        sys.exit(f"missing logo: {LOGO}")

    font = load_font()
    logo = Image.open(LOGO).convert("RGBA")
    logo_width = round(logo.width * LOGO_HEIGHT / logo.height)
    logo = logo.resize((logo_width, LOGO_HEIGHT), Image.LANCZOS)

    # Measure both halves so `dev` starts exactly where `tsu` ends.
    probe = ImageDraw.Draw(Image.new("RGBA", (1, 1)))
    tsu_width = round(probe.textlength("tsu", font=font))
    dev_width = round(probe.textlength("dev", font=font))
    ascent, descent = font.getmetrics()
    text_height = ascent + descent

    width = PADDING * 2 + logo_width + GAP + tsu_width + dev_width
    height = PADDING * 2 + max(LOGO_HEIGHT, text_height)

    for name, tsu_colour, dev_colour in VARIANTS:
        canvas = Image.new("RGBA", (width, height), (0, 0, 0, 0))
        canvas.alpha_composite(logo, (PADDING, (height - LOGO_HEIGHT) // 2))

        draw = ImageDraw.Draw(canvas)
        baseline_y = (height - text_height) // 2
        x = PADDING + logo_width + GAP
        draw.text((x, baseline_y), "tsu", font=font, fill=tsu_colour)
        draw.text((x + tsu_width, baseline_y), "dev", font=font, fill=dev_colour)

        out = OUT / name
        canvas.resize((width // SCALE, height // SCALE), Image.LANCZOS).save(
            out, optimize=True
        )
        print(f"  {name}  {out.stat().st_size // 1024} KB  ({width // SCALE}x{height // SCALE})")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
