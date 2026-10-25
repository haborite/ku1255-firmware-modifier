"""Draw the KU-1255 Firmware Modifier app icon.

Usage: python3 dev/make_icon.py icons

Motif: a black ThinkPad keyboard tile with the red TrackPoint and the three
mouse buttons, plus one highlighted (remapped) key.
"""
import io
import struct
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFilter

SS = 4  # supersampling factor
BASE = 1024

BODY_TOP = (0x2b, 0x2b, 0x2e)
BODY_BOTTOM = (0x16, 0x16, 0x18)
KEY = (0x45, 0x45, 0x4a)
KEY_SMALL = (0x5c, 0x5c, 0x62)
KEY_HL = (0x58, 0x9c, 0xf0)  # remapped key
BUTTON = (0x3a, 0x3a, 0x3e)
RED = (0xe2, 0x23, 0x1a)
RED_DARK = (0xa8, 0x14, 0x0e)


def draw(size_px: int = BASE, small: bool = False) -> Image.Image:
    """`small` drops detail so the icon stays legible at 16-32 px."""
    S = BASE * SS
    u = S / BASE  # 1 design unit in supersampled px
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))

    # Body: rounded tile with a vertical gradient and soft drop shadow.
    m, r = 72 * u, 190 * u
    box = (m, m + 8 * u, S - m, S - m - 8 * u)
    shadow = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    ImageDraw.Draw(shadow).rounded_rectangle(
        (box[0], box[1] + 18 * u, box[2], box[3] + 18 * u), r, fill=(0, 0, 0, 110))
    if not small:
        img.alpha_composite(shadow.filter(ImageFilter.GaussianBlur(22 * u)))

    grad = Image.new("RGBA", (S, S))
    gd = ImageDraw.Draw(grad)
    for y in range(S):
        t = min(max((y - box[1]) / (box[3] - box[1]), 0), 1)
        c = tuple(round(a + (b - a) * t) for a, b in zip(BODY_TOP, BODY_BOTTOM))
        gd.line((0, y, S, y), fill=c + (255,))
    mask = Image.new("L", (S, S), 0)
    ImageDraw.Draw(mask).rounded_rectangle(box, r, fill=255)
    img.paste(grad, (0, 0), mask)

    d = ImageDraw.Draw(img)

    # Keys: 3 rows x 6 columns, then a row with the space bar.
    cols, rows = (4, 3) if small else (6, 3)
    left, right = 170 * u, S - 170 * u
    top = (230 if small else 256) * u
    gap = (40 if small else 22) * u
    key = KEY_SMALL if small else KEY
    kw = (right - left - gap * (cols - 1)) / cols
    kh = kw * (0.62 if small else 0.86)
    kr = 22 * u
    for row in range(rows):
        y = top + row * (kh + gap)
        for col in range(cols):
            x = left + col * (kw + gap)
            fill = KEY_HL if (row, col) == ((0, 3) if small else (0, 4)) else key
            d.rounded_rectangle((x, y, x + kw, y + kh), kr, fill=fill)
    # Bottom row: short key, space bar, short key.
    y = top + rows * (kh + gap)
    sw = kw * 1.0
    d.rounded_rectangle((left, y, left + sw, y + kh), kr, fill=key)
    d.rounded_rectangle((left + sw + gap, y, right - sw - gap, y + kh), kr, fill=key)
    d.rounded_rectangle((right - sw, y, right, y + kh), kr, fill=key)
    keys_bottom = y + kh

    # TrackPoint: between the middle keys of rows 2 and 3, with a dark halo
    # that notches the surrounding key corners like the real keyboard.
    cx = S / 2
    cy = top + 2 * kh + 1.5 * gap
    halo, dot = (100 * u, 78 * u) if small else (58 * u, 44 * u)
    d.ellipse((cx - halo, cy - halo, cx + halo, cy + halo), fill=BODY_TOP)
    d.ellipse((cx - dot, cy - dot, cx + dot, cy + dot), fill=RED_DARK)
    d.ellipse((cx - dot + 5 * u, cy - dot + 3 * u, cx + dot - 5 * u, cy + dot - 7 * u), fill=RED)

    # Mouse buttons: three buttons, the outer two with red stripes.
    by0 = keys_bottom + (50 if small else 44) * u
    by1 = by0 + (90 if small else 74) * u
    bl, br = 250 * u, S - 250 * u
    bgap = (30 if small else 16) * u
    mid_w = 150 * u
    side_w = (br - bl - mid_w - 2 * bgap) / 2
    br_r = 18 * u
    xs = [bl, bl + side_w + bgap, bl + side_w + bgap + mid_w + bgap]
    ws = [side_w, mid_w, side_w]
    for i, (x, w) in enumerate(zip(xs, ws)):
        d.rounded_rectangle((x, by0, x + w, by1), br_r, fill=BUTTON)
    stripe = (22 if small else 9) * u
    for x, w in ((xs[0], ws[0]), (xs[2], ws[2])):
        if small:
            sy = (by0 + by1 - stripe) / 2
            d.rounded_rectangle((x + 24 * u, sy, x + w - 24 * u, sy + stripe), stripe / 2, fill=RED)
            continue
        d.rounded_rectangle((x + 30 * u, by0 + 14 * u, x + w - 30 * u, by0 + 14 * u + stripe),
                            stripe / 2, fill=RED)
        d.rounded_rectangle((x + 30 * u, by1 - 14 * u - stripe, x + w - 30 * u, by1 - 14 * u),
                            stripe / 2, fill=RED)

    return img.resize((size_px, size_px), Image.LANCZOS)


def main(out: Path) -> None:
    out.mkdir(parents=True, exist_ok=True)
    master = draw(BASE)
    # Tiny icons get no shadow and a tighter crop so the tile fills the square.
    small = draw(BASE, small=True).crop((56, 56, BASE - 56, BASE - 56))

    def at(s: int) -> Image.Image:
        src = small if s <= 32 else master
        return src if src.size == (s, s) else src.resize((s, s), Image.LANCZOS)

    for s in (32, 48, 128, 256, 512, 1024):
        at(s).save(out / f"{s}x{s}.png", optimize=True)
    write_ico(out / "icon.ico", [at(s) for s in (16, 24, 32, 48, 64, 128, 256)])


def write_ico(path: Path, images: list[Image.Image]) -> None:
    """PNG-compressed ICO; Pillow would resample the small sizes from one image."""
    blobs = []
    for im in images:
        buf = io.BytesIO()
        im.save(buf, "PNG", optimize=True)
        blobs.append(buf.getvalue())
    head = struct.pack("<HHH", 0, 1, len(images))
    offset = len(head) + 16 * len(images)
    for im, blob in zip(images, blobs):
        w, h = (v % 256 for v in im.size)
        head += struct.pack("<BBBBHHII", w, h, 0, 0, 1, 32, len(blob), offset)
        offset += len(blob)
    path.write_bytes(head + b"".join(blobs))


if __name__ == "__main__":
    main(Path(sys.argv[1]))
