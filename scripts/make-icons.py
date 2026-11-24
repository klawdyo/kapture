#!/usr/bin/env python3
"""Gera todos os derivados da logo a partir das logos transparentes em assets/.

Uso (na raiz do repositório):
    pip install pillow
    python scripts/make-icons.py

Entradas : assets/logo_light_transparent.png e assets/logo_dark_transparent.png (800x800, RGBA, fundo transparente).
Saídas   : src-tauri/icons, packaging/msix/Assets, site/public e src/assets/logo.png.
É idempotente: sempre regrava os mesmos arquivos a partir das duas logos (sem acumular perda).
Não mexe em assets/store/* nem na imagem original do Gemini.

Escolha da versão: os ícones do app, do MSIX e a barra do app usam a versão CLARA, que
mantém contraste na barra de tarefas escura (padrão do Windows) e nos blocos do menu Iniciar.
O site e o README usam as duas, escolhidas pelo tema.
"""
from __future__ import annotations

import io
import struct
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
ICONS = ROOT / "src-tauri" / "icons"
MSIX = ROOT / "packaging" / "msix" / "Assets"
SITE = ROOT / "site" / "public"
APP_LOGO = ROOT / "src" / "assets" / "logo.png"

# Margem de segurança (fração do lado) em volta da arte nos ícones.
ICON_MARGIN = 0.04
# Cor de fundo (grafite) do og.png.
GRAPHITE = (24, 25, 29)
ICO_SIZES = (16, 24, 32, 48, 64, 128, 256)


def load(name: str) -> Image.Image:
    """Abre a logo e recorta o espaço transparente em volta (a arte vira o quadrado todo)."""
    img = Image.open(ASSETS / name).convert("RGBA")
    box = img.getchannel("A").getbbox()
    return img.crop(box) if box else img


def fit(art: Image.Image, canvas: tuple[int, int], height_ratio: float) -> Image.Image:
    """Centraliza a arte, com altura = height_ratio * altura do canvas, sobre fundo transparente."""
    width, height = canvas
    side = max(1, round(min(width, height) * height_ratio))
    scaled = art.resize((side, side), Image.LANCZOS)
    out = Image.new("RGBA", canvas, (0, 0, 0, 0))
    out.paste(scaled, ((width - side) // 2, (height - side) // 2), scaled)
    return out


def square(art: Image.Image, size: int, margin: float = ICON_MARGIN) -> Image.Image:
    return fit(art, (size, size), 1 - 2 * margin)


def save_png(img: Image.Image, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    img.save(path, optimize=True)


def png_bytes(img: Image.Image) -> bytes:
    buf = io.BytesIO()
    img.save(buf, format="PNG", optimize=True)
    return buf.getvalue()


def write_ico(art: Image.Image, path: Path) -> None:
    """ICO multi-tamanho com PNG comprimido em cada entrada (cada tamanho vem da arte original)."""
    blobs = [(s, png_bytes(square(art, s))) for s in ICO_SIZES]
    header = struct.pack("<HHH", 0, 1, len(blobs))
    offset = 6 + 16 * len(blobs)
    entries, data = b"", b""
    for size, blob in blobs:
        dim = 0 if size >= 256 else size  # 0 significa 256 no formato ICO
        entries += struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(blob), offset)
        data += blob
        offset += len(blob)
    path.write_bytes(header + entries + data)


def make_og(art: Image.Image) -> Image.Image:
    """1200x630: logo centralizada sobre grafite com um halo vermelho discreto, sem texto."""
    w, h = 1200, 630
    bg = Image.new("RGB", (w, h), GRAPHITE).convert("RGBA")
    glow = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    ImageDraw.Draw(glow).ellipse((w / 2 - 330, h / 2 - 330, w / 2 + 330, h / 2 + 330), fill=(250, 52, 74, 36))
    bg.alpha_composite(glow.filter(ImageFilter.GaussianBlur(120)))
    bg.alpha_composite(fit(art, (w, h), 0.70))
    return bg.convert("RGB")


def main() -> None:
    light, dark = load("logo_light_transparent.png"), load("logo_dark_transparent.png")

    # Ícones do Tauri (versão clara, com margem pequena).
    for name, size in (("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256), ("icon.png", 512)):
        save_png(square(light, size), ICONS / name)
    write_ico(light, ICONS / "icon.ico")

    # MSIX: fundo transparente; Square* usam ~80% do lado, o Wide usa ~80% da altura.
    save_png(square(light, 50, 0.04), MSIX / "StoreLogo.png")
    save_png(square(light, 44, 0.04), MSIX / "Square44x44.png")
    save_png(fit(light, (71, 71), 0.84), MSIX / "Square71x71.png")
    save_png(fit(light, (150, 150), 0.80), MSIX / "Square150x150.png")
    save_png(fit(light, (310, 150), 0.80), MSIX / "Wide310x150.png")
    save_png(fit(light, (310, 310), 0.80), MSIX / "Square310x310.png")

    # Site: 64 (cabeçalho a 32 px, tela 2x), 128 (uso a 64 px) e 480 (hero a 320 px, 1,5x).
    for variant, art in (("light", light), ("dark", dark)):
        for size, suffix in ((64, "-64"), (128, "-128"), (480, "")):
            save_png(art.resize((size, size), Image.LANCZOS), SITE / f"logo-{variant}{suffix}.png")
    save_png(square(light, 48, 0.02), SITE / "favicon.png")
    og = make_og(light)
    og.save(SITE / "og.png", optimize=True)

    # Barra do app (exibida a ~40-46 px; 128 cobre 2x): versão clara sobre a barra escura.
    save_png(square(light, 128, 0.02), APP_LOGO)


if __name__ == "__main__":
    main()
