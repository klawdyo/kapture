//! Implementações FAKE das portas: gravam as chamadas e permitem injetar
//! falhas (`fail.set("op", "mensagem")`). Reutilizáveis em outros crates.

use crate::domain::{
    Devices, Frame, RecCfg, RecState, Region, Stamp, Targets as TargetsData, UpdInfo, UpdState,
};
use crate::lk;
use crate::log::Lvl;
use crate::ports::{
    Cam, Capture, Clock, Devs, Encoder, Events, Fs, Log, MicSink, Opener, Picker, Targets, Updater,
};
use crate::{CapErr, Res};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Registro das chamadas recebidas por um fake.
#[derive(Default)]
pub struct Calls(Mutex<Vec<String>>);

impl Calls {
    /// Anota uma chamada.
    pub fn log(&self, s: impl Into<String>) {
        lk(&self.0).push(s.into());
    }
    /// Todas as chamadas, em ordem.
    pub fn all(&self) -> Vec<String> {
        lk(&self.0).clone()
    }
}

/// Falhas injetáveis por nome de operação.
#[derive(Default)]
pub struct Inject(Mutex<HashMap<String, String>>);

impl Inject {
    /// Faz a operação `op` falhar com `msg`.
    pub fn set(&self, op: &str, msg: &str) {
        lk(&self.0).insert(op.to_string(), msg.to_string());
    }
    /// Remove a falha de `op`.
    pub fn clear(&self, op: &str) {
        lk(&self.0).remove(op);
    }
    /// Retorna o erro injetado para `op`, se houver.
    pub fn check(&self, op: &str) -> Res<()> {
        match lk(&self.0).get(op) {
            Some(m) => Err(CapErr(m.clone())),
            None => Ok(()),
        }
    }
}

/// Fake de `ports::Targets`. Operação: `list`.
#[derive(Default)]
pub struct FakeTargets {
    pub data: TargetsData,
    pub calls: Calls,
    pub fail: Inject,
}

impl Targets for FakeTargets {
    fn list(&self) -> Res<TargetsData> {
        self.calls.log("list");
        self.fail.check("list")?;
        Ok(self.data.clone())
    }
}

/// Fake de `ports::Devs`. Operação: `list`.
#[derive(Default)]
pub struct FakeDevs {
    pub data: Devices,
    pub calls: Calls,
    pub fail: Inject,
}

impl Devs for FakeDevs {
    fn list(&self) -> Res<Devices> {
        self.calls.log("list");
        self.fail.check("list")?;
        Ok(self.data.clone())
    }
}

/// Fake de `ports::Capture`. Operações: `start`, `pause`, `resume`, `stop`,
/// `mic_start`, `mic_stop`.
#[derive(Default)]
pub struct FakeCapture {
    pub calls: Calls,
    pub fail: Inject,
    sink: Mutex<Option<Arc<dyn Encoder>>>,
    mic: Mutex<Option<MicSink>>,
}

impl FakeCapture {
    /// Simula amostras chegando do microfone (se o medidor estiver ativo).
    pub fn push_mic(&self, pcm: &[i16]) {
        let sink = lk(&self.mic).clone();
        if let Some(f) = sink {
            f(pcm);
        }
    }
    /// Encoder recebido em `start` (para simular quadros/áudio).
    pub fn encoder(&self) -> Option<Arc<dyn Encoder>> {
        lk(&self.sink).clone()
    }
}

impl Capture for FakeCapture {
    fn start(&self, cfg: &RecCfg, sink: Arc<dyn Encoder>) -> Res<()> {
        self.calls.log(format!("start:{}", cfg.fps));
        self.fail.check("start")?;
        *lk(&self.sink) = Some(sink);
        Ok(())
    }
    fn pause(&self) -> Res<()> {
        self.calls.log("pause");
        self.fail.check("pause")
    }
    fn resume(&self) -> Res<()> {
        self.calls.log("resume");
        self.fail.check("resume")
    }
    fn stop(&self) -> Res<()> {
        self.calls.log("stop");
        *lk(&self.sink) = None;
        self.fail.check("stop")
    }
    fn mic_start(&self, mic_id: &str, sink: MicSink) -> Res<()> {
        self.calls.log(format!("mic_start:{mic_id}"));
        self.fail.check("mic_start")?;
        *lk(&self.mic) = Some(sink);
        Ok(())
    }
    fn mic_stop(&self) -> Res<()> {
        self.calls.log("mic_stop");
        *lk(&self.mic) = None;
        self.fail.check("mic_stop")
    }
}

/// Fake de `ports::Encoder`. Operações: `open`, `write_video`,
/// `write_audio`, `finish`.
#[derive(Default)]
pub struct FakeEncoder {
    pub calls: Calls,
    pub fail: Inject,
}

impl Encoder for FakeEncoder {
    fn open(&self, path: &str, _cfg: &RecCfg) -> Res<()> {
        self.calls.log(format!("open:{path}"));
        self.fail.check("open")
    }
    fn write_video(&self, frame: &Frame) -> Res<()> {
        self.calls.log(format!("video:{}", frame.ts_ms));
        self.fail.check("write_video")
    }
    fn write_audio(&self, pcm: &[i16], ts_ms: u64) -> Res<()> {
        self.calls.log(format!("audio:{}:{}", pcm.len(), ts_ms));
        self.fail.check("write_audio")
    }
    fn finish(&self) -> Res<()> {
        self.calls.log("finish");
        self.fail.check("finish")
    }
}

/// Relógio controlável.
pub struct FakeClock {
    stamp: Mutex<Stamp>,
    ms: Mutex<u64>,
}

impl Default for FakeClock {
    fn default() -> Self {
        FakeClock {
            stamp: Mutex::new(Stamp {
                y: 2026,
                mo: 1,
                d: 2,
                h: 3,
                mi: 4,
                s: 5,
            }),
            ms: Mutex::new(0),
        }
    }
}

impl FakeClock {
    /// Avança o relógio monotônico.
    pub fn advance(&self, ms: u64) {
        *lk(&self.ms) += ms;
    }
    /// Define a data/hora local.
    pub fn set_stamp(&self, s: Stamp) {
        *lk(&self.stamp) = s;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Stamp {
        *lk(&self.stamp)
    }
    fn mono_ms(&self) -> u64 {
        *lk(&self.ms)
    }
}

/// Fake de `ports::Fs`. Operações: `ensure_dir`, `videos_dir`.
pub struct FakeFs {
    pub videos: String,
    pub calls: Calls,
    pub fail: Inject,
}

impl Default for FakeFs {
    fn default() -> Self {
        FakeFs {
            videos: "/videos".to_string(),
            calls: Calls::default(),
            fail: Inject::default(),
        }
    }
}

impl Fs for FakeFs {
    fn ensure_dir(&self, dir: &str) -> Res<()> {
        self.calls.log(format!("ensure:{dir}"));
        self.fail.check("ensure_dir")
    }
    fn videos_dir(&self) -> Res<String> {
        self.calls.log("videos_dir");
        self.fail.check("videos_dir")?;
        Ok(self.videos.clone())
    }
    fn join(&self, dir: &str, name: &str) -> String {
        format!("{dir}/{name}")
    }
}

/// Fake de `ports::Cam`. Operações: `show`, `hide`.
#[derive(Default)]
pub struct FakeCam {
    pub calls: Calls,
    pub fail: Inject,
}

impl Cam for FakeCam {
    fn show(&self, dev_id: Option<&str>) -> Res<()> {
        self.calls.log(format!("show:{}", dev_id.unwrap_or("-")));
        self.fail.check("show")
    }
    fn hide(&self) -> Res<()> {
        self.calls.log("hide");
        self.fail.check("hide")
    }
}

/// Fake de `ports::Opener`. Operação: `open` (anota `open:<url>`).
#[derive(Default)]
pub struct FakeOpener {
    pub calls: Calls,
    pub fail: Inject,
}

impl Opener for FakeOpener {
    fn open(&self, url: &str) -> Res<()> {
        self.calls.log(format!("open:{url}"));
        self.fail.check("open")
    }
}

/// Fake de `ports::Picker`. Operação: `pick`.
#[derive(Default)]
pub struct FakePicker {
    pub result: Mutex<Option<Region>>,
    pub calls: Calls,
    pub fail: Inject,
}

impl Picker for FakePicker {
    fn pick(&self) -> Res<Option<Region>> {
        self.calls.log("pick");
        self.fail.check("pick")?;
        Ok(*lk(&self.result))
    }
}

/// Fake de `ports::Events`: guarda os eventos emitidos.
#[derive(Default)]
pub struct FakeEvents {
    pub levels: Mutex<Vec<f32>>,
    pub states: Mutex<Vec<RecState>>,
    pub updates: Mutex<Vec<UpdState>>,
}

impl Events for FakeEvents {
    fn mic_level(&self, level: f32) {
        lk(&self.levels).push(level);
    }
    fn rec_state(&self, st: &RecState) {
        lk(&self.states).push(*st);
    }
    fn update_state(&self, st: &UpdState) {
        lk(&self.updates).push(st.clone());
    }
}

/// Fake de `ports::Updater`. Operações: `check`, `download` e `apply`
/// (falham com `fail.set`). `supported` e `found` são ajustáveis; o
/// `download` reporta os percentuais de `steps`.
pub struct FakeUpdater {
    pub supported: Mutex<bool>,
    pub found: Mutex<Option<UpdInfo>>,
    pub steps: Mutex<Vec<u8>>,
    pub calls: Calls,
    pub fail: Inject,
}

impl Default for FakeUpdater {
    fn default() -> Self {
        FakeUpdater {
            supported: Mutex::new(true),
            found: Mutex::new(None),
            steps: Mutex::new(vec![]),
            calls: Calls::default(),
            fail: Inject::default(),
        }
    }
}

impl Updater for FakeUpdater {
    fn supported(&self) -> bool {
        *lk(&self.supported)
    }
    fn check(&self) -> Res<Option<UpdInfo>> {
        self.calls.log("check");
        self.fail.check("check")?;
        Ok(lk(&self.found).clone())
    }
    fn download(&self, progress: &dyn Fn(u8)) -> Res<()> {
        self.calls.log("download");
        for p in lk(&self.steps).iter() {
            progress(*p);
        }
        self.fail.check("download")
    }
    fn apply_and_restart(&self) -> Res<()> {
        self.calls.log("apply");
        self.fail.check("apply")
    }
}

/// Fake de `ports::Log`: guarda (nível, mensagem) em memória.
#[derive(Default)]
pub struct FakeLog(Mutex<Vec<(Lvl, String)>>);

impl FakeLog {
    /// Todas as mensagens, em ordem.
    pub fn all(&self) -> Vec<(Lvl, String)> {
        lk(&self.0).clone()
    }
    /// Só os textos, em ordem.
    pub fn msgs(&self) -> Vec<String> {
        lk(&self.0).iter().map(|(_, m)| m.clone()).collect()
    }
    /// Alguma mensagem contém `s`?
    pub fn has(&self, s: &str) -> bool {
        lk(&self.0).iter().any(|(_, m)| m.contains(s))
    }
}

impl Log for FakeLog {
    fn write(&self, lvl: Lvl, msg: &str) {
        lk(&self.0).push((lvl, msg.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Dev, Mode, Monitor, St};

    fn drop_pcm(_: &[i16]) {}

    #[test]
    fn drop_pcm_ignora() {
        drop_pcm(&[1]);
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
    fn inject_set_check_clear() {
        let i = Inject::default();
        assert!(i.check("a").is_ok());
        i.set("a", "ruim");
        assert_eq!(i.check("a").unwrap_err().0, "ruim");
        i.clear("a");
        assert!(i.check("a").is_ok());
    }

    #[test]
    fn targets_e_devs() {
        let t = FakeTargets {
            data: TargetsData {
                monitors: vec![Monitor {
                    id: 1,
                    name: "m".into(),
                    w: 1,
                    h: 1,
                    x: 0,
                    y: 0,
                    primary: true,
                }],
                wins: vec![],
            },
            ..Default::default()
        };
        assert_eq!(t.list().unwrap().monitors.len(), 1);
        t.fail.set("list", "x");
        assert!(t.list().is_err());
        assert_eq!(t.calls.all(), vec!["list", "list"]);

        let d = FakeDevs {
            data: Devices {
                cams: vec![Dev {
                    id: "c".into(),
                    name: "C".into(),
                }],
                mics: vec![],
            },
            ..Default::default()
        };
        assert_eq!(d.list().unwrap().cams.len(), 1);
        d.fail.set("list", "y");
        assert_eq!(d.list().unwrap_err().0, "y");
    }

    #[test]
    fn capture_completo() {
        let c = FakeCapture::default();
        assert!(c.encoder().is_none());
        c.push_mic(&[1]); // sem sink: não faz nada
        let enc: Arc<dyn Encoder> = Arc::new(FakeEncoder::default());
        c.start(&cfg(), enc).unwrap();
        assert!(c.encoder().is_some());
        c.pause().unwrap();
        c.resume().unwrap();
        c.stop().unwrap();
        assert!(c.encoder().is_none());

        let got = Arc::new(Mutex::new(0usize));
        let g = got.clone();
        c.mic_start("m", Arc::new(move |p: &[i16]| *lk(&g) += p.len()))
            .unwrap();
        c.push_mic(&[1, 2, 3]);
        assert_eq!(*lk(&got), 3);
        c.mic_stop().unwrap();
        c.push_mic(&[1]);
        assert_eq!(*lk(&got), 3);

        for op in ["start", "pause", "resume", "stop", "mic_start", "mic_stop"] {
            c.fail.set(op, op);
        }
        let enc: Arc<dyn Encoder> = Arc::new(FakeEncoder::default());
        assert_eq!(c.start(&cfg(), enc).unwrap_err().0, "start");
        assert_eq!(c.pause().unwrap_err().0, "pause");
        assert_eq!(c.resume().unwrap_err().0, "resume");
        assert_eq!(c.stop().unwrap_err().0, "stop");
        assert_eq!(
            c.mic_start("m", Arc::new(drop_pcm)).unwrap_err().0,
            "mic_start"
        );
        assert_eq!(c.mic_stop().unwrap_err().0, "mic_stop");
    }

    #[test]
    fn encoder_completo() {
        let e = FakeEncoder::default();
        e.open("p", &cfg()).unwrap();
        e.write_video(&Frame {
            w: 1,
            h: 1,
            ts_ms: 5,
            data: vec![],
        })
        .unwrap();
        e.write_audio(&[0, 0], 7).unwrap();
        e.finish().unwrap();
        assert_eq!(
            e.calls.all(),
            vec!["open:p", "video:5", "audio:2:7", "finish"]
        );
        for op in ["open", "write_video", "write_audio", "finish"] {
            e.fail.set(op, op);
        }
        assert!(e.open("p", &cfg()).is_err());
        assert!(e
            .write_video(&Frame {
                w: 1,
                h: 1,
                ts_ms: 0,
                data: vec![]
            })
            .is_err());
        assert!(e.write_audio(&[], 0).is_err());
        assert!(e.finish().is_err());
    }

    #[test]
    fn relogio() {
        let c = FakeClock::default();
        assert_eq!(c.mono_ms(), 0);
        c.advance(1500);
        assert_eq!(c.mono_ms(), 1500);
        assert_eq!(c.now().y, 2026);
        c.set_stamp(Stamp {
            y: 2030,
            mo: 1,
            d: 1,
            h: 0,
            mi: 0,
            s: 0,
        });
        assert_eq!(c.now().y, 2030);
    }

    #[test]
    fn fs_cam_picker_events() {
        let f = FakeFs::default();
        assert_eq!(f.videos_dir().unwrap(), "/videos");
        f.ensure_dir("d").unwrap();
        assert_eq!(f.join("d", "n"), "d/n");
        f.fail.set("ensure_dir", "e1");
        f.fail.set("videos_dir", "e2");
        assert_eq!(f.ensure_dir("d").unwrap_err().0, "e1");
        assert_eq!(f.videos_dir().unwrap_err().0, "e2");

        let c = FakeCam::default();
        c.show(Some("x")).unwrap();
        c.show(None).unwrap();
        c.hide().unwrap();
        assert_eq!(c.calls.all(), vec!["show:x", "show:-", "hide"]);
        c.fail.set("show", "s");
        c.fail.set("hide", "h");
        assert!(c.show(None).is_err());
        assert!(c.hide().is_err());

        let p = FakePicker::default();
        assert_eq!(p.pick().unwrap(), None);
        *lk(&p.result) = Some(Region {
            x: 1,
            y: 2,
            w: 3,
            h: 4,
        });
        assert_eq!(p.pick().unwrap().unwrap().w, 3);
        p.fail.set("pick", "p");
        assert!(p.pick().is_err());

        let e = FakeEvents::default();
        e.mic_level(0.5);
        e.rec_state(&RecState {
            state: St::Rec,
            secs: 1,
        });
        assert_eq!(*lk(&e.levels), vec![0.5]);
        assert_eq!(lk(&e.states)[0].state, St::Rec);
        e.update_state(&UpdState::Checking);
        assert_eq!(*lk(&e.updates), vec![UpdState::Checking]);

        let u = FakeUpdater::default();
        assert!(u.supported());
        assert_eq!(u.check().unwrap(), None);
        *lk(&u.found) = Some(UpdInfo {
            version: "2.0.0".into(),
        });
        assert_eq!(u.check().unwrap().unwrap().version, "2.0.0");
        *lk(&u.steps) = vec![10, 90];
        let seen = Mutex::new(vec![]);
        u.download(&|p| lk(&seen).push(p)).unwrap();
        assert_eq!(*lk(&seen), vec![10, 90]);
        u.apply_and_restart().unwrap();
        u.fail.set("check", "c");
        u.fail.set("download", "d");
        u.fail.set("apply", "a");
        assert!(
            u.check().is_err() && u.download(&|_| {}).is_err() && u.apply_and_restart().is_err()
        );
        *lk(&u.supported) = false;
        assert!(!u.supported());
        assert_eq!(u.calls.all().len(), 7);

        let l = FakeLog::default();
        l.debug("d");
        l.info("i");
        l.warn("w");
        l.error("e");
        assert_eq!(l.msgs(), vec!["d", "i", "w", "e"]);
        assert_eq!(l.all()[2], (Lvl::Warn, "w".to_string()));
        assert!(l.has("i") && !l.has("zzz"));
    }
}
