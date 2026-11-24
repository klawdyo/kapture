//! Mixagem de áudio pura: junta microfone e áudio do sistema num fluxo só.

use std::collections::VecDeque;

/// Taxa de amostragem do app (Hz).
pub const RATE: u32 = 48_000;
/// Canais (estéreo intercalado).
pub const CH: usize = 2;
/// Limite de cada fila: 2 s de áudio (descarta o mais antigo se passar).
const MAX_Q: usize = RATE as usize * CH * 2;

/// Fontes aceitas pelo mixer.
pub const SRC_MIC: usize = 0;
pub const SRC_SYS: usize = 1;

/// Mixer de duas fontes PCM i16 estéreo, dirigido pelo relógio.
///
/// Cada fonte entrega amostras quando quer (o loopback do sistema fica
/// mudo se nada toca); quem chama `pull` decide quantos quadros saem,
/// e o que faltar vira silêncio. Assim o áudio nunca trava esperando
/// uma fonte.
#[derive(Default)]
pub struct Mixer {
    q: [VecDeque<i16>; 2],
}

impl Mixer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enfileira amostras de uma fonte (`SRC_MIC` ou `SRC_SYS`).
    pub fn push(&mut self, src: usize, pcm: &[i16]) {
        let q = &mut self.q[src];
        q.extend(pcm.iter().copied());
        let extra = q.len().saturating_sub(MAX_Q);
        q.drain(..extra);
    }

    /// Retira `frames` quadros (`frames * 2` amostras) somando as fontes
    /// com saturação; falta de dados vira silêncio.
    pub fn pull(&mut self, frames: usize) -> Vec<i16> {
        let n = frames * CH;
        (0..n)
            .map(|_| {
                let a = self.q[SRC_MIC].pop_front().unwrap_or(0) as i32;
                let b = self.q[SRC_SYS].pop_front().unwrap_or(0) as i32;
                (a + b).clamp(i16::MIN as i32, i16::MAX as i32) as i16
            })
            .collect()
    }

    /// Descarta tudo (ao pausar/retomar, para não vazar áudio antigo).
    pub fn clear(&mut self) {
        self.q.iter_mut().for_each(VecDeque::clear);
    }
}

/// Quadros de áudio devidos até `elapsed_ms`, descontando os já emitidos.
pub fn frames_due(elapsed_ms: u64, emitted: u64) -> u64 {
    (elapsed_ms * RATE as u64 / 1000).saturating_sub(emitted)
}

/// Carimbo de tempo (ms) de um total de quadros de áudio já emitidos.
pub fn audio_ts_ms(emitted: u64) -> u64 {
    emitted * 1000 / RATE as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soma_fontes_com_saturacao() {
        let mut m = Mixer::new();
        m.push(SRC_MIC, &[100, -100, i16::MAX, i16::MIN]);
        m.push(SRC_SYS, &[1, 2, 10, -10]);
        assert_eq!(m.pull(2), vec![101, -98, i16::MAX, i16::MIN]);
    }

    #[test]
    fn falta_vira_silencio() {
        let mut m = Mixer::new();
        m.push(SRC_SYS, &[5, 6]);
        // Pede 2 quadros, só há 1: o resto é zero.
        assert_eq!(m.pull(2), vec![5, 6, 0, 0]);
        assert_eq!(m.pull(1), vec![0, 0]);
        assert_eq!(m.pull(0), Vec::<i16>::new());
    }

    #[test]
    fn fila_limitada_e_limpeza() {
        let mut m = Mixer::new();
        let big = vec![1i16; MAX_Q + 10];
        m.push(SRC_MIC, &big);
        assert_eq!(m.q[SRC_MIC].len(), MAX_Q);
        m.clear();
        assert_eq!(m.pull(1), vec![0, 0]);
    }

    #[test]
    fn relogio_de_quadros() {
        assert_eq!(frames_due(1000, 0), 48_000);
        assert_eq!(frames_due(1000, 47_000), 1_000);
        assert_eq!(frames_due(10, 1_000), 0);
        assert_eq!(audio_ts_ms(48_000), 1000);
        assert_eq!(audio_ts_ms(0), 0);
    }
}
