//! Casos de uso simples: listagens, webcam, seletor de região e VU meter.

use crate::domain::{level_i16, region_ok, Devices, Region, Targets as TargetsData};
use crate::lk;
use crate::ports::{Cam, Capture, Devs, Events, Log, Picker, Targets};
use crate::Res;
use std::sync::{Arc, Mutex};

/// Lista alvos de captura e dispositivos.
pub struct Lister {
    targets: Arc<dyn Targets>,
    devs: Arc<dyn Devs>,
    log: Arc<dyn Log>,
}

impl Lister {
    pub fn new(targets: Arc<dyn Targets>, devs: Arc<dyn Devs>, log: Arc<dyn Log>) -> Self {
        Lister { targets, devs, log }
    }
    /// Monitores e janelas (contagens e nomes vão para o log).
    pub fn targets(&self) -> Res<TargetsData> {
        let r = self.targets.list();
        match &r {
            Ok(t) => {
                let mons: Vec<&str> = t.monitors.iter().map(|m| m.name.as_str()).collect();
                let wins: Vec<&str> = t.wins.iter().map(|w| w.title.as_str()).collect();
                self.log.info(&format!(
                    "alvos: {} monitores {mons:?}, {} janelas {wins:?}",
                    mons.len(),
                    wins.len()
                ));
            }
            Err(e) => self.log.error(&format!("alvos: erro: {e}")),
        }
        r
    }
    /// Câmeras e microfones (contagens e nomes vão para o log).
    pub fn devices(&self) -> Res<Devices> {
        let r = self.devs.list();
        match &r {
            Ok(d) => {
                let cams: Vec<&str> = d.cams.iter().map(|c| c.name.as_str()).collect();
                let mics: Vec<&str> = d.mics.iter().map(|c| c.name.as_str()).collect();
                self.log.info(&format!(
                    "dispositivos: {} câmeras {cams:?}, {} microfones {mics:?}",
                    cams.len(),
                    mics.len()
                ));
            }
            Err(e) => self.log.error(&format!("dispositivos: erro: {e}")),
        }
        r
    }
}

/// Mostra/oculta a janela da webcam.
pub struct CamCtl {
    cam: Arc<dyn Cam>,
    log: Arc<dyn Log>,
}

impl CamCtl {
    pub fn new(cam: Arc<dyn Cam>, log: Arc<dyn Log>) -> Self {
        CamCtl { cam, log }
    }
    /// Liga ou desliga a webcam.
    pub fn toggle(&self, visible: bool, dev_id: Option<&str>) -> Res<()> {
        let r = if visible {
            self.log
                .info(&format!("webcam: mostrar janela (dev={dev_id:?})"));
            self.cam.show(dev_id)
        } else {
            self.log.info("webcam: ocultar janela");
            self.cam.hide()
        };
        if let Err(e) = &r {
            self.log.error(&format!("webcam: erro: {e}"));
        }
        r
    }
}

/// Seleção de região da tela.
pub struct RegionPick {
    picker: Arc<dyn Picker>,
    log: Arc<dyn Log>,
}

impl RegionPick {
    pub fn new(picker: Arc<dyn Picker>, log: Arc<dyn Log>) -> Self {
        RegionPick { picker, log }
    }
    /// Abre o seletor; `None` se o usuário cancelou.
    pub fn pick(&self) -> Res<Option<Region>> {
        self.log.info("seletor de área: abrindo janela");
        let r = self.picker.pick();
        match &r {
            Ok(Some(g)) => self.log.info(&format!("seletor de área: região {g:?}")),
            Ok(None) => self.log.info("seletor de área: cancelado"),
            Err(e) => self.log.error(&format!("seletor de área: erro: {e}")),
        }
        match r? {
            Some(r) if !region_ok(&r) => Err("A área selecionada é muito pequena.".into()),
            other => Ok(other),
        }
    }
}

/// Medidor de nível do microfone (VU meter).
///
/// Regra: no máximo um medidor ativo. Iniciar com o mesmo microfone já
/// medido é um no-op (reaproveita, p.ex. quando a gravação começa com o
/// medidor ligado); com outro microfone, reinicia sem erro.
pub struct MicMeter {
    cap: Arc<dyn Capture>,
    events: Arc<dyn Events>,
    log: Arc<dyn Log>,
    cur: Mutex<Option<String>>,
}

impl MicMeter {
    pub fn new(cap: Arc<dyn Capture>, events: Arc<dyn Events>, log: Arc<dyn Log>) -> Self {
        MicMeter {
            cap,
            events,
            log,
            cur: Mutex::new(None),
        }
    }
    /// Começa a medir e emitir `mic-level` para o microfone indicado.
    pub fn start(&self, mic_id: &str) -> Res<()> {
        let mut cur = lk(&self.cur);
        if cur.as_deref() == Some(mic_id) {
            return Ok(());
        }
        // Troca de microfone: o anterior para antes (erro ignorado).
        if cur.take().is_some() {
            let _ = self.cap.mic_stop();
        }
        let events = self.events.clone();
        self.log
            .info(&format!("medidor do microfone: iniciar ({mic_id})"));
        let r = self.cap.mic_start(
            mic_id,
            Arc::new(move |pcm: &[i16]| events.mic_level(level_i16(pcm))),
        );
        if let Err(e) = &r {
            self.log.error(&format!("medidor do microfone: erro: {e}"));
        }
        r?;
        *cur = Some(mic_id.to_string());
        Ok(())
    }
    /// Para de medir e zera o indicador.
    pub fn stop(&self) -> Res<()> {
        lk(&self.cur).take();
        let r = self.cap.mic_stop();
        self.events.mic_level(0.0);
        match &r {
            Ok(()) => self.log.info("medidor do microfone: parado"),
            Err(e) => self
                .log
                .error(&format!("medidor do microfone: erro ao parar: {e}")),
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Dev, Monitor, Win};
    use crate::fakes::*;
    use crate::lk;

    #[test]
    fn lister_repassa_e_propaga_erros() {
        let t = Arc::new(FakeTargets {
            data: TargetsData {
                monitors: vec![Monitor {
                    id: 1,
                    name: "Tela 1".into(),
                    w: 1,
                    h: 1,
                    x: 0,
                    y: 0,
                    primary: true,
                }],
                wins: vec![Win {
                    id: 1,
                    title: "a".into(),
                }],
            },
            ..Default::default()
        });
        let d = Arc::new(FakeDevs {
            data: Devices {
                cams: vec![Dev {
                    id: "c".into(),
                    name: "C".into(),
                }],
                mics: vec![Dev {
                    id: "m".into(),
                    name: "M".into(),
                }],
            },
            ..Default::default()
        });
        let log = Arc::new(FakeLog::default());
        let l = Lister::new(t.clone(), d.clone(), log.clone());
        assert_eq!(l.targets().unwrap().wins.len(), 1);
        assert_eq!(l.devices().unwrap().mics.len(), 1);
        assert_eq!(
            log.msgs(),
            vec![
                "alvos: 1 monitores [\"Tela 1\"], 1 janelas [\"a\"]",
                "dispositivos: 1 câmeras [\"C\"], 1 microfones [\"M\"]",
            ]
        );
        t.fail.set("list", "t");
        d.fail.set("list", "d");
        assert_eq!(l.targets().unwrap_err().0, "t");
        assert_eq!(l.devices().unwrap_err().0, "d");
        assert!(log.has("alvos: erro: t") && log.has("dispositivos: erro: d"));
    }

    #[test]
    fn toggle_cam() {
        let c = Arc::new(FakeCam::default());
        let log = Arc::new(FakeLog::default());
        let ctl = CamCtl::new(c.clone(), log.clone());
        ctl.toggle(true, Some("c1")).unwrap();
        ctl.toggle(false, None).unwrap();
        assert_eq!(c.calls.all(), vec!["show:c1", "hide"]);
        c.fail.set("hide", "h");
        assert!(ctl.toggle(false, None).is_err());
        assert_eq!(
            log.msgs(),
            vec![
                "webcam: mostrar janela (dev=Some(\"c1\"))",
                "webcam: ocultar janela",
                "webcam: ocultar janela",
                "webcam: erro: h",
            ]
        );
    }

    #[test]
    fn pick_region() {
        let p = Arc::new(FakePicker::default());
        let log = Arc::new(FakeLog::default());
        let rp = RegionPick::new(p.clone(), log.clone());
        assert_eq!(rp.pick().unwrap(), None);
        assert!(log.has("cancelado"));
        let ok = Region {
            x: 0,
            y: 0,
            w: 100,
            h: 80,
        };
        *lk(&p.result) = Some(ok);
        assert_eq!(rp.pick().unwrap(), Some(ok));
        assert!(log.has("região Region { x: 0, y: 0, w: 100, h: 80 }"));
        *lk(&p.result) = Some(Region {
            x: 0,
            y: 0,
            w: 2,
            h: 2,
        });
        assert!(rp.pick().unwrap_err().0.contains("pequena"));
        p.fail.set("pick", "boom");
        assert_eq!(rp.pick().unwrap_err().0, "boom");
        assert!(log.has("seletor de área: erro: boom"));
    }

    #[test]
    fn mic_meter() {
        let cap = Arc::new(FakeCapture::default());
        let ev = Arc::new(FakeEvents::default());
        let log = Arc::new(FakeLog::default());
        let m = MicMeter::new(cap.clone(), ev.clone(), log.clone());
        m.start("mic1").unwrap();
        cap.push_mic(&[16384, -16384]);
        assert!((lk(&ev.levels)[0] - 0.5).abs() < 1e-6);
        m.stop().unwrap();
        assert_eq!(*lk(&ev.levels).last().unwrap(), 0.0);
        assert!(log.has("iniciar (mic1)") && log.has("parado"));
        cap.fail.set("mic_start", "a");
        cap.fail.set("mic_stop", "b");
        assert_eq!(m.start("mic1").unwrap_err().0, "a");
        assert_eq!(m.stop().unwrap_err().0, "b");
        assert!(log.has("medidor do microfone: erro: a"));
        assert!(log.has("erro ao parar: b"));
    }

    #[test]
    fn mic_meter_reaproveita_e_reinicia() {
        let cap = Arc::new(FakeCapture::default());
        let ev = Arc::new(FakeEvents::default());
        let m = MicMeter::new(cap.clone(), ev, Arc::new(FakeLog::default()));
        m.start("a").unwrap();
        m.start("a").unwrap(); // mesmo microfone: reaproveita
        assert_eq!(cap.calls.all().len(), 1);
        m.start("b").unwrap(); // outro: para o anterior e reinicia
        assert_eq!(cap.calls.all(), ["mic_start:a", "mic_stop", "mic_start:b"]);
        // Falha ao iniciar não deixa o medidor marcado como ativo.
        m.stop().unwrap();
        cap.fail.set("mic_start", "x");
        assert!(m.start("a").is_err());
        cap.fail.clear("mic_start");
        m.start("a").unwrap(); // não ficou marcado: inicia de fato
        assert_eq!(cap.calls.all().last().unwrap(), "mic_start:a");
    }
}
