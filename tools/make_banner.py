#!/usr/bin/env python3
"""Compose the README/OpenGraph banner and landing-page thumbnail from shots.

Requires Pillow and a current screenshots set; run after `shots/` is captured.
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
SHOTS = ROOT / "shots"
ART = ROOT / "assets" / "art"
ART.mkdir(parents=True, exist_ok=True)

THEMES = [
    ("saloon", "SALOON"),
    ("sports", "SPORTS BAR"),
    ("punk", "PUNK BAR"),
    ("alien", "ALIEN BAR"),
]


def font(size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(ROOT / "assets" / "fonts" / "Saloon17.ttf"), size)


def mono(size: int) -> ImageFont.FreeTypeFont:
    for candidate in (
        "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
    ):
        if Path(candidate).exists():
            return ImageFont.truetype(candidate, size)
    return ImageFont.load_default()


def main():
    canvas = Image.new("RGBA", (1200, 750), (11, 7, 13, 255))
    draw = ImageDraw.Draw(canvas, "RGBA")

    logo = Image.open(ROOT / "assets" / "sprites" / "logo.png")
    logo = logo.convert("RGBA").resize((340, 102), Image.Resampling.NEAREST)
    canvas.paste(logo, (24, 38), logo)

    draw.text((26, 168), "FOUR BARS.", font=font(26), fill=(247, 234, 208, 255))
    draw.text((26, 200), "FOUR CROWDS.", font=font(26), fill=(242, 168, 60, 255))
    draw.text((26, 232), "ONE BARTENDER.", font=font(26), fill=(247, 234, 208, 255))
    draw.text((28, 292), "SERVE / CATCH / TIP / SURVIVE", font=font(17), fill=(82, 224, 207, 255))

    for index, (name, label) in enumerate(THEMES):
        shot = Image.open(SHOTS / f"{name}.png").convert("RGB").resize(
            (390, 293), Image.Resampling.NEAREST
        )
        x = 380 + (index % 2) * 400
        y = 50 + (index // 2) * 350
        canvas.paste(shot, (x, y))
        draw.rectangle([x - 2, y - 2, x + 391, y + 294], outline=(242, 168, 60, 120), width=2)
        draw.rectangle([x, y + 270, x + 390, y + 293], fill=(9, 6, 11, 190))
        draw.text((x + 10, y + 273), label, font=font(17), fill=(247, 234, 208, 255))

    draw.text((26, 650), "PLAY FREE IN YOUR BROWSER", font=font(17), fill=(247, 234, 208, 255))
    draw.text(
        (26, 690),
        "RUST + BEVY 0.20.1  /  ORIGINAL KRITA ART",
        font=mono(13),
        fill=(167, 154, 168, 255),
    )
    draw.rectangle([0, 0, 1199, 749], outline=(242, 168, 60, 120), width=2)

    canvas.convert("RGB").save(ART / "banner.png")
    canvas.convert("RGB").save(ART / "tapper-thumb.webp", quality=90)
    print("wrote assets/art/banner.png and assets/art/tapper-thumb.webp")


if __name__ == "__main__":
    main()
