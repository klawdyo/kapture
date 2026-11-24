//! Plano de captura: o que gravar (monitor ou janela) e qual recorte aplicar.

use super::{pick_monitor, rel_region};
use crate::domain::{Mode, Monitor, RecCfg, Region};
use crate::Res;

/// Origem do vídeo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Monitor (id do monitor).
    Monitor(u32),
    /// Janela (id da janela).
    Window(u32),
}

/// Origem e recorte (em coordenadas locais da origem).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plan {
    pub src: Source,
    pub crop: Option<Region>,
}

/// Traduz a configuração em um plano de captura.
///
/// - `screen`: monitor escolhido (ou o principal), quadro inteiro;
/// - `window`: a janela escolhida;
/// - `area`: monitor que contém o centro da região, recortado nela.
pub fn plan(cfg: &RecCfg, mons: &[Monitor]) -> Res<Plan> {
    match cfg.mode {
        Mode::Screen => {
            let m = match cfg.target_id {
                Some(id) => mons.iter().find(|m| m.id == id),
                None => mons.iter().find(|m| m.primary).or_else(|| mons.first()),
            }
            .ok_or("O monitor escolhido não foi encontrado.")?;
            Ok(Plan {
                src: Source::Monitor(m.id),
                crop: None,
            })
        }
        Mode::Window => Ok(Plan {
            src: Source::Window(cfg.target_id.ok_or("Selecione uma janela para gravar.")?),
            crop: None,
        }),
        Mode::Area => {
            let r = cfg
                .region
                .ok_or("Selecione uma área da tela para gravar.")?;
            let m = pick_monitor(&r, mons).ok_or("Nenhum monitor foi encontrado.")?;
            Ok(Plan {
                src: Source::Monitor(m.id),
                crop: Some(rel_region(&r, m)),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mon(id: u32, x: i32, primary: bool) -> Monitor {
        Monitor {
            id,
            name: String::new(),
            w: 100,
            h: 100,
            x,
            y: 0,
            primary,
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
    fn tela_inteira() {
        let ms = [mon(1, 0, false), mon(2, 100, true)];
        // Sem alvo: monitor principal.
        let p = plan(&cfg(Mode::Screen), &ms).unwrap();
        assert_eq!(p.src, Source::Monitor(2));
        assert_eq!(p.crop, None);
        // Com alvo.
        let mut c = cfg(Mode::Screen);
        c.target_id = Some(1);
        assert_eq!(plan(&c, &ms).unwrap().src, Source::Monitor(1));
        // Alvo inexistente.
        c.target_id = Some(9);
        assert!(plan(&c, &ms).unwrap_err().0.contains("monitor"));
        // Sem principal: o primeiro; sem monitores: erro.
        let sem = [mon(7, 0, false)];
        assert_eq!(
            plan(&cfg(Mode::Screen), &sem).unwrap().src,
            Source::Monitor(7)
        );
        assert!(plan(&cfg(Mode::Screen), &[]).is_err());
    }

    #[test]
    fn janela() {
        let mut c = cfg(Mode::Window);
        assert!(plan(&c, &[]).unwrap_err().0.contains("janela"));
        c.target_id = Some(42);
        let p = plan(&c, &[]).unwrap();
        assert_eq!(p.src, Source::Window(42));
        assert_eq!(p.crop, None);
    }

    #[test]
    fn area_recorta_no_monitor_certo() {
        let ms = [mon(1, 0, true), mon(2, 100, false)];
        let mut c = cfg(Mode::Area);
        assert!(plan(&c, &ms).unwrap_err().0.contains("área"));
        c.region = Some(Region {
            x: 120,
            y: 10,
            w: 30,
            h: 40,
        });
        let p = plan(&c, &ms).unwrap();
        assert_eq!(p.src, Source::Monitor(2));
        assert_eq!(
            p.crop,
            Some(Region {
                x: 20,
                y: 10,
                w: 30,
                h: 40
            })
        );
        assert!(plan(&c, &[]).unwrap_err().0.contains("monitor"));
    }
}
