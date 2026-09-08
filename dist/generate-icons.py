#!/usr/bin/env python3
"""Regenerate every raster app icon from desktop/assets/icon.svg.

icon.svg is the single source of truth for the app mark (the Synthwave-skin
neon note). Everything else in the tree is derived, so run this after editing
it:

    python3 dist/generate-icons.py

Outputs:
  desktop/assets/AppIcon.icns          bundled by desktop/src/main.rs and
                                       dist/package-macos.sh
  webui/musicplayer/public/            favicon.ico, logo192.png, logo512.png
  webui/musicplayer/build/             same three (build output is committed)
  webui/chromecast/public/             same three

The Linux package installs icon.svg itself (dist/package-linux.sh), so there is
nothing to rasterize for it.

Requires Inkscape for SVG rendering and iconutil (macOS) for the .icns. The
.ico is assembled here: entries are PNG-encoded, which every browser since IE9
reads, and it keeps the file far smaller than BMP entries.
"""

from __future__ import annotations

import shutil
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SOURCE = REPO / "desktop" / "assets" / "icon.svg"

# Sizes packed into favicon.ico. 16/32 are what browsers actually draw; the
# larger entries are for Windows shortcuts and taskbar scaling.
ICO_SIZES = [16, 24, 32, 48, 64, 256]

# (filename, pixel size) pairs for the .iconset iconutil consumes.
ICONSET = [
    ("icon_16x16.png", 16),
    ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32),
    ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128),
    ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256),
    ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512),
    ("icon_512x512@2x.png", 1024),
]

WEB_TARGETS = [
    REPO / "webui" / "musicplayer" / "public",
    REPO / "webui" / "musicplayer" / "build",
    REPO / "webui" / "chromecast" / "public",
]


def require(tool: str) -> str:
    path = shutil.which(tool)
    if path is None:
        sys.exit(f"error: {tool} not found on PATH")
    return path


def render(inkscape: str, size: int, out: Path) -> Path:
    """Rasterize icon.svg at `size`x`size`."""
    out.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [inkscape, "-w", str(size), "-h", str(size), "-o", str(out), str(SOURCE)],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if not out.exists():
        sys.exit(f"error: inkscape produced no output at {size}px")
    return out


def write_ico(pngs: list[tuple[int, bytes]], out: Path) -> None:
    """Pack PNG blobs into an .ico. 256px is stored as 0 per the ICO spec."""
    count = len(pngs)
    header = struct.pack("<HHH", 0, 1, count)
    offset = 6 + 16 * count
    entries, blobs = b"", b""
    for size, data in pngs:
        entries += struct.pack(
            "<BBBBHHII",
            size if size < 256 else 0,
            size if size < 256 else 0,
            0,  # palette size: 0 for truecolor
            0,  # reserved
            1,  # colour planes
            32,  # bits per pixel
            len(data),
            offset,
        )
        blobs += data
        offset += len(data)
    out.write_bytes(header + entries + blobs)


def main() -> None:
    if not SOURCE.exists():
        sys.exit(f"error: {SOURCE} not found")
    inkscape = require("inkscape")
    iconutil = require("iconutil")

    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)

        # macOS .icns
        iconset = tmp / "AppIcon.iconset"
        iconset.mkdir()
        for name, size in ICONSET:
            render(inkscape, size, iconset / name)
        icns = REPO / "desktop" / "assets" / "AppIcon.icns"
        subprocess.run(
            [iconutil, "-c", "icns", str(iconset), "-o", str(icns)], check=True
        )
        print(f"  {icns.relative_to(REPO)}")

        # favicon.ico
        ico_pngs = [
            (size, render(inkscape, size, tmp / f"ico-{size}.png").read_bytes())
            for size in ICO_SIZES
        ]
        ico = tmp / "favicon.ico"
        write_ico(ico_pngs, ico)

        # PWA logos
        logo192 = render(inkscape, 192, tmp / "logo192.png")
        logo512 = render(inkscape, 512, tmp / "logo512.png")

        for target in WEB_TARGETS:
            if not target.is_dir():
                print(f"  skipped {target.relative_to(REPO)} (missing)")
                continue
            for src, name in ((ico, "favicon.ico"), (logo192, "logo192.png"), (logo512, "logo512.png")):
                shutil.copyfile(src, target / name)
            print(f"  {target.relative_to(REPO)}/{{favicon.ico,logo192.png,logo512.png}}")


if __name__ == "__main__":
    main()
