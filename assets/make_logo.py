"""Gera o logo do Cap (obturador de 4 lâminas) em SVG e os ícones PNG/ICO.

Uso: python3 assets/make_logo.py   (requer Pillow)
Sem dependência de rede nem de cairosvg: desenha com Pillow em supersampling.
"""
import math
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
RED = "#EF4444"
TEMAS = {
    "light": {"bg": "#F8FAFC", "blade": "#2B2F36"},
    "dark": {"bg": "#14161A", "blade": "#8A93A3"},
}
# Lâmina base (grade 512) e rotações de 90 graus
BLADE = [(232, 70), (392, 138), (338, 238), (232, 178)]
C = 256


def rot(p, deg):
    a = math.radians(deg)
    x, y = p[0] - C, p[1] - C
    return (C + x * math.cos(a) - y * math.sin(a), C + x * math.sin(a) + y * math.cos(a))


def svg(t):
    polys = "\n".join(
        f'  <polygon points="{" ".join(f"{x:g},{y:g}" for x, y in BLADE)}" '
        f'fill="{t["blade"]}" transform="rotate({d} 256 256)"/>'
        for d in (0, 90, 180, 270)
    )
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">\n'
        f'  <rect width="512" height="512" rx="112" fill="{t["bg"]}"/>\n{polys}\n'
        f'  <circle cx="256" cy="256" r="34" fill="{RED}"/>\n</svg>\n'
    )


def render(t, size, pad=0.0):
    k = 4
    s = 512 * k
    im = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle([0, 0, s - 1, s - 1], radius=112 * k, fill=t["bg"])
    for deg in (0, 90, 180, 270):
        d.polygon([(x * k, y * k) for x, y in (rot(p, deg) for p in BLADE)], fill=t["blade"])
    d.ellipse([(C - 34) * k, (C - 34) * k, (C + 34) * k, (C + 34) * k], fill=RED)
    if pad:
        canvas = Image.new("RGBA", (s, s), (0, 0, 0, 0))
        n = int(s * (1 - 2 * pad))
        canvas.paste(im.resize((n, n), Image.LANCZOS), ((s - n) // 2, (s - n) // 2))
        im = canvas
    return im.resize((size, size), Image.LANCZOS)


def wide(t, w, h):
    im = Image.new("RGBA", (w, h), t["bg"])
    ico = render(t, int(h * 0.8))
    im.alpha_composite(ico, ((w - ico.width) // 2, (h - ico.height) // 2))
    return im


def main():
    (ROOT / "assets").mkdir(exist_ok=True)
    for nome, t in TEMAS.items():
        (ROOT / "assets" / f"logo-{nome}.svg").write_text(svg(t), encoding="utf-8")
    t = TEMAS["light"]
    ic = ROOT / "src-tauri" / "icons"
    ic.mkdir(parents=True, exist_ok=True)
    for nome, sz in (("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256), ("icon.png", 512)):
        render(t, sz).save(ic / nome, optimize=True)
    base = render(t, 256)
    base.save(ic / "icon.ico", format="ICO", sizes=[(s, s) for s in (16, 24, 32, 48, 64, 128, 256)])
    a = ROOT / "packaging" / "msix" / "Assets"
    a.mkdir(parents=True, exist_ok=True)
    render(t, 50).save(a / "StoreLogo.png", optimize=True)
    render(t, 44).save(a / "Square44x44.png", optimize=True)
    render(t, 150).save(a / "Square150x150.png", optimize=True)
    wide(t, 310, 150).save(a / "Wide310x150.png", optimize=True)


if __name__ == "__main__":
    main()
