//! Caso de uso Recorder: máquina de estados da gravação.

use crate::domain::{file_name, validate, RecCfg, RecState, St};
use crate::lk;
use crate::log::cfg_line;
use crate::ports::{Capture, Clock, Encoder, Events, Fs, Log};
use crate::Res;
use std::sync::{Arc, Mutex};

/// Dados mutáveis protegidos por `Mutex`.
struct Inner {
    st: St,
    /// Instante (ms monotônicos) em que o trecho atual começou.
    since: u64,
    /// Tempo já acumulado em trechos anteriores (ms).
    acc: u64,
    /// Caminho do arquivo em gravação.
    path: String,
}

/// Controla início, pausa, retomada e fim da gravação.
pub struct Recorder {
    cap: Arc<dyn Capture>,
    enc: Arc<dyn Encoder>,
    clock: Arc<dyn Clock>,
    fs: Arc<dyn Fs>,
    events: Arc<dyn Events>,
    log: Arc<dyn Log>,
    inner: Mutex<Inner>,
}

impl Recorder {
    pub fn new(
        cap: Arc<dyn Capture>,
        enc: Arc<dyn Encoder>,
        clock: Arc<dyn Clock>,
        fs: Arc<dyn Fs>,
        events: Arc<dyn Events>,
        log: Arc<dyn Log>,
    ) -> Self {
        Recorder {
            cap,
            enc,
            clock,
            fs,
            events,
            log,
            inner: Mutex::new(Inner {
                st: St::Idle,
                since: 0,
                acc: 0,
                path: String::new(),
            }),
        }
    }

    /// Estado atual com o cronômetro em segundos.
    pub fn state(&self) -> RecState {
        let i = lk(&self.inner);
        self.snapshot(&i)
    }

    /// Reemite `rec-state` (chamado a cada segundo pelo adaptador) se houver gravação.
    pub fn tick(&self) {
        let s = self.state();
        if s.state != St::Idle {
            self.events.rec_state(&s);
        }
    }

    fn snapshot(&self, i: &Inner) -> RecState {
        let run = if i.st == St::Rec {
            self.clock.mono_ms().saturating_sub(i.since)
        } else {
            0
        };
        RecState {
            state: i.st,
            secs: (i.acc + run) / 1000,
        }
    }

    fn emit(&self, i: &Inner) {
        self.events.rec_state(&self.snapshot(i));
    }

    /// Inicia a gravação (somente a partir de ocioso). Devolve os avisos da
    /// captura (ex.: um áudio que não abriu): a gravação segue sem ele.
    pub fn start(&self, cfg: &RecCfg) -> Res<Vec<String>> {
        let mut i = lk(&self.inner);
        if i.st != St::Idle {
            return Err("Já existe uma gravação em andamento.".into());
        }
        self.log
            .info(&format!("gravação: iniciar ({})", cfg_line(cfg)));
        validate(cfg)?;
        let dir = match &cfg.dir {
            Some(d) => d.clone(),
            None => self.fs.videos_dir()?,
        };
        self.fs.ensure_dir(&dir)?;
        let path = self.fs.join(&dir, &file_name(&self.clock.now()));
        self.enc.open(&path, cfg).inspect_err(|e| {
            self.log
                .error(&format!("encoder: falha ao abrir {path}: {e}"));
        })?;
        self.log.info(&format!("encoder: arquivo aberto em {path}"));
        let warns = match self.cap.start(cfg, self.enc.clone()) {
            Ok(w) => w,
            Err(e) => {
                // Fecha o arquivo aberto; o erro original é o que importa.
                let r = self.enc.finish();
                self.log.error(&format!("captura: falha ao iniciar: {e}"));
                if let Err(e2) = r {
                    self.log
                        .warn(&format!("encoder: falha ao fechar o arquivo: {e2}"));
                }
                return Err(e);
            }
        };
        for w in &warns {
            self.log.warn(&format!("captura: aviso ao usuário: {w}"));
        }
        self.log.info("captura: vídeo e áudio iniciados");
        *i = Inner {
            st: St::Rec,
            since: self.clock.mono_ms(),
            acc: 0,
            path,
        };
        self.emit(&i);
        self.log.info(&format!("gravação: iniciada em {}", i.path));
        Ok(warns)
    }

    /// Pausa a gravação em andamento.
    pub fn pause(&self) -> Res<()> {
        let mut i = lk(&self.inner);
        if i.st != St::Rec {
            return Err("Não há gravação ativa para pausar.".into());
        }
        self.cap.pause()?;
        i.acc += self.clock.mono_ms().saturating_sub(i.since);
        i.st = St::Paused;
        self.emit(&i);
        self.log.info(&format!(
            "gravação: pausada em {} s",
            self.snapshot(&i).secs
        ));
        Ok(())
    }

    /// Retoma uma gravação pausada.
    pub fn resume(&self) -> Res<()> {
        let mut i = lk(&self.inner);
        if i.st != St::Paused {
            return Err("Não há gravação pausada para retomar.".into());
        }
        self.cap.resume()?;
        i.since = self.clock.mono_ms();
        i.st = St::Rec;
        self.emit(&i);
        self.log.info("gravação: retomada");
        Ok(())
    }

    /// Finaliza a gravação e devolve o caminho do MP4.
    ///
    /// Sempre volta a ocioso, mesmo se a captura ou o encoder falharem.
    pub fn stop(&self) -> Res<String> {
        let mut i = lk(&self.inner);
        if i.st == St::Idle {
            return Err("Não há gravação em andamento.".into());
        }
        let secs = self.snapshot(&i).secs;
        let r_cap = self.cap.stop();
        let r_enc = self.enc.finish();
        let path = std::mem::take(&mut i.path);
        i.st = St::Idle;
        i.acc = 0;
        self.emit(&i);
        // Registra as duas falhas (o chamador só recebe a primeira).
        if let Err(e) = &r_cap {
            self.log.error(&format!("captura: falha ao parar: {e}"));
        }
        if let Err(e) = &r_enc {
            self.log.error(&format!("encoder: falha ao finalizar: {e}"));
        }
        r_cap?;
        r_enc?;
        self.log
            .info(&format!("gravação: finalizada ({secs} s) em {path}"));
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Mode, Region};
    use crate::fakes::*;

    struct Kit {
        cap: Arc<FakeCapture>,
        enc: Arc<FakeEncoder>,
        clock: Arc<FakeClock>,
        fs: Arc<FakeFs>,
        ev: Arc<FakeEvents>,
        log: Arc<FakeLog>,
        rec: Recorder,
    }

    fn kit() -> Kit {
        let cap = Arc::new(FakeCapture::default());
        let enc = Arc::new(FakeEncoder::default());
        let clock = Arc::new(FakeClock::default());
        let fs = Arc::new(FakeFs::default());
        let ev = Arc::new(FakeEvents::default());
        let log = Arc::new(FakeLog::default());
        let rec = Recorder::new(
            cap.clone(),
            enc.clone(),
            clock.clone(),
            fs.clone(),
            ev.clone(),
            log.clone(),
        );
        Kit {
            cap,
            enc,
            clock,
            fs,
            ev,
            log,
            rec,
        }
    }

    fn cfg() -> RecCfg {
        RecCfg {
            mode: Mode::Screen,
            target_id: None,
            region: None,
            mic_id: None,
            sys: false,
            fps: 30,
            dir: None,
        }
    }

    #[test]
    fn ciclo_completo_com_cronometro() {
        let k = kit();
        assert_eq!(k.rec.state(), RecState::idle());
        k.rec.start(&cfg()).unwrap();
        assert_eq!(
            k.enc.calls.all()[0],
            "open:/videos/kapture-20260102-030405.mp4"
        );
        assert_eq!(k.fs.calls.all(), vec!["videos_dir", "ensure:/videos"]);
        k.clock.advance(2500);
        assert_eq!(
            k.rec.state(),
            RecState {
                state: St::Rec,
                secs: 2
            }
        );
        k.rec.pause().unwrap();
        k.clock.advance(10_000); // pausado não conta
        assert_eq!(
            k.rec.state(),
            RecState {
                state: St::Paused,
                secs: 2
            }
        );
        k.rec.resume().unwrap();
        k.clock.advance(1500);
        assert_eq!(k.rec.state().secs, 4);
        let path = k.rec.stop().unwrap();
        assert_eq!(path, "/videos/kapture-20260102-030405.mp4");
        assert_eq!(k.rec.state(), RecState::idle());
        assert_eq!(
            k.cap.calls.all(),
            vec!["start:30", "pause", "resume", "stop"]
        );
        let sts: Vec<St> = lk(&k.ev.states).iter().map(|s| s.state).collect();
        assert_eq!(sts, vec![St::Rec, St::Paused, St::Rec, St::Idle]);
    }

    #[test]
    fn start_usa_dir_da_cfg() {
        let k = kit();
        let mut c = cfg();
        c.dir = Some("D:/x".into());
        k.rec.start(&c).unwrap();
        assert_eq!(k.fs.calls.all(), vec!["ensure:D:/x"]);
        assert!(k.rec.stop().unwrap().starts_with("D:/x/kapture-"));
    }

    #[test]
    fn captura_alimenta_encoder() {
        let k = kit();
        k.rec.start(&cfg()).unwrap();
        let e = k.cap.encoder().unwrap();
        e.write_audio(&[1, 2], 3).unwrap();
        assert!(k.enc.calls.all().contains(&"audio:2:3".to_string()));
    }

    #[test]
    fn transicoes_invalidas() {
        let k = kit();
        assert!(k.rec.pause().unwrap_err().0.contains("pausar"));
        assert!(k.rec.resume().unwrap_err().0.contains("retomar"));
        assert!(k.rec.stop().unwrap_err().0.contains("em andamento"));
        k.rec.start(&cfg()).unwrap();
        assert!(k.rec.start(&cfg()).unwrap_err().0.contains("Já existe"));
        assert!(k.rec.resume().is_err());
        k.rec.pause().unwrap();
        assert!(k.rec.pause().is_err());
        // parar estando pausado é permitido
        assert!(k.rec.stop().is_ok());
    }

    #[test]
    fn start_valida_cfg() {
        let k = kit();
        let mut c = cfg();
        c.mode = Mode::Area;
        assert!(k.rec.start(&c).is_err());
        c.mode = Mode::Screen;
        c.region = Some(Region {
            x: 0,
            y: 0,
            w: 1,
            h: 1,
        });
        c.fps = 0;
        assert!(k.rec.start(&c).is_err());
        assert_eq!(k.rec.state(), RecState::idle());
    }

    #[test]
    fn start_falhas_em_cada_etapa() {
        for (op, which) in [("videos_dir", "fs"), ("ensure_dir", "fs"), ("open", "enc")] {
            let k = kit();
            match which {
                "fs" => k.fs.fail.set(op, op),
                _ => k.enc.fail.set(op, op),
            }
            assert_eq!(k.rec.start(&cfg()).unwrap_err().0, op);
            assert_eq!(k.rec.state(), RecState::idle());
            assert!(lk(&k.ev.states).is_empty());
        }
        // falha na captura fecha o encoder e mantém o erro original
        let k = kit();
        k.cap.fail.set("start", "cap");
        assert_eq!(k.rec.start(&cfg()).unwrap_err().0, "cap");
        assert_eq!(k.enc.calls.all().last().unwrap(), "finish");
        assert_eq!(k.rec.state(), RecState::idle());
        assert!(k.log.has("captura: falha ao iniciar: cap"));
        // falha ao fechar o arquivo também é registrada
        let k = kit();
        k.cap.fail.set("start", "cap");
        k.enc.fail.set("finish", "fin");
        assert_eq!(k.rec.start(&cfg()).unwrap_err().0, "cap");
        assert!(k.log.has("falha ao fechar o arquivo: fin"));
    }

    #[test]
    fn registra_o_ciclo_no_log() {
        let k = kit();
        k.rec.start(&cfg()).unwrap();
        k.clock.advance(2000);
        k.rec.pause().unwrap();
        k.rec.resume().unwrap();
        k.rec.stop().unwrap();
        let m = k.log.msgs();
        assert!(m[0].starts_with("gravação: iniciar (modo=Screen"));
        assert_eq!(
            m[1],
            "encoder: arquivo aberto em /videos/kapture-20260102-030405.mp4"
        );
        assert_eq!(m[2], "captura: vídeo e áudio iniciados");
        assert_eq!(
            m[3],
            "gravação: iniciada em /videos/kapture-20260102-030405.mp4"
        );
        assert_eq!(m[4], "gravação: pausada em 2 s");
        assert_eq!(m[5], "gravação: retomada");
        assert_eq!(
            m[6],
            "gravação: finalizada (2 s) em /videos/kapture-20260102-030405.mp4"
        );
    }

    #[test]
    fn avisos_da_captura_sobem_e_vao_ao_log() {
        let k = kit();
        *lk(&k.cap.warns) = vec!["sem microfone".to_string()];
        assert_eq!(k.rec.start(&cfg()).unwrap(), vec!["sem microfone"]);
        assert!(k.log.has("captura: aviso ao usuário: sem microfone"));
        assert_eq!(k.rec.state().state, St::Rec);
    }

    #[test]
    fn falha_ao_abrir_o_encoder_vai_ao_log() {
        let k = kit();
        k.enc.fail.set("open", "sem disco");
        assert!(k.rec.start(&cfg()).is_err());
        assert!(k
            .log
            .has("encoder: falha ao abrir /videos/kapture-20260102-030405.mp4: sem disco"));
    }

    #[test]
    fn pause_resume_com_falha_mantem_estado() {
        let k = kit();
        k.rec.start(&cfg()).unwrap();
        k.cap.fail.set("pause", "p");
        assert_eq!(k.rec.pause().unwrap_err().0, "p");
        assert_eq!(k.rec.state().state, St::Rec);
        k.cap.fail.clear("pause");
        k.rec.pause().unwrap();
        k.cap.fail.set("resume", "r");
        assert_eq!(k.rec.resume().unwrap_err().0, "r");
        assert_eq!(k.rec.state().state, St::Paused);
    }

    #[test]
    fn stop_sempre_volta_a_ocioso() {
        let k = kit();
        k.rec.start(&cfg()).unwrap();
        k.cap.fail.set("stop", "cap");
        k.enc.fail.set("finish", "enc");
        assert_eq!(k.rec.stop().unwrap_err().0, "cap");
        assert_eq!(k.rec.state(), RecState::idle());
        assert_eq!(lk(&k.ev.states).last().unwrap().state, St::Idle);
        assert!(k.log.has("captura: falha ao parar: cap"));
        assert!(k.log.has("encoder: falha ao finalizar: enc"));

        let k = kit();
        k.rec.start(&cfg()).unwrap();
        k.enc.fail.set("finish", "enc");
        assert_eq!(k.rec.stop().unwrap_err().0, "enc");
        assert_eq!(k.rec.state(), RecState::idle());
    }

    #[test]
    fn tick_so_emite_se_gravando() {
        let k = kit();
        k.rec.tick();
        assert!(lk(&k.ev.states).is_empty());
        k.rec.start(&cfg()).unwrap();
        k.clock.advance(3000);
        k.rec.tick();
        assert_eq!(lk(&k.ev.states).last().unwrap().secs, 3);
    }
}
