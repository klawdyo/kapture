//! Linha do tempo da gravação: tempo gravado = relógio menos as pausas.

use crate::domain::RecCfg;

/// Cronologia que "congela" enquanto a gravação está pausada.
///
/// Recebe sempre o instante atual (ms monotônicos) por parâmetro, o que
/// a deixa pura e testável.
#[derive(Debug, Clone, Copy)]
pub struct Timeline {
    t0: u64,
    /// Instante em que a pausa atual começou (`None` = rodando).
    paused_at: Option<u64>,
    /// Soma das pausas já terminadas.
    paused_total: u64,
}

impl Timeline {
    /// Começa a contar em `now`.
    pub fn new(now: u64) -> Self {
        Timeline {
            t0: now,
            paused_at: None,
            paused_total: 0,
        }
    }

    /// Pausa (ignorado se já pausado).
    pub fn pause(&mut self, now: u64) {
        self.paused_at.get_or_insert(now);
    }

    /// Retoma (ignorado se não pausado), descontando a pausa.
    pub fn resume(&mut self, now: u64) {
        if let Some(p) = self.paused_at.take() {
            self.paused_total += now.saturating_sub(p);
        }
    }

    /// Está pausada?
    pub fn is_paused(&self) -> bool {
        self.paused_at.is_some()
    }

    /// Milissegundos gravados até `now` (congelado durante a pausa).
    pub fn rec_ms(&self, now: u64) -> u64 {
        let end = self.paused_at.unwrap_or(now);
        end.saturating_sub(self.t0)
            .saturating_sub(self.paused_total)
    }
}

/// A configuração pede áudio (microfone e/ou som do sistema)?
pub fn has_audio(cfg: &RecCfg) -> bool {
    cfg.sys || cfg.mic_id.as_deref().is_some_and(|m| !m.is_empty())
}

/// Junta diretório e arquivo com `\` (sem duplicar separadores finais).
pub fn join_win(dir: &str, name: &str) -> String {
    format!("{}\\{}", dir.trim_end_matches(['\\', '/']), name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Mode;

    fn cfg(mic: Option<&str>, sys: bool) -> RecCfg {
        RecCfg {
            mode: Mode::Screen,
            target_id: None,
            region: None,
            mic_id: mic.map(String::from),
            sys,
            fps: 30,
            dir: None,
        }
    }

    #[test]
    fn tempo_congela_na_pausa() {
        let mut t = Timeline::new(100);
        assert_eq!(t.rec_ms(100), 0);
        assert_eq!(t.rec_ms(1100), 1000);
        t.pause(1100);
        assert!(t.is_paused());
        t.pause(1500); // segunda pausa é ignorada
        assert_eq!(t.rec_ms(5000), 1000);
        t.resume(2100);
        assert!(!t.is_paused());
        t.resume(9999); // retomar sem pausa é ignorado
        assert_eq!(t.rec_ms(2600), 1500);
        // Relógio retrocedendo não estoura.
        assert_eq!(Timeline::new(50).rec_ms(10), 0);
    }

    #[test]
    fn detecta_audio() {
        assert!(!has_audio(&cfg(None, false)));
        assert!(!has_audio(&cfg(Some(""), false)));
        assert!(has_audio(&cfg(Some("m"), false)));
        assert!(has_audio(&cfg(None, true)));
    }

    #[test]
    fn junta_caminho() {
        assert_eq!(join_win("C:\\V", "a.mp4"), "C:\\V\\a.mp4");
        assert_eq!(join_win("C:\\V\\", "a.mp4"), "C:\\V\\a.mp4");
        assert_eq!(join_win("C:/V/", "a.mp4"), "C:/V\\a.mp4");
    }
}
