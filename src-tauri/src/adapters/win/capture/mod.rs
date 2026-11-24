//! Adaptador `Capture`: vídeo por WGC e áudio por WASAPI, cada um em
//! sua thread, alimentando o `Encoder` com carimbos de tempo coerentes
//! (relógio da gravação sem as pausas).

mod audio;
mod video;

use super::targets::monitors;
use super::worker::Worker;
use crate::lk;
use crate::logging;
use audio::Reader;
use cap_core::domain::{Frame, RecCfg};
use cap_core::media::{
    audio_ts_ms, frames_due, has_audio, plan, Mixer, Timeline, SRC_MIC, SRC_SYS,
};
use cap_core::ports::{Capture, Encoder, MicSink};
use cap_core::Res;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use video::Grab;

/// Linha do tempo compartilhada entre as threads de captura.
#[derive(Clone)]
struct Clk {
    t0: Instant,
    tl: Arc<Mutex<Timeline>>,
}

impl Clk {
    fn new() -> Self {
        Clk {
            t0: Instant::now(),
            tl: Arc::new(Mutex::new(Timeline::new(0))),
        }
    }
    fn now(&self) -> u64 {
        self.t0.elapsed().as_millis() as u64
    }
    /// Tempo gravado (ms) e se está pausado.
    fn rec(&self) -> (u64, bool) {
        let t = lk(&self.tl);
        (t.rec_ms(self.now()), t.is_paused())
    }
}

/// Sessão de gravação em andamento.
struct Sess {
    clk: Clk,
    vid: Worker,
    aud: Option<Worker>,
}

/// Estado do laço de áudio.
struct Aud {
    mic: Option<Reader>,
    sys: Option<Reader>,
    mix: Mixer,
    emitted: u64,
    buf: Vec<i16>,
}

/// Captura real de tela e áudio.
pub struct WinCapture {
    sess: Mutex<Option<Sess>>,
    meter: Mutex<Option<Worker>>,
}

impl WinCapture {
    pub fn new() -> Self {
        Self {
            sess: Mutex::new(None),
            meter: Mutex::new(None),
        }
    }
}

/// Inicia a thread de vídeo: um quadro por período, repetindo o último
/// quando a tela não muda (FPS constante).
fn start_video(cfg: &RecCfg, clk: Clk, sink: Arc<dyn Encoder>) -> Res<Worker> {
    let mons = monitors();
    let pl = plan(cfg, &mons).inspect_err(|e| {
        logging::get().error(&format!(
            "captura: plano inválido: {e} (monitores: {mons:?})"
        ));
    })?;
    logging::get().info(&format!("captura: plano {pl:?}, {} fps", cfg.fps));
    let period = Duration::from_micros(1_000_000 / cfg.fps.max(1) as u64);
    Worker::start(
        period,
        move || Grab::new(pl.src, pl.crop),
        move |g: &mut Grab| {
            let (ts, paused) = clk.rec();
            if paused {
                return;
            }
            let img = lk(&g.latest).clone();
            if let Some(i) = img {
                // Erros de escrita são reportados no `finish` do encoder.
                let _ = sink.write_video(&Frame {
                    w: i.w,
                    h: i.h,
                    ts_ms: ts,
                    data: i.data,
                });
            }
        },
    )
}

/// Inicia a thread de áudio: lê microfone e/ou loopback a cada 10 ms,
/// mistura e entrega ao encoder o que o relógio da gravação pede.
fn start_audio(cfg: &RecCfg, clk: Clk, sink: Arc<dyn Encoder>) -> Res<Worker> {
    let (mic_id, sys) = (cfg.mic_id.clone().filter(|m| !m.is_empty()), cfg.sys);
    Worker::start(
        Duration::from_millis(10),
        move || {
            Ok(Aud {
                mic: mic_id.as_deref().map(Reader::mic).transpose()?,
                sys: if sys { Some(Reader::system()?) } else { None },
                mix: Mixer::new(),
                emitted: 0,
                buf: Vec::new(),
            })
        },
        move |a: &mut Aud| {
            // Sempre drena as fontes (para não estourar o buffer do WASAPI).
            for (src, r) in [(SRC_MIC, &a.mic), (SRC_SYS, &a.sys)] {
                if let Some(r) = r {
                    a.buf.clear();
                    r.read(&mut a.buf);
                    a.mix.push(src, &a.buf);
                }
            }
            let (ms, paused) = clk.rec();
            if paused {
                a.mix.clear();
                return;
            }
            let due = frames_due(ms, a.emitted);
            if due > 0 {
                let pcm = a.mix.pull(due as usize);
                let _ = sink.write_audio(&pcm, audio_ts_ms(a.emitted));
                a.emitted += due;
            }
        },
    )
}

impl Capture for WinCapture {
    fn start(&self, cfg: &RecCfg, sink: Arc<dyn Encoder>) -> Res<()> {
        let mut g = lk(&self.sess);
        if g.is_some() {
            return Err("Já existe uma captura em andamento.".into());
        }
        let clk = Clk::new();
        let vid = start_video(cfg, clk.clone(), sink.clone())?;
        let aud = if has_audio(cfg) {
            // Se o áudio falhar, `vid` é solto aqui e a captura de tela para.
            Some(start_audio(cfg, clk.clone(), sink)?)
        } else {
            None
        };
        *g = Some(Sess { clk, vid, aud });
        Ok(())
    }

    fn pause(&self) -> Res<()> {
        let g = lk(&self.sess);
        let s = g.as_ref().ok_or("Não há captura em andamento.")?;
        lk(&s.clk.tl).pause(s.clk.now());
        Ok(())
    }

    fn resume(&self) -> Res<()> {
        let g = lk(&self.sess);
        let s = g.as_ref().ok_or("Não há captura em andamento.")?;
        lk(&s.clk.tl).resume(s.clk.now());
        Ok(())
    }

    fn stop(&self) -> Res<()> {
        // Para o vídeo e o áudio e espera as threads (sem mais escritas).
        if let Some(mut s) = lk(&self.sess).take() {
            s.vid.stop();
            if let Some(a) = s.aud.as_mut() {
                a.stop();
            }
        }
        Ok(())
    }

    fn mic_start(&self, mic_id: &str, sink: MicSink) -> Res<()> {
        let id = mic_id.to_string();
        let w = Worker::start(
            Duration::from_millis(30),
            move || Reader::mic(&id).map(|r| (r, Vec::<i16>::new())),
            move |(r, buf): &mut (Reader, Vec<i16>)| {
                buf.clear();
                r.read(buf);
                if !buf.is_empty() {
                    sink(buf);
                }
            },
        )?;
        // Substitui (e para) um medidor anterior, se houver.
        *lk(&self.meter) = Some(w);
        Ok(())
    }

    fn mic_stop(&self) -> Res<()> {
        lk(&self.meter).take();
        Ok(())
    }
}
