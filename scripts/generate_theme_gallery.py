"""Regenerates theme_gallery.png (referenced by THEMES.md) straight from
src/map/theme.rs's actual `Theme::colors` values, so the gallery can never
drift from the real palettes the way it could when it was a one-off,
uncommitted script.

Run from the repo root with `python3 scripts/generate_theme_gallery.py`
(requires Pillow: `pip install pillow`). Whenever a built-in theme's colors
change, or a new one is added, rerun this and commit the resulting PNG
alongside THEMES.md.

Each card mocks the shapes the widget actually paints: two nodes joined by a
segment-colored line, a third node wearing the selected+alert rings, a
fourth wearing the marker ring, and a node name rendered in the theme's real
`text` color -- so a text-contrast regression (see the ArticCyan/SolarAmber
bug fixed alongside this script's addition) would show up here directly
instead of only in a hex table.
"""

import re
from PIL import Image, ImageDraw, ImageFont

SRC = "src/map/theme.rs"
OUT = "theme_gallery.png"
FONT_REG = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
FONT_BOLD = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"

with open(SRC) as f:
    content = f.read()

enum_block = re.search(r"pub enum Theme \{(.*?)\n\}", content, re.S).group(1)
variant_pattern = re.compile(r"/// (.*?)\n(?:\s*#\[default\]\n)?\s*(\w+),", re.S)
order = []
descriptions = {}
for m in variant_pattern.finditer(enum_block):
    desc, name = m.groups()
    desc = " ".join(line.strip().lstrip("/// ").strip() for line in desc.splitlines())
    order.append(name)
    descriptions[name] = desc

# EguiDefault's enum doc is long (explanatory); use a short blurb for the card instead.
descriptions["EguiDefault"] = "Plain egui's own default colors. The default theme."

pat = re.compile(r"\((\w+),\s*(Dark|Light)\)\s*=>\s*ThemeColors\s*\{([^}]*)\}", re.S)


def parse_color(s, field):
    m = re.search(
        rf"{field}:\s*Color32::from_rgb\(0x([0-9A-Fa-f]+),\s*0x([0-9A-Fa-f]+),\s*0x([0-9A-Fa-f]+)\)",
        s,
    )
    return tuple(int(x, 16) for x in m.groups()) if m else None


data = {}
for m in pat.finditer(content):
    theme, mode, body = m.groups()
    data.setdefault(theme, {})[mode] = {
        f: parse_color(body, f)
        for f in ("node", "segment", "selected", "alert", "marker", "text")
    }

W = 894
PAD = 24
CARD_W = (W - PAD * 3) // 2
CARD_H = 190
ROW_H = 30  # title+desc header per theme
GAP = 14

n_themes = len(order)
H = 120 + n_themes * (ROW_H + 10 + CARD_H + GAP)

bg = (245, 245, 247)
img = Image.new("RGB", (W, H), bg)
draw = ImageDraw.Draw(img)

f_title = ImageFont.truetype(FONT_BOLD, 22)
f_sub = ImageFont.truetype(FONT_REG, 13)
f_theme_name = ImageFont.truetype(FONT_BOLD, 17)
f_desc = ImageFont.truetype(FONT_REG, 11)
f_mode = ImageFont.truetype(FONT_BOLD, 12)
f_legend = ImageFont.truetype(FONT_REG, 10)
f_label = ImageFont.truetype(FONT_REG, 11)

draw.text((PAD, 18), "egui-map -- built-in Theme gallery", font=f_title, fill=(20, 20, 24))
draw.text(
    (PAD, 46),
    "Each palette resolves via Theme::colors(ColorMode) -- node / segment / selected / alert / marker / text.",
    font=f_sub,
    fill=(90, 90, 96),
)

y = 78


def draw_card(x0, y0, w, h, mode, colors):
    dark = mode == "Dark"
    card_bg = (26, 26, 30) if dark else (255, 255, 255)
    border = (50, 50, 55) if dark else (222, 222, 226)
    fg_label = (230, 230, 232) if dark else (40, 40, 44)
    draw.rounded_rectangle([x0, y0, x0 + w, y0 + h], radius=8, fill=card_bg, outline=border)
    draw.text((x0 + 12, y0 + 8), mode, font=f_mode, fill=fg_label)

    # Mini graph: top node, bottom-left node, bottom-right node (gets the
    # selected+alert rings, as in the original mockup), plus a small
    # separate marker-ring node to its right.
    top = (x0 + w * 0.34, y0 + 42)
    left = (x0 + w * 0.16, y0 + 82)
    right = (x0 + w * 0.40, y0 + 82)
    marker_node = (x0 + w * 0.60, y0 + 60)

    seg = colors["segment"]
    draw.line([top, left], fill=seg, width=2)
    draw.line([top, right], fill=seg, width=2)

    r = 7
    draw.ellipse(
        [top[0] - r, top[1] - r, top[0] + r, top[1] + r], fill=colors["node"]
    )
    draw.ellipse(
        [left[0] - r, left[1] - r, left[0] + r, left[1] + r], fill=colors["node"]
    )
    # Right node: selection ring (outer) + alert ring (inner), same node,
    # same as the original gallery's convention.
    rr = r + 7
    draw.ellipse(
        [right[0] - rr, right[1] - rr, right[0] + rr, right[1] + rr],
        outline=colors["selected"],
        width=2,
    )
    draw.ellipse(
        [right[0] - r - 3, right[1] - r - 3, right[0] + r + 3, right[1] + r + 3],
        outline=colors["alert"],
        width=2,
    )
    draw.ellipse(
        [right[0] - r, right[1] - r, right[0] + r, right[1] + r], fill=colors["node"]
    )

    # A separate node with a marker ring -- the new, distinct "flagged"
    # role, deliberately drawn apart from the selected/alert node above so
    # it doesn't read as a third ring on the same target.
    draw.ellipse(
        [
            marker_node[0] - rr,
            marker_node[1] - rr,
            marker_node[0] + rr,
            marker_node[1] + rr,
        ],
        outline=colors["marker"],
        width=3,
    )
    draw.ellipse(
        [
            marker_node[0] - r,
            marker_node[1] - r,
            marker_node[0] + r,
            marker_node[1] + r,
        ],
        fill=colors["node"],
    )

    # Node name label, in the theme's actual `text` color, anchored under
    # the left node -- the concrete thing the ArticCyan/SolarAmber contrast
    # bug broke (this label was unreadable against this very card
    # background before the fix).
    label = "Node name"
    tx, ty = left[0] - 24, left[1] + 12
    draw.text((tx, ty), label, font=f_label, fill=colors["text"])

    # Legend chips: node / seg / sel / alert / marker / text
    legend_y = y0 + h - 22
    chip = 10
    lx = x0 + 12
    entries = [
        ("node", colors["node"]),
        ("seg", colors["segment"]),
        ("sel", colors["selected"]),
        ("alert", colors["alert"]),
        ("marker", colors["marker"]),
        ("text", colors["text"]),
    ]
    for name, col in entries:
        draw.rectangle([lx, legend_y, lx + chip, legend_y + chip], fill=col, outline=fg_label)
        tw = draw.textlength(name, font=f_legend)
        draw.text((lx + chip + 3, legend_y - 1), name, font=f_legend, fill=fg_label)
        lx += chip + 6 + tw + 10


for theme in order:
    draw.text((PAD, y), theme, font=f_theme_name, fill=(20, 20, 24))
    desc = descriptions[theme]
    # wrap description to card width roughly
    draw.text((PAD, y + 20), desc[:90], font=f_desc, fill=(110, 110, 116))
    card_y = y + ROW_H + 8
    draw_card(PAD, card_y, CARD_W, CARD_H, "Light", data[theme]["Light"])
    draw_card(PAD * 2 + CARD_W, card_y, CARD_W, CARD_H, "Dark", data[theme]["Dark"])
    y = card_y + CARD_H + GAP

img = img.crop((0, 0, W, y + 8))
img.save(OUT)
print("saved", OUT, img.size)
