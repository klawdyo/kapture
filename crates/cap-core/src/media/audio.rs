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

/// Formato de amostra de um dispositivo (o formato nativo do mix do WASAPI).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// PCM inteiro de 16 bits.
    I16,
    /// PCM inteiro de 24 bits (3 bytes por amostra).
    I24,
    /// PCM inteiro de 32 bits.
    I32,
    /// Ponto flutuante de 32 bits (o mais comum no mix do Windows).
    F32,
}

impl Kind {
    /// Bytes por amostra.
    pub fn bytes(self) -> usize {
        match self {
            Kind::I16 => 2,
            Kind::I24 => 3,
            Kind::I32 | Kind::F32 => 4,
        }
    }
}

/// Formato de origem: tipo de amostra, canais e taxa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SrcFmt {
    pub kind: Kind,
    pub ch: usize,
    pub rate: u32,
}

/// Etiqueta WAVE de PCM inteiro.
pub const TAG_PCM: u16 = 1;
/// Etiqueta WAVE de ponto flutuante.
pub const TAG_FLOAT: u16 = 3;
/// Etiqueta WAVE extensível (o tipo real vem no subformato).
pub const TAG_EXT: u16 = 0xFFFE;

/// Interpreta os campos de um `WAVEFORMATEX`/`WAVEFORMATEXTENSIBLE`.
/// `sub` é o início do GUID de subformato (só usado com `TAG_EXT`).
/// Devolve `None` para formatos que o app não sabe converter.
pub fn parse_fmt(tag: u16, bits: u16, ch: u16, rate: u32, sub: u16) -> Option<SrcFmt> {
    if ch == 0 || rate == 0 {
        return None;
    }
    let real = if tag == TAG_EXT { sub } else { tag };
    let kind = match (real, bits) {
        (TAG_PCM, 16) => Kind::I16,
        (TAG_PCM, 24) => Kind::I24,
        (TAG_PCM, 32) => Kind::I32,
        (TAG_FLOAT, 32) => Kind::F32,
        _ => return None,
    };
    Some(SrcFmt {
        kind,
        ch: ch as usize,
        rate,
    })
}

/// Descrição curta do formato, para o log (`F32 2 canais 48000 Hz`).
pub fn fmt_desc(f: &SrcFmt) -> String {
    format!("{:?} {} canais {} Hz", f.kind, f.ch, f.rate)
}

/// Decodifica uma amostra (little-endian) para `f32` em -1.0..=1.0.
fn sample(kind: Kind, b: &[u8]) -> f32 {
    match kind {
        Kind::I16 => i16::from_le_bytes([b[0], b[1]]) as f32 / 32_768.0,
        Kind::I24 => (i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8) as f32 / 8_388_608.0,
        Kind::I32 => i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f32 / 2_147_483_648.0,
        Kind::F32 => f32::from_le_bytes([b[0], b[1], b[2], b[3]]),
    }
}

/// Converte um quadro (todos os canais) em estéreo. Mono vira dois lados
/// iguais; com mais de 2 canais, os pares (0, 2, ...) vão à esquerda e os
/// ímpares à direita, pela média.
fn to_stereo(kind: Kind, frame: &[u8]) -> [f32; 2] {
    let w = kind.bytes();
    let n = frame.len() / w;
    let (mut sum, mut cnt) = ([0f32; 2], [0f32; 2]);
    for (i, b) in frame.chunks_exact(w).enumerate() {
        sum[i % 2] += sample(kind, b);
        cnt[i % 2] += 1.0;
    }
    if n == 1 {
        return [sum[0], sum[0]];
    }
    [sum[0] / cnt[0], sum[1] / cnt[1]]
}

/// Amostra `f32` para `i16` com saturação.
pub fn to_i16(v: f32) -> i16 {
    (v.clamp(-1.0, 1.0) * 32_767.0).round() as i16
}

/// Conversor contínuo de qualquer formato de origem para i16 estéreo a
/// 48 kHz: mixagem de canais e reamostragem linear. Guarda o último quadro
/// e a fase fracionária entre pacotes, então não há emenda audível.
pub struct Conv {
    fmt: SrcFmt,
    /// Último quadro do pacote anterior (ponto de partida da interpolação).
    prev: Option<[f32; 2]>,
    /// Posição de leitura (em quadros de origem) relativa a `prev`.
    pos: f64,
}

impl Conv {
    pub fn new(fmt: SrcFmt) -> Self {
        Conv {
            fmt,
            prev: None,
            pos: 0.0,
        }
    }

    /// Bytes de um quadro de origem (todos os canais).
    pub fn block(&self) -> usize {
        self.fmt.kind.bytes() * self.fmt.ch
    }

    /// Converte `data` (quadros inteiros; sobra parcial é ignorada) e
    /// acrescenta as amostras i16 estéreo 48 kHz a `out`.
    pub fn push(&mut self, data: &[u8], out: &mut Vec<i16>) {
        let mut buf: Vec<[f32; 2]> = self.prev.into_iter().collect();
        buf.extend(
            data.chunks_exact(self.block())
                .map(|f| to_stereo(self.fmt.kind, f)),
        );
        let step = self.fmt.rate as f64 / RATE as f64;
        while self.pos + 1.0 < buf.len() as f64 {
            let i = self.pos as usize;
            let t = (self.pos - i as f64) as f32;
            for c in 0..CH {
                out.push(to_i16(buf[i][c] * (1.0 - t) + buf[i + 1][c] * t));
            }
            self.pos += step;
        }
        // Só guarda o último quadro se houve algum (buf vazio: nada a fazer).
        if let Some(last) = buf.last() {
            self.pos -= (buf.len() - 1) as f64;
            self.prev = Some(*last);
        }
    }

    /// Como `push`, com `frames` quadros de silêncio (pacote marcado como mudo).
    pub fn push_silence(&mut self, frames: usize, out: &mut Vec<i16>) {
        self.push(&vec![0u8; frames * self.block()], out);
    }
}

/// Aviso ao usuário quando uma fonte de áudio não abriu.
pub fn audio_warn(src: usize) -> String {
    let what = if src == SRC_MIC {
        "o microfone"
    } else {
        "o som do sistema"
    };
    format!("Não foi possível usar {what}; a gravação segue sem ele.")
}

/// Linha de log de uma falha ao abrir o áudio: etapa, dispositivo, formato e HRESULT.
pub fn audio_fail_line(step: &str, dev: &str, fmt: &str, code: i32, msg: &str) -> String {
    format!(
        "áudio: {step} falhou (dispositivo '{dev}', formato {fmt}): HRESULT {} ({})",
        crate::log::hresult(code),
        msg.trim()
    )
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

    fn f32b(v: &[f32]) -> Vec<u8> {
        v.iter().flat_map(|x| x.to_le_bytes()).collect()
    }

    fn fmt(kind: Kind, ch: usize, rate: u32) -> SrcFmt {
        SrcFmt { kind, ch, rate }
    }

    #[test]
    fn interpreta_formatos() {
        assert_eq!(
            parse_fmt(1, 16, 2, 48_000, 0),
            Some(fmt(Kind::I16, 2, 48_000))
        );
        assert_eq!(
            parse_fmt(1, 24, 1, 44_100, 0),
            Some(fmt(Kind::I24, 1, 44_100))
        );
        assert_eq!(
            parse_fmt(1, 32, 2, 96_000, 0),
            Some(fmt(Kind::I32, 2, 96_000))
        );
        assert_eq!(
            parse_fmt(3, 32, 2, 48_000, 0),
            Some(fmt(Kind::F32, 2, 48_000))
        );
        // Extensível: vale o subformato.
        assert_eq!(
            parse_fmt(TAG_EXT, 32, 6, 48_000, 3),
            Some(fmt(Kind::F32, 6, 48_000))
        );
        assert_eq!(
            parse_fmt(TAG_EXT, 16, 2, 48_000, 1),
            Some(fmt(Kind::I16, 2, 48_000))
        );
        assert_eq!(parse_fmt(TAG_EXT, 16, 2, 48_000, 99), None);
        // Inválidos.
        assert_eq!(parse_fmt(1, 8, 2, 48_000, 0), None);
        assert_eq!(parse_fmt(2, 16, 2, 48_000, 0), None);
        assert_eq!(parse_fmt(1, 16, 0, 48_000, 0), None);
        assert_eq!(parse_fmt(1, 16, 2, 0, 0), None);
        assert_eq!(
            fmt_desc(&fmt(Kind::F32, 2, 48_000)),
            "F32 2 canais 48000 Hz"
        );
    }

    #[test]
    fn converte_cada_tipo_de_amostra() {
        let mut out = Vec::new();
        // I16 estéreo 48k: passa direto.
        let mut c = Conv::new(fmt(Kind::I16, 2, 48_000));
        assert_eq!(c.block(), 4);
        let d: Vec<u8> = [100i16, -100, 7, 8]
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect();
        c.push(&d, &mut out);
        // Atraso de um quadro: o último sai no próximo pacote.
        assert_eq!(out, vec![100, -100]);
        c.push(&[], &mut out);
        assert_eq!(out.len(), 2);
        // F32 mono vira estéreo igual; +-1.0 e além saturam.
        let mut c = Conv::new(fmt(Kind::F32, 1, 48_000));
        let mut out = Vec::new();
        c.push(&f32b(&[1.0, -1.0, 2.0, 0.0]), &mut out);
        assert_eq!(out, vec![32_767, 32_767, -32_767, -32_767, 32_767, 32_767]);
        // I32 e I24 no máximo e no mínimo.
        let mut c = Conv::new(fmt(Kind::I32, 1, 48_000));
        let mut out = Vec::new();
        let d: Vec<u8> = [i32::MAX, i32::MIN, 0]
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect();
        c.push(&d, &mut out);
        assert_eq!(out, vec![32_767, 32_767, -32_767, -32_767]);
        let mut c = Conv::new(fmt(Kind::I24, 1, 48_000));
        let mut out = Vec::new();
        // 0x400000 = metade da escala; 0xC00000 (negativo) = -0.5.
        c.push(&[0x00, 0x00, 0x40, 0x00, 0x00, 0xC0, 0, 0, 0], &mut out);
        assert_eq!(out, vec![16_384, 16_384, -16_384, -16_384]);
    }

    #[test]
    fn mixa_canais() {
        let mut out = Vec::new();
        // 4 canais: esquerda = média de 0 e 2; direita = média de 1 e 3.
        let mut c = Conv::new(fmt(Kind::F32, 4, 48_000));
        c.push(&f32b(&[0.2, 0.4, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0]), &mut out);
        assert_eq!(out, vec![to_i16(0.4), to_i16(0.2)]);
    }

    #[test]
    fn reamostra_sem_emenda() {
        // 24 kHz -> 48 kHz: dobra, interpolando no meio.
        let mut c = Conv::new(fmt(Kind::F32, 1, 24_000));
        let mut out = Vec::new();
        c.push(&f32b(&[0.0, 1.0]), &mut out);
        assert_eq!(out, vec![0, 0, to_i16(0.5), to_i16(0.5)]);
        // O próximo pacote continua de onde parou (1.0 -> 0.0).
        c.push(&f32b(&[0.0]), &mut out);
        assert_eq!(out.len(), 8);
        assert_eq!(&out[4..], &[32_767, 32_767, to_i16(0.5), to_i16(0.5)]);
        // 96 kHz -> 48 kHz: metade dos quadros.
        let mut c = Conv::new(fmt(Kind::F32, 1, 96_000));
        let mut out = Vec::new();
        c.push(&f32b(&[0.0, 0.1, 0.2, 0.3, 0.4]), &mut out);
        assert_eq!(out.len(), 4);
        // Total conserva a duração: 44,1 kHz em pacotes de 441 dá ~480 quadros por pacote.
        let mut c = Conv::new(fmt(Kind::I16, 2, 44_100));
        let mut out = Vec::new();
        let pkt = vec![0u8; 441 * 4];
        for _ in 0..100 {
            c.push(&pkt, &mut out);
        }
        let frames = out.len() / 2;
        assert!((47_990..=48_000).contains(&frames), "{frames}");
    }

    #[test]
    fn silencio_e_sobra_parcial() {
        let mut c = Conv::new(fmt(Kind::F32, 2, 48_000));
        let mut out = Vec::new();
        c.push_silence(4, &mut out);
        assert_eq!(out, vec![0; 6]);
        // Bytes soltos que não fecham um quadro são ignorados.
        let mut c = Conv::new(fmt(Kind::I16, 2, 48_000));
        let mut out = Vec::new();
        c.push(&[1, 2, 3], &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn saturacao_e_textos() {
        assert_eq!(to_i16(5.0), 32_767);
        assert_eq!(to_i16(-5.0), -32_767);
        assert_eq!(to_i16(0.0), 0);
        assert_eq!(
            audio_warn(SRC_MIC),
            "Não foi possível usar o microfone; a gravação segue sem ele."
        );
        assert!(audio_warn(SRC_SYS).contains("som do sistema"));
        assert_eq!(
            audio_fail_line("Initialize", "Mic", "F32 2 canais 48000 Hz", -2147024809, " parâmetro incorreto "),
            "áudio: Initialize falhou (dispositivo 'Mic', formato F32 2 canais 48000 Hz): HRESULT 0x80070057 (parâmetro incorreto)"
        );
    }
}
