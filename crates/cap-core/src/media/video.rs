//! Funções puras de vídeo: recorte, conversão de cor, ajuste de tamanho e bitrate.

use crate::domain::{Monitor, Region};

/// Arredonda para baixo até um número par (NV12/H.264 exigem lados pares).
pub fn even(n: u32) -> u32 {
    n & !1
}

/// Recorta um quadro BGRA (com `stride` em bytes por linha) para `r`.
///
/// A região é limitada aos limites do quadro e os lados ficam pares.
/// Devolve `(largura, altura, bytes compactados)` ou `None` se sobrar
/// menos de 2x2 pixels. O recorte nunca lê fora de `src` se
/// `src.len() >= stride * sh`.
pub fn crop_bgra(
    src: &[u8],
    stride: usize,
    sw: u32,
    sh: u32,
    r: &Region,
) -> Option<(u32, u32, Vec<u8>)> {
    // Conta em i64 para que regiões enormes ou negativas não estourem.
    let x0 = (r.x as i64).clamp(0, sw as i64);
    let y0 = (r.y as i64).clamp(0, sh as i64);
    let x1 = (r.x as i64 + r.w as i64).clamp(0, sw as i64);
    let y1 = (r.y as i64 + r.h as i64).clamp(0, sh as i64);
    let w = even((x1 - x0) as u32);
    let h = even((y1 - y0) as u32);
    if w == 0 || h == 0 {
        return None;
    }
    let row = w as usize * 4;
    let mut out = Vec::with_capacity(row * h as usize);
    for y in 0..h as usize {
        let at = (y0 as usize + y) * stride + x0 as usize * 4;
        out.extend_from_slice(&src[at..at + row]);
    }
    Some((w, h, out))
}

/// Converte BGRA compacto (`w` e `h` pares) para NV12 (BT.709, faixa limitada).
///
/// Saída: plano Y (`w*h`) seguido do plano UV intercalado (`w*h/2`),
/// com a crominância calculada pela média de cada bloco 2x2.
pub fn bgra_to_nv12(src: &[u8], w: u32, h: u32) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let mut out = vec![0u8; w * h * 3 / 2];
    let (yp, uv) = out.split_at_mut(w * h);
    // Luma de cada pixel.
    for (i, px) in src.chunks_exact(4).take(w * h).enumerate() {
        let (b, g, r) = (px[0] as i32, px[1] as i32, px[2] as i32);
        yp[i] = (((47 * r + 157 * g + 16 * b + 128) >> 8) + 16) as u8;
    }
    // Crominância: média do bloco 2x2.
    for by in 0..h / 2 {
        for bx in 0..w / 2 {
            let (mut sb, mut sg, mut sr) = (0i32, 0i32, 0i32);
            for (dy, dx) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                let at = ((by * 2 + dy) * w + bx * 2 + dx) * 4;
                sb += src[at] as i32;
                sg += src[at + 1] as i32;
                sr += src[at + 2] as i32;
            }
            let (b, g, r) = ((sb + 2) / 4, (sg + 2) / 4, (sr + 2) / 4);
            let at = by * w + bx * 2;
            uv[at] = (((-26 * r - 86 * g + 112 * b + 128) >> 8) + 128) as u8;
            uv[at + 1] = (((112 * r - 102 * g - 10 * b + 128) >> 8) + 128) as u8;
        }
    }
    out
}

/// Encaixa um quadro BGRA compacto `sw x sh` em `dw x dh` sem escalar:
/// copia a interseção a partir do canto superior esquerdo e preenche o
/// resto com preto. Usado quando a janela capturada muda de tamanho.
pub fn fit_bgra(src: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    let mut out = vec![0u8; dw as usize * dh as usize * 4];
    let cw = sw.min(dw) as usize * 4;
    for y in 0..sh.min(dh) as usize {
        let (s, d) = (y * sw as usize * 4, y * dw as usize * 4);
        out[d..d + cw].copy_from_slice(&src[s..s + cw]);
    }
    // Alfa opaco nos pixels pretos de preenchimento.
    for px in out.chunks_exact_mut(4) {
        px[3] = 255;
    }
    out
}

/// Empacota dois u32 num u64 (alto, baixo): formato dos atributos de
/// tamanho (`MF_MT_FRAME_SIZE`) e razão (`MF_MT_FRAME_RATE`) do Media Foundation.
pub fn pack(hi: u32, lo: u32) -> u64 {
    ((hi as u64) << 32) | lo as u64
}

/// Bitrate de vídeo (bits/s): ~0,07 bit por pixel por quadro, entre 1 e 16 Mbps.
pub fn video_bitrate(w: u32, h: u32, fps: u32) -> u32 {
    let bps = w as u64 * h as u64 * fps as u64 * 7 / 100;
    bps.clamp(1_000_000, 16_000_000) as u32
}

/// Escolhe o monitor que contém o centro da região (senão o principal,
/// senão o primeiro).
pub fn pick_monitor<'a>(r: &Region, mons: &'a [Monitor]) -> Option<&'a Monitor> {
    let cx = r.x as i64 + r.w as i64 / 2;
    let cy = r.y as i64 + r.h as i64 / 2;
    let hit = mons.iter().find(|m| {
        cx >= m.x as i64
            && cx < m.x as i64 + m.w as i64
            && cy >= m.y as i64
            && cy < m.y as i64 + m.h as i64
    });
    hit.or_else(|| mons.iter().find(|m| m.primary))
        .or_else(|| mons.first())
}

/// Converte a região (coordenadas da área de trabalho virtual) para
/// coordenadas locais do monitor.
pub fn rel_region(r: &Region, m: &Monitor) -> Region {
    Region {
        x: r.x - m.x,
        y: r.y - m.y,
        w: r.w,
        h: r.h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mon(id: u32, x: i32, y: i32, primary: bool) -> Monitor {
        Monitor {
            id,
            name: format!("M{id}"),
            w: 100,
            h: 50,
            x,
            y,
            primary,
        }
    }

    /// Quadro BGRA sintético em que cada pixel guarda seu índice no canal B.
    fn ramp(w: u32, h: u32) -> Vec<u8> {
        (0..w * h).flat_map(|i| [i as u8, 0, 0, 255]).collect()
    }

    #[test]
    fn lados_pares() {
        assert_eq!(even(7), 6);
        assert_eq!(even(8), 8);
        assert_eq!(even(0), 0);
    }

    #[test]
    fn recorte_basico_com_stride() {
        // Quadro 4x3 com stride maior que a largura (padding de 8 bytes).
        let stride = 4 * 4 + 8;
        let mut src = vec![0u8; stride * 3];
        for y in 0..3 {
            for x in 0..4 {
                src[y * stride + x * 4] = (y * 4 + x) as u8;
            }
        }
        let r = Region {
            x: 1,
            y: 1,
            w: 2,
            h: 2,
        };
        let (w, h, d) = crop_bgra(&src, stride, 4, 3, &r).unwrap();
        assert_eq!((w, h), (2, 2));
        let bs: Vec<u8> = d.chunks(4).map(|p| p[0]).collect();
        assert_eq!(bs, vec![5, 6, 9, 10]);
    }

    #[test]
    fn recorte_limita_e_arredonda() {
        let src = ramp(6, 4);
        // Passa da borda direita/inferior e começa fora da esquerda/topo.
        let r = Region {
            x: -3,
            y: -3,
            w: 100,
            h: 100,
        };
        let (w, h, d) = crop_bgra(&src, 24, 6, 4, &r).unwrap();
        assert_eq!((w, h), (6, 4));
        assert_eq!(d.len(), 6 * 4 * 4);
        // Lado ímpar vira par.
        let r = Region {
            x: 0,
            y: 0,
            w: 5,
            h: 3,
        };
        let (w, h, _) = crop_bgra(&src, 24, 6, 4, &r).unwrap();
        assert_eq!((w, h), (4, 2));
    }

    #[test]
    fn recorte_vazio() {
        let src = ramp(4, 4);
        let fora = Region {
            x: 10,
            y: 0,
            w: 5,
            h: 5,
        };
        assert!(crop_bgra(&src, 16, 4, 4, &fora).is_none());
        let fino = Region {
            x: 0,
            y: 0,
            w: 1,
            h: 4,
        };
        assert!(crop_bgra(&src, 16, 4, 4, &fino).is_none());
        let baixo = Region {
            x: 0,
            y: 0,
            w: 4,
            h: 1,
        };
        assert!(crop_bgra(&src, 16, 4, 4, &baixo).is_none());
    }

    #[test]
    fn nv12_cores_conhecidas() {
        // Branco, preto e cinza uniformes (BGRA).
        for (v, y, c) in [(255u8, 235u8, 128u8), (0, 16, 128)] {
            let src: Vec<u8> = (0..4).flat_map(|_| [v, v, v, 255]).collect();
            let o = bgra_to_nv12(&src, 2, 2);
            assert_eq!(o.len(), 6);
            assert!(o[..4].iter().all(|&p| p == y), "Y de {v}");
            assert_eq!(&o[4..], &[c, c]);
        }
        // Vermelho puro: V alto, U baixo.
        let red: Vec<u8> = (0..4).flat_map(|_| [0, 0, 255, 255]).collect();
        let o = bgra_to_nv12(&red, 2, 2);
        assert!(o[4] < 128 && o[5] > 200);
        // Azul puro: U alto, V baixo.
        let blue: Vec<u8> = (0..4).flat_map(|_| [255, 0, 0, 255]).collect();
        let o = bgra_to_nv12(&blue, 2, 2);
        assert!(o[4] > 200 && o[5] < 128);
    }

    #[test]
    fn nv12_tamanho_e_media_de_bloco() {
        let src = ramp(4, 4);
        let o = bgra_to_nv12(&src, 4, 4);
        assert_eq!(o.len(), 4 * 4 * 3 / 2);
        // Dois blocos horizontais distintos geram dois pares UV.
        let mut src = vec![0u8; 4 * 2 * 4];
        for px in src.chunks_exact_mut(4).skip(2).take(2) {
            px.copy_from_slice(&[255, 255, 255, 255]);
        }
        // Linha 2 (y=1) também: blocos 2x2 inteiros brancos no 2º bloco.
        for px in src.chunks_exact_mut(4).skip(6).take(2) {
            px.copy_from_slice(&[255, 255, 255, 255]);
        }
        let o = bgra_to_nv12(&src, 4, 2);
        assert_eq!(o[8..], [128, 128, 128, 128]);
        assert_eq!(o[0], 16);
        assert_eq!(o[2], 235);
    }

    #[test]
    fn encaixe_preenche_e_corta() {
        let src = ramp(2, 2);
        // Maior: copia no canto e preenche com preto opaco.
        let o = fit_bgra(&src, 2, 2, 3, 3);
        assert_eq!(o.len(), 3 * 3 * 4);
        assert_eq!(&o[0..4], &[0, 0, 0, 255]);
        assert_eq!(&o[4..8], &[1, 0, 0, 255]);
        assert_eq!(&o[8..12], &[0, 0, 0, 255]);
        assert_eq!(&o[12..16], &[2, 0, 0, 255]);
        assert_eq!(&o[24..28], &[0, 0, 0, 255]);
        // Menor: corta.
        let o = fit_bgra(&src, 2, 2, 1, 1);
        assert_eq!(o, vec![0, 0, 0, 255]);
    }

    #[test]
    fn empacota_atributos() {
        assert_eq!(pack(1920, 1080), (1920u64 << 32) | 1080);
        assert_eq!(pack(30, 1), 0x1_0000_0000 * 30 + 1);
    }

    #[test]
    fn bitrate_moderado() {
        assert_eq!(video_bitrate(1920, 1080, 30), 4_354_560);
        assert_eq!(video_bitrate(16, 16, 1), 1_000_000);
        assert_eq!(video_bitrate(7680, 4320, 120), 16_000_000);
    }

    #[test]
    fn escolha_de_monitor() {
        let ms = vec![mon(1, 0, 0, true), mon(2, 100, 0, false)];
        let r = |x| Region {
            x,
            y: 0,
            w: 40,
            h: 20,
        };
        assert_eq!(pick_monitor(&r(0), &ms).unwrap().id, 1);
        assert_eq!(pick_monitor(&r(150), &ms).unwrap().id, 2);
        // Centro fora de todos: cai no principal.
        assert_eq!(pick_monitor(&r(900), &ms).unwrap().id, 1);
        // Sem principal: o primeiro.
        let sem = vec![mon(5, 0, 0, false), mon(6, 100, 0, false)];
        assert_eq!(pick_monitor(&r(900), &sem).unwrap().id, 5);
        assert!(pick_monitor(&r(0), &[]).is_none());
        // Fora na vertical.
        let alto = Region {
            x: 0,
            y: 500,
            w: 10,
            h: 10,
        };
        assert_eq!(pick_monitor(&alto, &ms).unwrap().id, 1);
    }

    #[test]
    fn regiao_relativa() {
        let m = mon(2, 100, -20, false);
        let r = Region {
            x: 110,
            y: 0,
            w: 30,
            h: 40,
        };
        assert_eq!(
            rel_region(&r, &m),
            Region {
                x: 10,
                y: 20,
                w: 30,
                h: 40
            }
        );
    }
}
