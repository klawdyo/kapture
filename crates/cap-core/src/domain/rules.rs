//! Regras puras do domínio: validação, nomes, áudio, webcam e cronômetro.

use super::model::{Mode, RecCfg, Region, St, Stamp};
use crate::{CapErr, Res};

/// Lado mínimo (px) de uma região de captura.
pub const MIN_REGION: u32 = 16;
/// FPS máximo aceito.
pub const MAX_FPS: u32 = 120;
/// Lado mínimo (px) da janela da webcam.
pub const CAM_MIN: u32 = 96;
/// Lado máximo (px) da janela da webcam.
pub const CAM_MAX: u32 = 640;

/// A região atende ao tamanho mínimo?
pub fn region_ok(r: &Region) -> bool {
    r.w >= MIN_REGION && r.h >= MIN_REGION
}

/// A barra deve ficar FORA da captura (tela e Print Screen)? Só enquanto grava
/// ou pausado: ocioso, ela aparece normalmente nas capturas do usuário.
pub fn protected(st: St) -> bool {
    st != St::Idle
}

/// Prefixo das únicas URLs que o app abre no navegador (páginas de release).
pub const RELEASE_URL: &str = "https://github.com/klawdyo/kapture/releases/";

/// A URL é uma página de release do kapture, sem caracteres de controle,
/// espaços, aspas, barras invertidas nem `..`?
pub fn release_url_ok(url: &str) -> bool {
    url.starts_with(RELEASE_URL)
        && !url.contains("..")
        && !url
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '"' || c == '\\')
}

/// Valida a configuração de gravação.
pub fn validate(cfg: &RecCfg) -> Res<()> {
    if !(1..=MAX_FPS).contains(&cfg.fps) {
        return Err(CapErr::from(format!(
            "FPS inválido: use um valor entre 1 e {MAX_FPS}."
        )));
    }
    match cfg.mode {
        Mode::Screen => Ok(()),
        Mode::Window => match cfg.target_id {
            Some(_) => Ok(()),
            None => Err("Selecione uma janela para gravar.".into()),
        },
        Mode::Area => match &cfg.region {
            None => Err("Selecione uma área da tela para gravar.".into()),
            Some(r) if !region_ok(r) => Err(CapErr::from(format!(
                "A área selecionada é muito pequena (mínimo {MIN_REGION}x{MIN_REGION} pixels)."
            ))),
            Some(_) => Ok(()),
        },
    }
}

/// Nome do arquivo de vídeo: `kapture-AAAAMMDD-HHMMSS.mp4`.
pub fn file_name(t: &Stamp) -> String {
    format!(
        "kapture-{:04}{:02}{:02}-{:02}{:02}{:02}.mp4",
        t.y, t.mo, t.d, t.h, t.mi, t.s
    )
}

/// Normaliza soma de quadrados e contagem em RMS limitado a 0..=1.
fn rms(sum_sq: f64, n: usize) -> f32 {
    if n == 0 {
        return 0.0;
    }
    let v = (sum_sq / n as f64).sqrt();
    if v.is_nan() {
        0.0
    } else {
        v.min(1.0) as f32
    }
}

/// Nível RMS (0..=1) de amostras PCM de 16 bits, para o VU meter.
pub fn level_i16(s: &[i16]) -> f32 {
    let sum: f64 = s.iter().map(|&v| (v as f64 / 32768.0).powi(2)).sum();
    rms(sum, s.len())
}

/// Nível RMS (0..=1) de amostras de ponto flutuante (-1..=1).
pub fn level_f32(s: &[f32]) -> f32 {
    let sum: f64 = s.iter().map(|&v| (v as f64).powi(2)).sum();
    rms(sum, s.len())
}

/// Lado do quadrado da webcam (proporção 1:1) após um redimensionamento.
///
/// Usa a dimensão que o usuário mudou em relação ao lado anterior
/// (largura se mudou, senão altura) e limita entre `CAM_MIN` e `CAM_MAX`.
pub fn cam_side(w: u32, h: u32, prev: u32) -> u32 {
    let wanted = if w != prev { w } else { h };
    wanted.clamp(CAM_MIN, CAM_MAX)
}

/// Formata o cronômetro: `MM:SS` ou `H:MM:SS` a partir de 1 hora.
pub fn fmt_timer(secs: u64) -> String {
    let (h, m, s) = (secs / 3600, secs % 3600 / 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn so_aceita_url_de_release() {
        assert!(release_url_ok(
            "https://github.com/klawdyo/kapture/releases/tag/v1.2.3"
        ));
        for ruim in [
            "",
            "http://github.com/klawdyo/kapture/releases/tag/v1",
            "https://github.com/klawdyo/kapture/issues/1",
            "https://github.com/klawdyo/kapture/releases",
            "https://github.com/klawdyo/kapture/releases/../../x",
            "https://github.com/klawdyo/kapture/releases/a b",
            "https://github.com/klawdyo/kapture/releases/a\"b",
            "https://github.com/klawdyo/kapture/releases/a\\b",
            "https://github.com/klawdyo/kapture/releases/a\nb",
            "https://github.com.evil.com/klawdyo/kapture/releases/",
        ] {
            assert!(!release_url_ok(ruim), "{ruim:?}");
        }
    }

    fn cfg(mode: Mode) -> RecCfg {
        RecCfg {
            mode,
            target_id: None,
            region: None,
            mic_id: None,
            sys: false,
            fps: 30,
            dir: None,
        }
    }

    #[test]
    fn regiao_minima() {
        assert!(region_ok(&Region {
            x: 0,
            y: 0,
            w: 16,
            h: 16
        }));
        assert!(!region_ok(&Region {
            x: 0,
            y: 0,
            w: 15,
            h: 16
        }));
        assert!(!region_ok(&Region {
            x: 0,
            y: 0,
            w: 16,
            h: 15
        }));
    }

    #[test]
    fn fps_limites() {
        for fps in [0, 121] {
            let mut c = cfg(Mode::Screen);
            c.fps = fps;
            assert!(validate(&c).unwrap_err().0.contains("FPS inválido"));
        }
        for fps in [1, 120] {
            let mut c = cfg(Mode::Screen);
            c.fps = fps;
            assert!(validate(&c).is_ok());
        }
    }

    #[test]
    fn modo_janela_exige_alvo() {
        let mut c = cfg(Mode::Window);
        assert!(validate(&c).unwrap_err().0.contains("janela"));
        c.target_id = Some(1);
        assert!(validate(&c).is_ok());
    }

    #[test]
    fn modo_area_exige_regiao() {
        let mut c = cfg(Mode::Area);
        assert!(validate(&c).unwrap_err().0.contains("área"));
        c.region = Some(Region {
            x: 0,
            y: 0,
            w: 4,
            h: 4,
        });
        assert!(validate(&c).unwrap_err().0.contains("muito pequena"));
        c.region = Some(Region {
            x: 0,
            y: 0,
            w: 100,
            h: 100,
        });
        assert!(validate(&c).is_ok());
    }

    #[test]
    fn nome_do_arquivo() {
        let t = Stamp {
            y: 2026,
            mo: 3,
            d: 9,
            h: 7,
            mi: 5,
            s: 1,
        };
        assert_eq!(file_name(&t), "kapture-20260309-070501.mp4");
    }

    #[test]
    fn nivel_i16() {
        assert_eq!(level_i16(&[]), 0.0);
        assert_eq!(level_i16(&[0, 0]), 0.0);
        assert!((level_i16(&[i16::MIN, i16::MIN]) - 1.0).abs() < 1e-6);
        assert!((level_i16(&[16384, -16384]) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn nivel_f32() {
        assert_eq!(level_f32(&[]), 0.0);
        assert!((level_f32(&[0.5, -0.5]) - 0.5).abs() < 1e-6);
        assert_eq!(level_f32(&[3.0, 3.0]), 1.0);
        assert_eq!(level_f32(&[f32::NAN]), 0.0);
    }

    #[test]
    fn barra_fora_da_captura_so_gravando() {
        assert!(!protected(St::Idle));
        assert!(protected(St::Rec));
        assert!(protected(St::Paused));
    }

    #[test]
    fn webcam_quadrada() {
        assert_eq!(cam_side(200, 150, 150), 200);
        assert_eq!(cam_side(150, 220, 150), 220);
        assert_eq!(cam_side(10, 10, 150), CAM_MIN);
        assert_eq!(cam_side(9999, 1, 150), CAM_MAX);
    }

    #[test]
    fn cronometro() {
        assert_eq!(fmt_timer(0), "00:00");
        assert_eq!(fmt_timer(75), "01:15");
        assert_eq!(fmt_timer(3600), "1:00:00");
        assert_eq!(fmt_timer(3725), "1:02:05");
    }
}
