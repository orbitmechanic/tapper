"""Generate every raster asset for TAPPER: LAST CALL with Krita.

    ~/.agents/skills/krita/scripts/kritarun.sh tools/krita_assets.py

All sprites are true 1x pixel art drawn with QImage pixel writes, then pushed
through Krita layers and exported from the merged projection. Source `.kra`
boards land in assets/art/ for hand editing.
"""

import ksketch as ks

ks.started()

import math
import os
import random
import sys
import traceback

from PyQt6.QtGui import QColor

import pixelfont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SPRITES = os.path.join(ROOT, "assets", "sprites")
BACKGROUNDS = os.path.join(ROOT, "assets", "backgrounds")
ART = os.path.join(ROOT, "assets", "art")

for d in (SPRITES, BACKGROUNDS, ART):
    os.makedirs(d, exist_ok=True)

TRANSPARENT = QColor(0, 0, 0, 0)


def C(hexstr, a=255):
    hexstr = hexstr.lstrip("#")
    return QColor(int(hexstr[0:2], 16), int(hexstr[2:4], 16), int(hexstr[4:6], 16), a)


class Pix:
    def __init__(self, w, h):
        from PyQt6.QtGui import QImage

        self.w, self.h = w, h
        self.img = QImage(w, h, QImage.Format.Format_RGBA8888)
        self.img.fill(TRANSPARENT)

    def px(self, x, y, c):
        x, y = int(x), int(y)
        if 0 <= x < self.w and 0 <= y < self.h:
            self.img.setPixelColor(x, y, c)

    def rect(self, x, y, w, h, c):
        for yy in range(int(y), int(y + h)):
            for xx in range(int(x), int(x + w)):
                self.px(xx, yy, c)

    def frame(self, x, y, w, h, c):
        for xx in range(int(x), int(x + w)):
            self.px(xx, y, c)
            self.px(xx, y + h - 1, c)
        for yy in range(int(y), int(y + h)):
            self.px(x, yy, c)
            self.px(x + w - 1, yy, c)

    def hline(self, x, y, w, c):
        for xx in range(int(x), int(x + w)):
            self.px(xx, y, c)

    def vline(self, x, y, h, c):
        for yy in range(int(y), int(y + h)):
            self.px(x, yy, c)

    def line(self, x0, y0, x1, y1, c):
        x0, y0, x1, y1 = int(x0), int(y0), int(x1), int(y1)
        dx, dy = abs(x1 - x0), -abs(y1 - y0)
        sx = 1 if x0 < x1 else -1
        sy = 1 if y0 < y1 else -1
        err = dx + dy
        while True:
            self.px(x0, y0, c)
            if x0 == x1 and y0 == y1:
                break
            e2 = 2 * err
            if e2 >= dy:
                err += dy
                x0 += sx
            if e2 <= dx:
                err += dx
                y0 += sy

    def disc(self, cx, cy, r, c):
        for yy in range(int(cy - r), int(cy + r) + 1):
            for xx in range(int(cx - r), int(cx + r) + 1):
                if (xx - cx) ** 2 + (yy - cy) ** 2 <= r * r:
                    self.px(xx, yy, c)

    def text(self, text, x, y, scale=1, color=(255, 255, 255, 255), shadow=None):
        col = QColor(*color) if isinstance(color, tuple) else color
        if shadow is not None:
            scol = QColor(*shadow) if isinstance(shadow, tuple) else shadow
            pixelfont.draw_text(
                lambda px, py, _c: self.px(px, py, scol),
                text,
                x + scale,
                y + scale,
                scale,
            )
        pixelfont.draw_text(
            lambda px, py, _c: self.px(px, py, col), text, x, y, scale
        )

    def text_outlined(self, text, x, y, scale, fill, outline, shadow=None):
        if shadow is not None:
            off = max(1, scale // 2)
            pixelfont.draw_text(
                lambda px, py, _c: self.rect(px + off, py + off, scale, scale, shadow),
                text,
                x,
                y,
                scale,
            )
        pixelfont.draw_text(
            lambda px, py, _c: self.rect(px - 1, py - 1, scale + 2, scale + 2, outline),
            text,
            x,
            y,
            scale,
        )
        pixelfont.draw_text(
            lambda px, py, _c: self.rect(px, py, scale, scale, fill),
            text,
            x,
            y,
            scale,
        )

    def blit(self, other, x, y):
        for yy in range(other.h):
            for xx in range(other.w):
                c = other.img.pixelColor(xx, yy)
                if c.alpha() > 0:
                    self.px(x + xx, y + yy, c)


# --------------------------------------------------------------------------
# Shared palettes
# --------------------------------------------------------------------------

SKIN = ["#f2c49b", "#d99a6c", "#a9663f", "#7d4428"]
DARK = C("#160d10")
WHITE = C("#fff4d8")

THEMES = ["saloon", "sports", "punk", "alien"]


def style(theme, variant):
    """Return a small palette + body plan for a patron."""
    if theme == 0:
        looks = [
            dict(skin=0, shirt="#3f6fb5", shirt2="#274a80", pants="#5a3a24", hat="#a8783f", hat2="#6e4820", hair="#3a2416", kind="cowboy", extra="#c94b45"),
            dict(skin=1, shirt="#4f8a52", shirt2="#2f5a33", pants="#4a4038", hat="#2f2f38", hat2="#1a1a22", hair="#6b4a2c", kind="cowboy", extra="#d8b04a"),
            dict(skin=0, shirt="#c0474f", shirt2="#8a2c36", pants="#7a3038", hat="#c0474f", hat2="#7c2730", hair="#e4b34e", kind="cowgirl", extra="#f4e0b2"),
            dict(skin=2, shirt="#7b5aa6", shirt2="#533a76", pants="#7d5a38", hat="#c9a34a", hat2="#8a6c26", hair="#d8d0c0", kind="prospector", extra="#b93b3b"),
        ]
        return looks[variant]
    if theme == 1:
        looks = [
            dict(skin=2, shirt="#e08a32", shirt2="#a85c1a", pants="#c25c2a", hat="#f4e0b2", hat2="#c7ae7a", hair="#1d1512", kind="basketball", extra="#f2c94c"),
            dict(skin=0, shirt="#b63a3f", shirt2="#7d2229", pants="#e8e2d2", hat="#b63a3f", hat2="#6e1a20", hair="#6a4426", kind="football", extra="#f4f0e4"),
            dict(skin=0, shirt="#f2eee0", shirt2="#c8c2b0", pants="#f2eee0", hat="#f2eee0", hat2="#bdb5a2", hair="#e8c85a", kind="tennis", extra="#e8c85a"),
            dict(skin=1, shirt="#3e5f9e", shirt2="#263f70", pants="#e8e2d2", hat="#3e5f9e", hat2="#243c68", hair="#2a1c12", kind="baseball", extra="#d23f3f"),
        ]
        return looks[variant]
    if theme == 2:
        looks = [
            dict(skin=0, shirt="#6f8a4a", shirt2="#445a2c", pants="#2e2a34", hat="#b9b9a0", hat2="#777766", hair="#b9b9a0", kind="pinhead", extra="#d84f9a"),
            dict(skin=1, shirt="#2c2a34", shirt2="#17161d", pants="#3f4a5c", hat="#e24f9e", hat2="#9c2c6a", hair="#e24f9e", kind="mohawk", extra="#4fd8d2"),
            dict(skin=0, shirt="#3c3642", shirt2="#211d27", pants="#a83a4a", hat="#d8d0c0", hat2="#8a8478", hair="#d8d0c0", kind="spike", extra="#f2c94c"),
            dict(skin=2, shirt="#7c4a8a", shirt2="#4c2c5a", pants="#e24f9e", hat="#4c2c5a", hat2="#2a172f", hair="#4fd8d2", kind="punkgirl", extra="#f2c94c"),
        ]
        return looks[variant]
    looks = [
        dict(skin=2, shirt="#5a7a4a", shirt2="#354c2c", pants="#6b5a4a", hat="#8ad06a", hat2="#4c8a3c", hair="#8ad06a", kind="beak", extra="#f2c94c"),
        dict(skin=3, shirt="#c8c8d8", shirt2="#8a8a9e", pants="#4a4a62", hat="#b07ad8", hat2="#6e4a96", hair="#b07ad8", kind="threeeyes", extra="#4fd8d2"),
        dict(skin=0, shirt="#b04a4a", shirt2="#7a2c2c", pants="#2c2c3a", hat="#e24f4f", hat2="#8a2c2c", hair="#e24f4f", kind="smiley", extra="#f2c94c"),
        dict(skin=2, shirt="#4a7ab0", shirt2="#2c4c7a", pants="#2c2c3a", hat="#6ad8b0", hat2="#2c8a6a", hair="#6ad8b0", kind="little", extra="#d84f9a"),
    ]
    return looks[variant]


def patron(theme, variant, frame):
    p = Pix(18, 26)
    s = style(theme, variant)
    skin = C(SKIN[s["skin"]])
    skin_d = C(SKIN[min(3, s["skin"] + 1)])
    shirt = C(s["shirt"])
    shirt2 = C(s["shirt2"])
    pants = C(s["pants"])
    hat = C(s["hat"])
    hat2 = C(s["hat2"])
    hair = C(s["hair"])
    extra = C(s["extra"])
    boot = C("#2a1a14")
    walk = frame % 2
    drink = frame == 2

    # Legs / feet.
    if walk == 0:
        p.rect(6, 20, 2, 5, pants)
        p.rect(10, 20, 2, 4, pants)
        p.rect(5, 24, 3, 2, boot)
        p.rect(10, 23, 3, 2, boot)
    else:
        p.rect(7, 20, 2, 4, pants)
        p.rect(10, 20, 2, 5, pants)
        p.rect(6, 23, 3, 2, boot)
        p.rect(10, 24, 3, 2, boot)

    # Torso.
    p.rect(5, 11, 8, 9, shirt)
    p.rect(5, 15, 8, 1, shirt2)
    p.rect(5, 19, 8, 1, C("#1a1218"))

    # Arms.
    p.rect(3, 12, 2, 6, shirt)
    p.rect(3, 17, 2, 2, skin)
    if drink:
        p.rect(13, 8, 2, 4, shirt)
        p.rect(14, 6, 2, 2, skin)
        # mug held at the mouth
        p.rect(14, 4, 4, 4, C("#e8a73c"))
        p.rect(14, 3, 4, 2, C("#fff4d8"))
        p.rect(17, 5, 1, 2, C("#fff4d8"))
    else:
        p.rect(13, 12, 2, 5, shirt)
        p.rect(13, 17, 2, 2, skin)

    # Head.
    p.rect(5, 4, 8, 8, skin)
    p.rect(5, 9, 8, 2, skin_d)
    p.rect(11, 7, 1, 1, DARK)
    p.rect(11, 10, 2, 1, C("#7a3a2a"))

    # Theme-specific headgear and silhouette.
    if theme == 0:
        if s["kind"] == "cowgirl":
            p.rect(4, 3, 10, 2, hat)
            p.rect(6, 1, 6, 2, hat2)
            p.rect(3, 5, 2, 5, hair)
            p.rect(5, 12, 8, 6, shirt)
            p.rect(4, 17, 10, 3, hat2)
            p.rect(6, 20, 6, 2, shirt2)
        else:
            p.rect(3, 3, 12, 2, hat)
            p.rect(5, 1, 8, 2, hat2)
            p.rect(5, 1, 8, 1, hat)
            p.rect(2, 4, 2, 1, hat2)
            p.rect(14, 4, 2, 1, hat2)
        p.rect(5, 12, 2, 6, hat2 if s["kind"] != "cowgirl" else shirt2)
        p.rect(4, 14, 10, 1, extra)
        if s["kind"] == "prospector":
            p.rect(5, 10, 7, 3, hair)
            p.rect(4, 11, 2, 3, hair)
            p.rect(9, 9, 2, 1, DARK)
        if s["kind"] == "cowboy":
            p.rect(8, 10, 3, 1, hair)
    elif theme == 1:
        if s["kind"] == "basketball":
            p.rect(5, 3, 8, 2, extra)
            p.rect(5, 5, 8, 1, hair)
            p.rect(4, 9, 2, 1, extra)
            p.text("3", 8, 13, 1, C("#fff4d8"))
        elif s["kind"] == "football":
            p.rect(4, 3, 10, 6, hat)
            p.rect(5, 4, 8, 4, hat2)
            p.rect(3, 9, 12, 1, C("#e8e2d2"))
            p.rect(4, 11, 10, 3, shirt)
            p.rect(3, 11, 2, 3, C("#e8e2d2"))
            p.rect(13, 11, 2, 3, C("#e8e2d2"))
            p.text("7", 8, 13, 1, C("#fff4d8"))
        elif s["kind"] == "tennis":
            p.rect(5, 2, 8, 2, hat)
            p.rect(4, 4, 10, 1, hat2)
            p.rect(4, 5, 2, 6, hair)
            p.rect(8, 12, 1, 4, C("#d8d2c2"))
            p.rect(9, 12, 4, 1, C("#d8d2c2"))
        else:
            p.rect(5, 2, 8, 3, hat)
            p.rect(4, 5, 10, 1, hat2)
            p.rect(4, 9, 2, 1, extra)
            p.rect(6, 12, 1, 5, C("#e8e2d2"))
            p.rect(10, 12, 1, 5, C("#e8e2d2"))
    elif theme == 2:
        if s["kind"] == "pinhead":
            for i in range(5):
                p.rect(5 + i * 2, 1, 1, 3, hat)
            p.rect(5, 3, 8, 2, skin)
            p.rect(4, 8, 2, 2, C("#9a9a84"))
            p.rect(12, 8, 2, 2, C("#9a9a84"))
            p.text("X", 8, 13, 1, extra)
        elif s["kind"] == "mohawk":
            p.rect(8, 0, 3, 4, hat)
            p.rect(7, 1, 5, 2, hat)
            p.rect(4, 8, 2, 3, hair)
            p.rect(2, 12, 2, 7, C("#211d27"))
            p.rect(14, 12, 2, 7, C("#211d27"))
            for yy in (13, 15, 17):
                p.px(3, yy, C("#c8c8d8"))
                p.px(14, yy, C("#c8c8d8"))
            p.rect(8, 12, 2, 5, extra)
        elif s["kind"] == "spike":
            for i in range(4):
                p.rect(5 + i * 2, 1 + (i % 2), 1, 3, hat)
            p.rect(4, 9, 2, 2, C("#777766"))
            p.rect(2, 11, 2, 6, C("#211d27"))
            p.rect(14, 11, 2, 6, C("#211d27"))
            p.rect(8, 14, 2, 1, extra)
        else:
            p.rect(6, 0, 4, 4, hat)
            p.rect(4, 2, 8, 2, hat)
            p.rect(3, 6, 2, 8, hair)
            p.rect(13, 6, 2, 8, hair)
            p.rect(4, 12, 10, 3, shirt2)
            p.rect(4, 16, 10, 3, hat2)
            p.rect(8, 13, 2, 1, extra)
    else:
        if s["kind"] == "beak":
            p.rect(4, 3, 10, 8, skin)
            p.rect(4, 3, 3, 3, extra)
            p.rect(5, 6, 2, 2, DARK)
            p.rect(9, 7, 5, 2, C("#e8a73c"))
            p.rect(4, 2, 1, 3, hat)
            p.rect(13, 2, 1, 3, hat)
            p.rect(4, 12, 10, 8, shirt)
            p.rect(3, 14, 2, 2, skin)
            p.rect(13, 14, 2, 2, skin)
        elif s["kind"] == "threeeyes":
            p.rect(4, 2, 10, 9, skin)
            p.rect(5, 4, 2, 2, DARK)
            p.rect(8, 3, 2, 2, DARK)
            p.rect(11, 4, 2, 2, DARK)
            p.rect(5, 8, 8, 2, skin_d)
            p.rect(4, 0, 1, 3, hat)
            p.rect(13, 0, 1, 3, hat)
            p.px(4, 0, extra)
            p.px(13, 0, extra)
            p.rect(4, 12, 10, 8, shirt)
            p.rect(6, 14, 6, 1, extra)
        elif s["kind"] == "smiley":
            p.rect(4, 2, 10, 9, skin)
            p.rect(4, 1, 10, 1, hat)
            p.rect(4, 2, 2, 2, extra)
            p.rect(11, 2, 2, 2, extra)
            p.rect(7, 4, 4, 1, DARK)
            for i in range(5):
                p.px(6 + i, 8 + (i % 2), DARK)
            p.rect(4, 12, 10, 8, shirt)
            p.rect(5, 13, 8, 1, extra)
        else:
            p.rect(4, 4, 10, 10, skin)
            p.rect(3, 5, 1, 6, skin_d)
            p.rect(14, 5, 1, 6, skin_d)
            p.rect(5, 8, 3, 4, DARK)
            p.rect(10, 8, 3, 4, DARK)
            p.px(6, 9, WHITE)
            p.px(11, 9, WHITE)
            p.rect(6, 1, 1, 4, hat)
            p.rect(11, 0, 1, 5, hat)
            p.disc(6, 1, 1, extra)
            p.disc(11, 0, 1, extra)
            p.rect(4, 14, 10, 6, shirt)
            p.rect(3, 16, 2, 3, skin)
            p.rect(13, 16, 2, 3, skin)

    # A little shading down the front and a highlight on the shoulders.
    p.vline(12, 12, 7, shirt2)
    p.hline(5, 11, 8, C("#fff8e8") if theme < 2 else shirt2)
    return p


# --------------------------------------------------------------------------
# Bartender
# --------------------------------------------------------------------------


def bartender(frame):
    p = Pix(20, 30)
    skin = C("#f2c49b")
    skin_d = C("#d99a6c")
    shirt = C("#f4efe2")
    shirt_d = C("#c8c0ae")
    vest = C("#2c2632")
    vest_l = C("#443c4c")
    pants = C("#1d1a24")
    apron = C("#d8cfb8")
    tie = C("#b83a3f")
    gold = C("#d8b04a")
    boot = C("#120e14")

    # legs
    if frame in (1, 3):
        p.rect(7, 23, 2, 6, pants)
        p.rect(10, 23, 2, 5, pants)
        p.rect(6, 28, 3, 2, boot)
        p.rect(10, 27, 3, 2, boot)
    elif frame == 2:
        p.rect(7, 23, 2, 5, pants)
        p.rect(10, 23, 2, 6, pants)
        p.rect(7, 27, 3, 2, boot)
        p.rect(10, 28, 3, 2, boot)
    else:
        p.rect(7, 23, 2, 6, pants)
        p.rect(10, 23, 2, 6, pants)
        p.rect(6, 28, 3, 2, boot)
        p.rect(10, 28, 3, 2, boot)

    # torso, vest, apron
    p.rect(5, 11, 10, 12, shirt)
    p.rect(4, 11, 3, 10, vest)
    p.rect(13, 11, 3, 10, vest)
    p.rect(4, 12, 1, 8, vest_l)
    p.rect(15, 12, 1, 8, vest_l)
    p.rect(6, 17, 8, 6, apron)
    p.rect(6, 22, 8, 1, shirt_d)
    p.rect(8, 11, 4, 2, tie)
    p.rect(9, 13, 2, 1, tie)
    p.rect(4, 19, 12, 1, C("#1a141c"))

    # arms
    if frame == 3 or frame == 4:
        p.rect(2, 12, 3, 4, shirt)
        p.rect(2, 16, 2, 3, skin)
        p.rect(15, 9, 4, 3, shirt)
        p.rect(18, 10, 2, 2, skin)
        p.rect(15, 6, 3, 3, C("#e8a73c"))
        p.rect(15, 5, 3, 2, WHITE)
    elif frame == 5:
        p.rect(2, 8, 2, 6, shirt)
        p.rect(2, 5, 2, 3, skin)
        p.rect(16, 8, 2, 6, shirt)
        p.rect(16, 5, 2, 3, skin)
    else:
        p.rect(3, 12, 2, 6, shirt)
        p.rect(3, 17, 2, 3, skin)
        p.rect(15, 12, 2, 6, shirt)
        p.rect(15, 17, 2, 3, skin)

    # head, hair, moustache
    p.rect(6, 3, 8, 8, skin)
    p.rect(6, 8, 8, 2, skin_d)
    p.rect(6, 2, 8, 2, C("#241a14"))
    p.rect(5, 3, 1, 5, C("#241a14"))
    p.rect(14, 3, 1, 4, C("#241a14"))
    p.rect(9, 7, 1, 1, DARK)
    p.rect(12, 7, 1, 1, DARK)
    p.rect(9, 9, 4, 1, C("#4a2c1c"))
    p.rect(9, 10, 2, 1, C("#4a2c1c"))
    # collar + gold studs
    p.px(7, 12, gold)
    p.px(12, 12, gold)
    if frame == 5:
        p.rect(9, 7, 1, 1, tie)
        p.rect(12, 7, 1, 1, tie)
    return p


# --------------------------------------------------------------------------
# Props
# --------------------------------------------------------------------------


def keg(theme):
    p = Pix(26, 33)
    if theme == 0:
        body = C("#8a4a24")
        dark = C("#5a2c14")
        band = C("#b8a070")
        top = C("#d8c090")
        accent = C("#c0392b")
    elif theme == 1:
        body = C("#3a6e8a")
        dark = C("#204456")
        band = C("#c8d8e0")
        top = C("#9ab8c8")
        accent = C("#f2c94c")
    elif theme == 2:
        body = C("#4a4450")
        dark = C("#2a2430")
        band = C("#8a7a6a")
        top = C("#6a5a50")
        accent = C("#e24f9e")
    else:
        body = C("#3c2c62")
        dark = C("#241a3e")
        band = C("#6ad8b0")
        top = C("#a8f0d0")
        accent = C("#8affc8")
    p.rect(3, 6, 20, 27, body)
    p.rect(2, 8, 22, 3, dark)
    p.rect(2, 25, 22, 3, dark)
    p.rect(1, 29, 24, 3, dark)
    p.rect(4, 12, 18, 2, band)
    p.rect(4, 22, 18, 2, band)
    p.rect(4, 6, 18, 2, top)
    p.rect(6, 3, 14, 3, top)
    p.rect(8, 1, 10, 2, C("#fff4d8") if theme != 3 else accent)
    # tap toward the bar (left in art space)
    p.rect(0, 16, 4, 3, band)
    p.rect(0, 18, 2, 2, C("#e8c060") if theme == 0 else accent)
    p.rect(4, 17, 1, 1, dark)
    # emblem
    if theme == 0:
        p.disc(13, 17, 3, accent)
        p.rect(12, 15, 2, 4, C("#f4e0b2"))
    elif theme == 1:
        p.text("H", 11, 14, 1, accent)
        p.text("O", 11, 17, 1, accent)
    elif theme == 2:
        p.text("X", 11, 15, 1, accent)
        p.line(9, 19, 16, 13, accent)
    else:
        p.disc(13, 16, 3, accent)
        p.rect(12, 15, 3, 3, C("#10202a"))
        p.px(13, 15, C("#ffffff"))
    # foam / pressure valve
    if theme == 0:
        p.rect(7, 0, 2, 2, C("#fff4d8"))
        p.rect(12, 0, 2, 2, C("#fff4d8"))
    elif theme == 3:
        p.vline(6, 8, 24, C("#6ad8b0"))
        p.vline(19, 8, 24, C("#6ad8b0"))
    return p


def door(theme):
    p = Pix(24, 34)
    if theme == 0:
        frame = C("#5a3218")
        trim = C("#8a5a2a")
        inner = C("#2a1208")
        light = C("#e8a848")
    elif theme == 1:
        frame = C("#7a8288")
        trim = C("#c8d0d4")
        inner = C("#1a3038")
        light = C("#4fd8d2")
    elif theme == 2:
        frame = C("#3a3440")
        trim = C("#6a6070")
        inner = C("#120e18")
        light = C("#e24f9e")
    else:
        frame = C("#2a2044")
        trim = C("#6a58a0")
        inner = C("#0c0a1c")
        light = C("#6ad8b0")
    p.rect(1, 1, 22, 33, frame)
    p.rect(3, 4, 18, 28, inner)
    p.frame(2, 2, 20, 30, trim)
    if theme == 0:
        # swinging saloon doors
        p.rect(3, 14, 8, 17, C("#7a4420"))
        p.rect(13, 14, 8, 17, C("#7a4420"))
        p.rect(4, 16, 6, 12, C("#9a5a28"))
        p.rect(14, 16, 6, 12, C("#9a5a28"))
        p.rect(3, 20, 18, 1, trim)
        p.rect(3, 26, 18, 1, trim)
        p.rect(8, 22, 1, 5, C("#e8c060"))
        p.rect(15, 22, 1, 5, C("#e8c060"))
        p.text("SAL", 7, 6, 1, light)
    elif theme == 1:
        p.rect(4, 6, 16, 22, C("#3a4a52"))
        p.frame(4, 6, 16, 22, trim)
        p.text("H", 8, 9, 2, light)
        p.text("T", 8, 19, 2, C("#f2c94c"))
        p.rect(17, 24, 3, 1, trim)
    elif theme == 2:
        p.rect(4, 6, 16, 22, C("#2a2430"))
        for x in (6, 9, 12, 15, 18):
            p.px(x, 8, trim)
            p.px(x, 24, trim)
        p.text("PIT", 6, 12, 2, light)
        p.line(5, 22, 18, 27, C("#4fd8d2"))
        p.rect(17, 20, 2, 4, trim)
    else:
        p.rect(4, 5, 16, 24, C("#16243a"))
        p.frame(4, 5, 16, 24, trim)
        p.rect(10, 5, 1, 24, light)
        p.disc(12, 12, 2, light)
        p.rect(7, 26, 10, 1, light)
        p.px(17, 18, C("#ffffff"))
    return p


def mug(full):
    p = Pix(10, 12)
    glass = C("#cfe8f0")
    glass_d = C("#7aa8b8")
    beer = C("#e8a73c")
    beer_l = C("#f8d070")
    foam = C("#fff8e0")
    p.rect(1, 2, 7, 9, glass)
    p.rect(2, 2, 5, 9, beer if full else C("#2a2a32"))
    if full:
        p.rect(2, 4, 5, 6, beer)
        p.rect(2, 2, 5, 2, foam)
        p.rect(2, 3, 1, 1, foam)
        p.rect(4, 1, 2, 1, foam)
    else:
        p.rect(2, 8, 5, 2, C("#4a3a2a"))
        p.rect(2, 2, 5, 1, glass_d)
    p.frame(1, 2, 7, 9, glass)
    p.rect(8, 4, 2, 4, glass_d)
    p.px(9, 4, glass)
    p.px(9, 7, glass)
    if full:
        p.vline(2, 4, 6, beer_l)
    return p


def tip():
    p = Pix(12, 10)
    green = C("#4f8a52")
    green_d = C("#2f5a33")
    gold = C("#e8c060")
    p.rect(1, 2, 10, 7, green)
    p.frame(1, 2, 10, 7, green_d)
    p.text("$", 5, 3, 1, gold)
    p.px(2, 3, C("#8ac06a"))
    p.px(9, 7, C("#8ac06a"))
    return p


def can():
    p = Pix(18, 27)
    silver = C("#c8d0d8")
    silver_d = C("#8a96a2")
    red = C("#c8383f")
    red_d = C("#8a2228")
    p.rect(3, 3, 12, 21, silver)
    p.rect(4, 6, 10, 15, red)
    p.rect(4, 8, 10, 2, C("#f2e8d8"))
    p.rect(6, 12, 6, 1, C("#f2e8d8"))
    p.rect(4, 15, 10, 2, red_d)
    p.rect(3, 1, 12, 2, silver_d)
    p.rect(5, 0, 8, 1, C("#e8eef4"))
    p.rect(8, 1, 3, 1, silver_d)
    p.frame(3, 3, 12, 21, silver_d)
    p.vline(6, 6, 15, C("#e86a70"))
    return p


def arrow():
    p = Pix(16, 10)
    gold = C("#f2c94c")
    dark = C("#5a3a10")
    for i in range(5):
        p.hline(2 + i, 1 + i, 12 - i * 2, gold)
        p.hline(3 + i, 1 + i, 10 - i * 2, dark if i == 0 else gold)
    p.hline(2, 0, 12, dark)
    p.rect(7, 0, 2, 2, dark)
    p.hline(2, 6, 12, dark)
    return p


def shaker():
    p = Pix(30, 40)
    coat = C("#24202c")
    coat_l = C("#3c3648")
    hat = C("#1a1620")
    mask = C("#c8383f")
    skin = C("#e8b080")
    can_c = C("#c8d0d8")
    p.rect(7, 14, 16, 22, coat)
    p.rect(8, 15, 14, 1, coat_l)
    p.rect(7, 33, 7, 4, C("#120e16"))
    p.rect(16, 33, 7, 4, C("#120e16"))
    p.rect(4, 10, 7, 9, coat)
    p.rect(19, 10, 7, 9, coat)
    p.rect(3, 8, 24, 3, hat)
    p.rect(8, 2, 14, 7, hat)
    p.rect(8, 9, 14, 1, C("#3c3648"))
    p.rect(9, 15, 12, 6, skin)
    p.rect(9, 16, 12, 3, mask)
    p.rect(11, 16, 2, 1, C("#1a1620"))
    p.rect(17, 16, 2, 1, C("#1a1620"))
    p.rect(11, 19, 8, 1, C("#8a2228"))
    p.rect(20, 20, 6, 10, coat)
    p.rect(23, 24, 3, 10, can_c)
    p.rect(24, 23, 1, 1, C("#e8eef4"))
    p.rect(10, 25, 10, 3, C("#120e16"))
    p.px(12, 26, C("#e8c060"))
    p.px(17, 26, C("#e8c060"))
    return p


def dancer(theme):
    p = Pix(18, 26)
    if theme == 0:
        skin, dress, hair, boot = C("#f2c49b"), C("#c0474f"), C("#e4b34e"), C("#f4efe2")
    elif theme == 1:
        skin, dress, hair, boot = C("#f2c49b"), C("#e8e2d2"), C("#e8c85a"), C("#f2eee0")
    elif theme == 2:
        skin, dress, hair, boot = C("#d99a6c"), C("#e24f9e"), C("#4fd8d2"), C("#1d1a24")
    else:
        skin, dress, hair, boot = C("#8ad06a"), C("#6a58a0"), C("#8affc8"), C("#241a3e")
    # raised arms
    p.rect(3, 4, 2, 8, skin)
    p.rect(13, 4, 2, 8, skin)
    p.rect(2, 2, 3, 3, dress)
    p.rect(13, 2, 3, 3, dress)
    # head + hair
    p.rect(6, 4, 6, 6, skin)
    p.rect(5, 3, 8, 2, hair)
    p.rect(4, 4, 2, 6, hair)
    p.rect(12, 4, 2, 6, hair)
    p.px(8, 7, DARK)
    p.px(10, 7, DARK)
    p.rect(8, 9, 2, 1, C("#8a3a3a"))
    # dress + legs
    p.rect(5, 10, 8, 7, dress)
    p.rect(4, 16, 10, 3, dress)
    p.rect(6, 19, 2, 5, skin)
    p.rect(10, 19, 2, 5, skin)
    p.rect(5, 23, 3, 2, boot)
    p.rect(10, 23, 3, 2, boot)
    p.rect(5, 11, 8, 1, C("#fff4d8"))
    return p


def logo():
    p = Pix(280, 84)
    # A single brass-edged plaque so the wordmark reads as one sign.
    p.rect(2, 2, 276, 80, C("#2a0c08"))
    p.rect(5, 5, 270, 74, C("#5a1810"))
    p.frame(2, 2, 276, 80, C("#c88a3a"))
    p.frame(5, 5, 270, 74, C("#8a4a20"))
    p.text_outlined(
        "TAPPER",
        35,
        9,
        6,
        C("#f8d878"),
        C("#3a1208"),
        C("#8a2a18"),
    )
    p.hline(18, 55, 244, C("#c88a3a"))
    p.hline(18, 56, 244, C("#6a3a18"))
    p.text_outlined(
        "LAST CALL",
        60,
        59,
        3,
        C("#f4efe2"),
        C("#3a1208"),
        C("#8a2a18"),
    )
    return p


# --------------------------------------------------------------------------
# Backgrounds
# --------------------------------------------------------------------------


def bar(p, y, top, edge, front, dark, metal=False):
    p.rect(0, y, 320, 4, top)
    p.hline(0, y, 320, edge)
    p.rect(0, y + 4, 320, 10, front)
    p.rect(0, y + 14, 320, 2, dark)
    if metal:
        for x in range(4, 320, 22):
            p.rect(x, y + 5, 2, 8, dark)
            p.px(x + 1, y + 6, edge)
    else:
        for x in range(6, 320, 26):
            p.rect(x, y + 5, 1, 8, dark)
            p.rect(x + 1, y + 5, 18, 1, front)
    p.hline(0, y + 15, 320, C("#000000"))


def saloon_bg():
    p = Pix(320, 240)
    wall = C("#2a160e")
    wood = C("#4a2814")
    wood_l = C("#64381c")
    p.rect(0, 0, 320, 240, wall)
    for y in range(0, 240, 14):
        p.hline(0, y, 320, C("#1c0e08"))
        p.hline(0, y + 1, 320, wood)
        for x in range((y * 7) % 31, 320, 61):
            p.rect(x, y + 1, 1, 13, C("#1c0e08"))
    # sign
    p.rect(72, 22, 176, 34, C("#1a0c06"))
    p.rect(75, 25, 170, 28, C("#5a2c14"))
    p.frame(78, 28, 164, 22, C("#c8964a"))
    p.text_outlined("DUSTY SPUR", 92, 32, 2, C("#f8d878"), C("#3a1208"))
    p.disc(60, 30, 4, C("#e8a848"))
    p.disc(260, 30, 4, C("#e8a848"))
    # shelves and bottles in the gaps between bars
    for y in (106, 152, 198):
        p.rect(10, y, 42, 2, C("#8a5a2a"))
        for i, col in enumerate(["#4f8a52", "#c0474f", "#e8c060", "#3f6fb5"]):
            p.rect(14 + i * 9, y - 7, 4, 7, C(col))
            p.rect(15 + i * 9, y - 10, 2, 3, C("#d8d0c0"))
        p.rect(230, y - 18, 44, 28, C("#d8c8a8"))
        p.text("WANTED", 235, y - 15, 1, C("#4a2010"))
        p.rect(244, y - 8, 14, 8, C("#a87a52"))
        p.px(248, y - 6, DARK)
        p.px(253, y - 6, DARK)
    # mission-style lamps above the top bar
    for x in (100, 220):
        p.vline(x, 40, 12, C("#1a0c06"))
        p.rect(x - 4, 52, 9, 6, C("#f2b04a"))
        p.rect(x - 5, 51, 11, 2, C("#5a2c14"))
    # bars
    for i, y in enumerate((66, 112, 158, 204)):
        bar(p, y, C("#8a4a22"), C("#d89a4a"), C("#5a2c14"), C("#2a1208"))
        for x in range(0, 320, 26):
            p.rect(x, y + 5, 1, 8, C("#3a1808"))
        p.hline(0, y + 4, 320, C("#b0703a"))
    # floor
    p.rect(0, 218, 320, 22, C("#1a0c08"))
    for x in range(0, 320, 20):
        p.vline(x, 218, 22, C("#120806"))
    p.hline(0, 219, 320, C("#3a1c10"))
    return p


def sports_bg():
    p = Pix(320, 240)
    p.rect(0, 0, 320, 240, C("#0e2a2a"))
    for y in range(0, 240, 16):
        p.hline(0, y, 320, C("#0a2020"))
    # scoreboard
    p.rect(96, 22, 128, 38, C("#1a1418"))
    p.frame(99, 25, 122, 32, C("#c8d0d4"))
    p.text_outlined("HOME 42", 110, 30, 2, C("#f2c94c"), C("#3a1a08"))
    p.text_outlined("AWAY 38", 110, 46, 1, C("#4fd8d2"), C("#0a2020"))
    # pennants
    for i, col in enumerate(["#4f8a52", "#c0474f", "#3f6fb5", "#f2c94c"]):
        x = 18 + i * 12
        p.line(x, 14, x + 8, 24, C(col))
        p.line(x + 8, 24, x, 32, C(col))
        p.rect(x, 14, 3, 2, C(col))
    for i, col in enumerate(["#f2c94c", "#4fd8d2", "#c0474f"]):
        x = 262 + i * 12
        p.line(x, 14, x + 8, 24, C(col))
        p.line(x + 8, 24, x, 32, C(col))
    # neon sign
    p.rect(14, 42, 50, 15, C("#0a1414"))
    p.text("BAR", 24, 45, 1, C("#4fd8d2"))
    p.rect(256, 42, 50, 15, C("#0a1414"))
    p.text("TV", 268, 45, 2, C("#e24f9e"))
    # lockers / TVs
    for i in range(3):
        x = 18 + i * 22
        p.rect(x, 134, 17, 20, C("#2a4a4a"))
        p.frame(x, 134, 17, 20, C("#4a7a7a"))
        p.rect(x + 6, 137, 1, 3, C("#c8d0d4"))
    p.rect(268, 134, 34, 20, C("#101c20"))
    p.frame(268, 134, 34, 20, C("#4a7a7a"))
    p.rect(271, 137, 28, 14, C("#2a6a7a"))
    p.text("12", 278, 139, 2, C("#f2c94c"))
    # bars
    for y in (66, 112, 158, 204):
        bar(p, y, C("#5a8a7a"), C("#a8e8d0"), C("#244a44"), C("#0a2020"), metal=True)
        p.hline(0, y + 4, 320, C("#4fd8d2"))
    p.rect(0, 218, 320, 22, C("#0a1c1c"))
    for x in range(0, 320, 18):
        p.hline(x, 230, 10, C("#143030"))
    return p


def punk_bg():
    p = Pix(320, 240)
    p.rect(0, 0, 320, 240, C("#1a1420"))
    # brick wall
    for y in range(0, 240, 12):
        p.hline(0, y, 320, C("#0e0a12"))
        for x in range((y // 12 % 2) * 12, 320, 24):
            p.vline(x, y, 12, C("#0e0a12"))
            p.hline(x + 1, y + 1, 22, C("#241a2c"))
    # graffiti and posters
    p.text_outlined("NO RULES", 20, 28, 2, C("#e24f9e"), C("#2a0c1c"))
    p.text_outlined("PIT", 236, 24, 3, C("#4fd8d2"), C("#0c2024"))
    p.rect(246, 44, 42, 32, C("#e8e2d2"))
    p.rect(249, 47, 36, 26, C("#2a2430"))
    p.text("LIVE", 256, 52, 1, C("#e24f9e"))
    p.text("TONITE", 253, 60, 1, C("#f2c94c"))
    # chain link in the aisle between the top two bars
    for x in range(4, 70, 8):
        p.line(x, 134, x + 24, 154, C("#5a5060"))
        p.line(x + 24, 134, x, 154, C("#5a5060"))
    # neon tubes
    p.vline(14, 90, 60, C("#e24f9e"))
    p.vline(306, 90, 60, C("#4fd8d2"))
    p.rect(13, 88, 3, 3, C("#ffd0f0"))
    p.rect(305, 148, 3, 3, C("#d0fff8"))
    # bars
    for y in (66, 112, 158, 204):
        bar(p, y, C("#4a4450"), C("#e24f9e"), C("#26202c"), C("#0e0a12"), metal=True)
        p.hline(0, y + 4, 320, C("#4fd8d2"))
        for x in range(8, 320, 34):
            p.px(x, y + 7, C("#8a8090"))
    p.rect(0, 218, 320, 22, C("#0e0a12"))
    for x in range(0, 320, 16):
        p.line(x, 218, x + 8, 240, C("#241a2c"))
    return p


def alien_bg():
    p = Pix(320, 240)
    p.rect(0, 0, 320, 240, C("#080a20"))
    rnd = random.Random(7)
    for _ in range(130):
        x, y = rnd.randrange(320), rnd.randrange(200)
        c = C("#cfe8ff") if rnd.random() < 0.25 else C("#5a6a9a")
        p.px(x, y, c)
    # porthole with planet
    p.disc(160, 34, 30, C("#1a2450"))
    p.disc(160, 34, 27, C("#0c1230"))
    p.disc(168, 30, 18, C("#7a5ac8"))
    p.disc(174, 26, 12, C("#a888e8"))
    for a in range(0, 360, 12):
        x = 160 + int(math.cos(math.radians(a)) * 29)
        y = 34 + int(math.sin(math.radians(a)) * 29)
        p.px(x, y, C("#4a6a9a"))
    p.text_outlined("LAST ROUND", 14, 28, 2, C("#8affc8"), C("#10202a"))
    # glowing tubes
    for x in (12, 308):
        p.vline(x, 80, 110, C("#6ad8b0"))
        for y in range(84, 190, 14):
            p.rect(x - 2, y, 5, 3, C("#2a8a6a"))
    # organic shelves
    for y in (126, 172):
        p.rect(20, y, 40, 3, C("#3c2c62"))
        for i, col in enumerate(["#6ad8b0", "#b07ad8", "#e24f4f"]):
            p.disc(28 + i * 12, y - 3, 3, C(col))
    # bars
    for y in (66, 112, 158, 204):
        bar(p, y, C("#3c2c62"), C("#8affc8"), C("#241a3e"), C("#0c0a1c"), metal=True)
        p.hline(0, y + 4, 320, C("#8affc8"))
        p.hline(0, y + 13, 320, C("#6a58a0"))
    p.rect(0, 218, 320, 22, C("#0c0a1c"))
    p.hline(0, 225, 320, C("#241a3e"))
    return p


def background(theme):
    return [saloon_bg, sports_bg, punk_bg, alien_bg][theme]()


# --------------------------------------------------------------------------
# Export
# --------------------------------------------------------------------------


def hide_background(doc):
    """Krita adds an opaque white Background layer; hide it to keep alpha."""
    for node in doc.rootNode().childNodes():
        if node.name() == "Background":
            node.setVisible(False)


def main():
    # Sprite board layout.
    board = Pix(512, 330)
    regions = {}
    for theme in range(4):
        sheet = Pix(54, 104)
        for variant in range(4):
            for frame in range(3):
                sheet.blit(patron(theme, variant, frame), frame * 18, variant * 26)
        x = theme * 58
        board.blit(sheet, x, 0)
        regions[f"patron_{THEMES[theme]}"] = (x, 0, 54, 104)

    board.blit(bartender(0), 0, 110)
    regions["bartender"] = (0, 110, 120, 30)
    for f in range(1, 6):
        board.blit(bartender(f), f * 20, 110)

    for i, name in enumerate(THEMES):
        x = 130 + i * 30
        board.blit(keg(i), x, 110)
        regions[f"keg_{name}"] = (x, 110, 26, 33)
        x = 130 + i * 28
        board.blit(door(i), x, 148)
        regions[f"door_{name}"] = (x, 148, 24, 34)

    board.blit(mug(True), 0, 148)
    regions["mug_full"] = (0, 148, 10, 12)
    board.blit(mug(False), 13, 148)
    regions["mug_empty"] = (13, 148, 10, 12)
    board.blit(tip(), 26, 148)
    regions["tip"] = (26, 148, 12, 10)
    board.blit(can(), 42, 148)
    regions["can"] = (42, 148, 18, 27)
    board.blit(arrow(), 64, 148)
    regions["arrow"] = (64, 148, 16, 10)
    board.blit(shaker(), 84, 148)
    regions["shaker"] = (84, 148, 30, 40)

    for i, name in enumerate(THEMES):
        x = i * 20
        board.blit(dancer(i), x, 190)
        regions[f"dancer_{name}"] = (x, 190, 18, 26)

    board.blit(logo(), 0, 224)
    regions["logo"] = (0, 224, 280, 84)

    doc = ks.new_doc(512, 330, "sprite_board")
    hide_background(doc)
    lay = ks.new_layer(doc, "sprites")
    ks.write_pixels(lay, board.img)
    ks.save_kra(doc, os.path.join(ART, "sprite_board.kra"))

    doc = ks.new_doc(512, 330, "sprite_board_export")
    hide_background(doc)
    lay = ks.new_layer(doc, "sprites")
    ks.write_pixels(lay, board.img)
    for name, (x, y, w, h) in regions.items():
        ks.export_region_png(doc, x, y, w, h, os.path.join(SPRITES, f"{name}.png"))
    ks.log(f"sprites: {len(regions)} PNGs")

    strip = Pix(1280, 240)
    for i in range(4):
        strip.blit(background(i), i * 320, 0)
    doc = ks.new_doc(1280, 240, "backgrounds")
    hide_background(doc)
    lay = ks.new_layer(doc, "themes")
    ks.write_pixels(lay, strip.img)
    ks.save_kra(doc, os.path.join(ART, "backgrounds.kra"))
    for i, name in enumerate(THEMES):
        ks.export_region_png(
            doc, i * 320, 0, 320, 240, os.path.join(BACKGROUNDS, f"bg_{name}.png")
        )
    ks.log("backgrounds: 4 PNGs")

    # A contact sheet for visual review.
    contact = Pix(560, 430)
    contact.rect(0, 0, 560, 430, C("#202028"))
    contact.blit(board, 20, 20)
    doc = ks.new_doc(560, 430, "contact")
    hide_background(doc)
    lay = ks.new_layer(doc, "contact")
    ks.write_pixels(lay, contact.img)
    ks.save_kra(doc, os.path.join(ART, "sprite_contact.kra"))
    ks.export_png(doc, os.path.join(ART, "sprite_contact.png"))

    ks.done()


def __main__(args):
    try:
        main()
    except BaseException:
        traceback.print_exc(file=sys.stderr)
        raise
