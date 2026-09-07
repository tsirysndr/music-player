#!/usr/bin/env python3
"""Generate `src/Components/UI/desktopIcons.tsx` from the desktop's SVGs.

The two clients are meant to look like one product, and the icons are the most
visible way they could drift: a different disc, a different heart, a play
triangle with a different weight. Copying them by hand would drift the moment
one side is touched, so this reads `desktop/assets/icons/*.svg` — the files
`desktop/ui/icons.slint` embeds — and emits a React component per glyph.

Run from `webui/musicplayer`:

    python3 scripts/generate-icons.py

The output is committed; this is not a build step. Re-run it when a desktop
icon changes.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from xml.etree import ElementTree

SVG_NS = "http://www.w3.org/2000/svg"
ElementTree.register_namespace("", SVG_NS)

ICONS_DIR = Path("../../desktop/assets/icons")
OUT = Path("src/Components/UI/desktopIcons.tsx")

# The Slint name (`Icons.list-music`) → the camelCase name the web set uses.
# Anything not listed keeps its file name, camelCased.
NAME_OVERRIDES = {
    "volume-2": "volume",
    "volume-mute": "volumeMute",
    "directory": "folder",
    "x": "close",
}

# Dropped from the root <svg>: a hard-coded size (the component takes one), a
# styled-components class, presentation left over from wherever the file was
# copied from.
ROOT_DROP_ATTRS = {
    "width",
    "height",
    "class",
    "style",
    "aria-hidden",
    "focusable",
    "color",
    "xmlns",
}

# Dropped from the *children*: only the leftovers. `width`/`height` are
# geometry on a <rect> — dropping them there erases the shape, which is what a
# pause button and the panel icons are made of.
CHILD_DROP_ATTRS = {"class", "style", "xmlns", "color"}

# SVG attribute → JSX prop. Anything else is camelCased mechanically.
ATTR_MAP = {
    "stroke-width": "strokeWidth",
    "stroke-linecap": "strokeLinecap",
    "stroke-linejoin": "strokeLinejoin",
    "stroke-opacity": "strokeOpacity",
    "fill-opacity": "fillOpacity",
    "fill-rule": "fillRule",
    "clip-rule": "clipRule",
    "stroke-dasharray": "strokeDasharray",
}


def camel(name: str) -> str:
    head, *rest = name.split("-")
    return head + "".join(part.capitalize() for part in rest)


def jsx_attr(name: str) -> str:
    if name in ATTR_MAP:
        return ATTR_MAP[name]
    return camel(name)


def render(element: ElementTree.Element, indent: str) -> str:
    """One SVG child element as JSX."""
    tag = element.tag.split("}")[-1]
    attrs = []
    for key, value in element.attrib.items():
        key = key.split("}")[-1]
        if key in CHILD_DROP_ATTRS:
            continue
        # A `fill` or `stroke` naming a literal colour would ignore the skin;
        # `none` and `currentColor` are both meaningful, so they stay.
        if key in ("fill", "stroke") and value not in ("none", "currentColor"):
            continue
        attrs.append(f'{jsx_attr(key)}="{value}"')

    children = list(element)
    open_tag = f"<{tag}" + ("" if not attrs else " " + " ".join(attrs))
    if not children:
        return f"{indent}{open_tag} />"

    inner = "\n".join(render(child, indent + "  ") for child in children)
    return f"{indent}{open_tag}>\n{inner}\n{indent}</{tag}>"


def component(path: Path) -> tuple[str, str] | None:
    root = ElementTree.parse(path).getroot()
    view_box = root.get("viewBox")
    if not view_box:
        print(f"skipping {path.name}: no viewBox", file=sys.stderr)
        return None

    # Some of the files wrap their paths in a <g> that only carries a literal
    # fill; unwrap so the paths inherit `currentColor` from the <svg>.
    def flatten(element: ElementTree.Element) -> list[ElementTree.Element]:
        out = []
        for child in element:
            if child.tag.split("}")[-1] == "g":
                out.extend(flatten(child))
            else:
                out.append(child)
        return out

    body = "\n".join(render(child, "    ") for child in flatten(root))

    root_fill = root.get("fill")

    # An icon is stroked if the root says so, or if any shape inside does —
    # some of these files put the stroke on the paths and leave the root bare.
    # Getting this wrong is not subtle: an outline heart that inherits a
    # `currentColor` fill renders as a solid one.
    def strokes(element: ElementTree.Element) -> bool:
        for child in element.iter():
            if child.get("stroke") == "currentColor":
                return True
        return False

    stroked = root.get("stroke") == "currentColor" or strokes(root)

    # A file that declares neither is a filled shape whose fill was on the
    # paths (often as `fill="initial"`, which is dropped above). Without this
    # it would come out `fill="none"` and render as nothing at all.
    if not stroked and root_fill in (None, "none"):
        root_fill = "currentColor"
    elif stroked and root_fill is None:
        root_fill = "none"

    props = "{ size = 24, " + ("stroke = 2, " if stroked else "stroke: _stroke, ") + "...props }"
    svg_attrs = [f'viewBox="{view_box}"', "width={size}", "height={size}"]
    svg_attrs.append(f'fill="{root_fill or "none"}"')
    if stroked:
        svg_attrs += [
            'stroke="currentColor"',
            "strokeWidth={stroke}",
            f'strokeLinecap="{root.get("stroke-linecap", "round")}"',
            f'strokeLinejoin="{root.get("stroke-linejoin", "round")}"',
        ]

    name = NAME_OVERRIDES.get(path.stem, camel(path.stem))
    pascal = name[0].upper() + name[1:]
    attrs = "\n    ".join(svg_attrs)

    return name, (
        f"/** `desktop/assets/icons/{path.name}` */\n"
        f"export const {pascal}: FC<IconProps> = ({props}) => (\n"
        f"  <svg\n    {attrs}\n    aria-hidden=\"true\"\n    {{...props}}\n  >\n"
        f"{body}\n"
        f"  </svg>\n"
        f");\n"
    )


def main() -> None:
    if not ICONS_DIR.is_dir():
        sys.exit(f"{ICONS_DIR} not found — run this from webui/musicplayer")

    made: list[tuple[str, str]] = []
    for path in sorted(ICONS_DIR.glob("*.svg")):
        result = component(path)
        if result:
            made.append(result)

    header = (
        "/**\n"
        " * The desktop client's icons, as React components.\n"
        " *\n"
        " * GENERATED by `scripts/generate-icons.py` from\n"
        " * `desktop/assets/icons/*.svg` — the same files `desktop/ui/icons.slint`\n"
        " * embeds. Do not edit: change the SVG and re-run the script.\n"
        " *\n"
        " * Every glyph takes `currentColor`, so it tints from the skin exactly as\n"
        " * `Image.colorize` tints it on the desktop.\n"
        " */\n"
        "import type { FC } from \"react\";\n"
        "import type { IconProps } from \"./iconTypes\";\n\n"
    )

    body = "\n".join(source for _, source in made)
    index = (
        "/** Every desktop glyph, under the name the Slint `Icons` global uses. */\n"
        "export const DESKTOP_ICONS = {\n"
        + "".join(
            f"  {name}: {name[0].upper() + name[1:]},\n" for name, _ in made
        )
        + "} as const;\n"
    )

    OUT.write_text(header + body + "\n" + index)
    print(f"wrote {OUT} ({len(made)} icons)")


if __name__ == "__main__":
    main()
