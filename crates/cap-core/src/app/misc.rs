//! Casos de uso simples: listagens, webcam, seletor de região e VU meter.

use crate::domain::{level_i16, region_ok, Devices, Region, Targets as TargetsData};
use crate::ports::{Cam, Capture, Devs, Events, Picker, Targets};
use crate::Res;
use std::sync::Arc;

/// Lista alvos de captura e dispositivos.
pub struct Lister {
    targets: Arc<dyn Targets>,
    devs: Arc<dyn Devs>,
}

impl Lister {
    pub fn new(targets: Arc<dyn Targets>, devs: Arc<dyn Devs>) -> Self {
        Lister { targets, devs }
    }
    /// Monitores e janelas.
    pub fn targets(&self) -> Res<TargetsData> {
        self.targets.list()
    }
    /// Câmeras e microfones.
    pub fn devices(&self) -> Res<Devices> {
        self.devs.list()
    }
}

/// Mostra/oculta a janela da webcam.
pub struct CamCtl {
    cam: Arc<dyn Cam>,
}

impl CamCtl {
    pub fn new(cam: Arc<dyn Cam>) -> Self {
        CamCtl { cam }
    }
    /// Liga ou desliga a webcam.
    pub fn toggle(&self, visible: bool, dev_id: Option<&str>) -> Res<()> {
        if visible {
            self.cam.show(dev_id)
        } else {
            self.cam.hide()
        }
    }
}

/// Seleção de região da tela.
pub struct RegionPick {
    picker: Arc<dyn Picker>,
}

impl RegionPick {
    pub fn new(picker: Arc<dyn Picker>) -> Self {
        RegionPick { picker }
    }
    /// Abre o seletor; `None` se o usuário cancelou.
    pub fn pick(&self) -> Res<Option<Region>> {
        match self.picker.pick()? {
            Some(r) if !region_ok(&r) => Err("A área selecionada é muito pequena.".into()),
            other => Ok(other),
        }
    }
}

/// Medidor de nível do microfone (VU meter).
pub struct MicMeter {
    cap: Arc<dyn Capture>,
    events: Arc<dyn Events>,
}

impl MicMeter {
    pub fn new(cap: Arc<dyn Capture>, events: Arc<dyn Events>) -> Self {
        MicMeter { cap, events }
    }
    /// Começa a medir e emitir `mic-level` para o microfone indicado.
    pub fn start(&self, mic_id: &str) -> Res<()> {
        let events = self.events.clone();
        self.cap.mic_start(
            mic_id,
            Arc::new(move |pcm: &[i16]| events.mic_level(level_i16(pcm))),
        )
    }
    /// Para de medir e zera o indicador.
    pub fn stop(&self) -> Res<()> {
        let r = self.cap.mic_stop();
        self.events.mic_level(0.0);
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Dev, Win};
    use crate::fakes::*;
    use crate::lk;

    #[test]
    fn lister_repassa_e_propaga_erros() {
        let t = Arc::new(FakeTargets {
            data: TargetsData {
                monitors: vec![],
                wins: vec![Win {
                    id: 1,
                    title: "a".into(),
                }],
            },
            ..Default::default()
        });
        let d = Arc::new(FakeDevs {
            data: Devices {
                cams: vec![],
                mics: vec![Dev {
                    id: "m".into(),
                    name: "M".into(),
                }],
            },
            ..Default::default()
        });
        let l = Lister::new(t.clone(), d.clone());
        assert_eq!(l.targets().unwrap().wins.len(), 1);
        assert_eq!(l.devices().unwrap().mics.len(), 1);
        t.fail.set("list", "t");
        d.fail.set("list", "d");
        assert_eq!(l.targets().unwrap_err().0, "t");
        assert_eq!(l.devices().unwrap_err().0, "d");
    }

    #[test]
    fn toggle_cam() {
        let c = Arc::new(FakeCam::default());
        let ctl = CamCtl::new(c.clone());
        ctl.toggle(true, Some("c1")).unwrap();
        ctl.toggle(false, None).unwrap();
        assert_eq!(c.calls.all(), vec!["show:c1", "hide"]);
        c.fail.set("hide", "h");
        assert!(ctl.toggle(false, None).is_err());
    }

    #[test]
    fn pick_region() {
        let p = Arc::new(FakePicker::default());
        let rp = RegionPick::new(p.clone());
        assert_eq!(rp.pick().unwrap(), None);
        let ok = Region {
            x: 0,
            y: 0,
            w: 100,
            h: 80,
        };
        *lk(&p.result) = Some(ok);
        assert_eq!(rp.pick().unwrap(), Some(ok));
        *lk(&p.result) = Some(Region {
            x: 0,
            y: 0,
            w: 2,
            h: 2,
        });
        assert!(rp.pick().unwrap_err().0.contains("pequena"));
        p.fail.set("pick", "boom");
        assert_eq!(rp.pick().unwrap_err().0, "boom");
    }

    #[test]
    fn mic_meter() {
        let cap = Arc::new(FakeCapture::default());
        let ev = Arc::new(FakeEvents::default());
        let m = MicMeter::new(cap.clone(), ev.clone());
        m.start("mic1").unwrap();
        cap.push_mic(&[16384, -16384]);
        assert!((lk(&ev.levels)[0] - 0.5).abs() < 1e-6);
        m.stop().unwrap();
        assert_eq!(*lk(&ev.levels).last().unwrap(), 0.0);
        cap.fail.set("mic_start", "a");
        cap.fail.set("mic_stop", "b");
        assert_eq!(m.start("mic1").unwrap_err().0, "a");
        assert_eq!(m.stop().unwrap_err().0, "b");
    }
}
