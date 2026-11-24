//! Decorador de `Encoder` que registra no log: abertura, primeiro quadro
//! (resolução, fps e bitrate), contagem periódica de quadros/áudio
//! escritos e falhas de escrita (sem repetir a mesma falha a cada quadro).

use crate::domain::{Frame, RecCfg};
use crate::lk;
use crate::media::{even, has_audio, video_bitrate, CH, RATE};
use crate::ports::{Clock, Encoder, Log};
use crate::Res;
use std::sync::{Arc, Mutex};

/// Intervalo entre as linhas de progresso (ms).
const EVERY_MS: u64 = 5000;
/// Falhas repetidas: registra a primeira e depois uma a cada tantas.
const ERR_EVERY: u64 = 300;

/// Contadores da gravação atual.
#[derive(Default)]
struct Stat {
    fps: u32,
    first: bool,
    video: u64,
    samples: u64,
    err_v: u64,
    err_a: u64,
    last: u64,
}

/// Encoder que delega a outro e anota o que acontece.
pub struct LoggedEnc {
    inner: Arc<dyn Encoder>,
    log: Arc<dyn Log>,
    clock: Arc<dyn Clock>,
    st: Mutex<Stat>,
}

impl LoggedEnc {
    pub fn new(inner: Arc<dyn Encoder>, log: Arc<dyn Log>, clock: Arc<dyn Clock>) -> Self {
        LoggedEnc {
            inner,
            log,
            clock,
            st: Mutex::new(Stat::default()),
        }
    }

    /// Linha de progresso a cada `EVERY_MS`.
    fn tick(&self, s: &mut Stat) {
        let now = self.clock.mono_ms();
        if now.saturating_sub(s.last) >= EVERY_MS {
            s.last = now;
            self.log.info(&format!(
                "gravando: {} quadros de vídeo, {} amostras de áudio ({} s)",
                s.video,
                s.samples,
                s.samples / (RATE as u64 * CH as u64)
            ));
        }
    }
}

/// Deve registrar a falha de número `n` (1ª e depois a cada `ERR_EVERY`)?
fn report(n: u64) -> bool {
    n == 1 || n.is_multiple_of(ERR_EVERY)
}

impl Encoder for LoggedEnc {
    fn open(&self, path: &str, cfg: &RecCfg) -> Res<()> {
        *lk(&self.st) = Stat {
            fps: cfg.fps,
            last: self.clock.mono_ms(),
            ..Stat::default()
        };
        self.log.info(&format!(
            "encoder: abrir {path} (fps={}, áudio={})",
            cfg.fps,
            has_audio(cfg)
        ));
        let r = self.inner.open(path, cfg);
        if let Err(e) = &r {
            self.log.error(&format!("encoder: falha ao abrir: {e}"));
        }
        r
    }

    fn write_video(&self, frame: &Frame) -> Res<()> {
        let r = self.inner.write_video(frame);
        let mut s = lk(&self.st);
        if !s.first {
            s.first = true;
            let (w, h) = (even(frame.w), even(frame.h));
            self.log.info(&format!(
                "encoder: primeiro quadro {}x{} (vídeo {w}x{h}, {} fps, bitrate {} bps)",
                frame.w,
                frame.h,
                s.fps,
                video_bitrate(w, h, s.fps)
            ));
        }
        match &r {
            Ok(()) => s.video += 1,
            Err(e) => {
                s.err_v += 1;
                if report(s.err_v) {
                    self.log.error(&format!(
                        "encoder: falha ao gravar vídeo (ocorrência {}): {e}",
                        s.err_v
                    ));
                }
            }
        }
        self.tick(&mut s);
        r
    }

    fn write_audio(&self, pcm: &[i16], ts_ms: u64) -> Res<()> {
        let r = self.inner.write_audio(pcm, ts_ms);
        let mut s = lk(&self.st);
        match &r {
            Ok(()) => s.samples += pcm.len() as u64,
            Err(e) => {
                s.err_a += 1;
                if report(s.err_a) {
                    self.log.error(&format!(
                        "encoder: falha ao gravar áudio (ocorrência {}): {e}",
                        s.err_a
                    ));
                }
            }
        }
        self.tick(&mut s);
        r
    }

    fn finish(&self) -> Res<()> {
        {
            let s = lk(&self.st);
            self.log.info(&format!(
                "encoder: finalizar ({} quadros de vídeo, {} amostras de áudio, {} falhas de vídeo, {} de áudio)",
                s.video, s.samples, s.err_v, s.err_a
            ));
        }
        let r = self.inner.finish();
        if let Err(e) = &r {
            self.log.error(&format!("encoder: falha ao finalizar: {e}"));
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Mode;
    use crate::fakes::*;

    fn cfg() -> RecCfg {
        RecCfg {
            mode: Mode::Screen,
            target_id: None,
            region: None,
            mic_id: None,
            sys: true,
            fps: 30,
            dir: None,
        }
    }

    fn frame() -> Frame {
        Frame {
            w: 101,
            h: 51,
            ts_ms: 0,
            data: vec![0; 101 * 51 * 4],
        }
    }

    struct Kit {
        inner: Arc<FakeEncoder>,
        log: Arc<FakeLog>,
        clock: Arc<FakeClock>,
        enc: LoggedEnc,
    }

    fn kit() -> Kit {
        let inner = Arc::new(FakeEncoder::default());
        let log = Arc::new(FakeLog::default());
        let clock = Arc::new(FakeClock::default());
        let enc = LoggedEnc::new(inner.clone(), log.clone(), clock.clone());
        Kit {
            inner,
            log,
            clock,
            enc,
        }
    }

    #[test]
    fn registra_abertura_primeiro_quadro_e_progresso() {
        let k = kit();
        k.enc.open("C:\\v\\a.mp4", &cfg()).unwrap();
        assert_eq!(
            k.log.msgs()[0],
            "encoder: abrir C:\\v\\a.mp4 (fps=30, áudio=true)"
        );
        k.enc.write_video(&frame()).unwrap();
        k.enc.write_video(&frame()).unwrap();
        k.enc.write_audio(&[0; 96_000], 0).unwrap();
        let m = k.log.msgs();
        assert_eq!(m.len(), 2); // ainda sem progresso
        assert!(m[1].starts_with("encoder: primeiro quadro 101x51 (vídeo 100x50, 30 fps, bitrate "));
        k.clock.advance(5000);
        k.enc.write_video(&frame()).unwrap();
        assert_eq!(
            k.log.msgs()[2],
            "gravando: 3 quadros de vídeo, 96000 amostras de áudio (1 s)"
        );
        k.clock.advance(1000);
        k.enc.write_audio(&[0; 2], 0).unwrap(); // dentro do intervalo
        assert_eq!(k.log.msgs().len(), 3);
        k.clock.advance(4000);
        k.enc.write_audio(&[0; 2], 0).unwrap();
        assert!(k.log.msgs()[3].contains("96004 amostras"));
        k.enc.finish().unwrap();
        assert!(k.log.msgs()[4].starts_with("encoder: finalizar (3 quadros"));
        assert_eq!(k.inner.calls.all().last().unwrap(), "finish");
    }

    #[test]
    fn falhas_nao_se_repetem_a_cada_quadro() {
        let k = kit();
        k.enc.open("p", &cfg()).unwrap();
        k.inner.fail.set("write_video", "v");
        k.inner.fail.set("write_audio", "a");
        for _ in 0..301 {
            assert!(k.enc.write_video(&frame()).is_err());
        }
        assert!(k.enc.write_audio(&[0; 2], 0).is_err());
        let erros: Vec<String> = k
            .log
            .msgs()
            .into_iter()
            .filter(|m| m.contains("falha ao gravar"))
            .collect();
        assert_eq!(erros.len(), 3);
        assert!(erros[0].contains("vídeo (ocorrência 1): v"));
        assert!(erros[1].contains("vídeo (ocorrência 300): v"));
        assert!(erros[2].contains("áudio (ocorrência 1): a"));
    }

    #[test]
    fn falhas_de_abrir_e_finalizar() {
        let k = kit();
        k.inner.fail.set("open", "o");
        assert!(k.enc.open("p", &cfg()).is_err());
        assert!(k.log.has("falha ao abrir: o"));
        k.inner.fail.set("finish", "f");
        assert!(k.enc.finish().is_err());
        assert!(k.log.has("falha ao finalizar: f"));
    }
}
