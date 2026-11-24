//! Cap: raiz de composição. Monta as portas reais, injeta nos casos de
//! uso do `cap-core` e registra os comandos Tauri.

mod adapters;
mod commands;

use adapters::{TauriCam, TauriEvents, TauriPicker};
use cap_core::app::{CamCtl, Lister, MicMeter, Recorder, RegionPick};
use cap_core::media::PickChan;
use cap_core::ports::{Capture, Clock, Devs, Encoder, Fs, Targets};
use commands::Core;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tauri::{AppHandle, Manager, PhysicalPosition};

/// Trava um `Mutex` ignorando envenenamento.
pub(crate) fn lk<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Portas dependentes da plataforma.
struct Plat {
    targets: Arc<dyn Targets>,
    devs: Arc<dyn Devs>,
    cap: Arc<dyn Capture>,
    enc: Arc<dyn Encoder>,
    clock: Arc<dyn Clock>,
    fs: Arc<dyn Fs>,
}

/// Windows: adaptadores reais.
#[cfg(windows)]
fn plat() -> Plat {
    use adapters::win::*;
    Plat {
        targets: Arc::new(WinTargets),
        devs: Arc::new(WinDevs),
        cap: Arc::new(WinCapture::new()),
        enc: Arc::new(MfEncoder::new()),
        clock: Arc::new(WinClock::new()),
        fs: Arc::new(WinFs),
    }
}

/// Outros SOs (só para desenvolver a interface): usa os fakes do núcleo.
#[cfg(not(windows))]
fn plat() -> Plat {
    use cap_core::fakes::*;
    Plat {
        targets: Arc::new(FakeTargets::default()),
        devs: Arc::new(FakeDevs::default()),
        cap: Arc::new(FakeCapture::default()),
        enc: Arc::new(FakeEncoder::default()),
        clock: Arc::new(FakeClock::default()),
        fs: Arc::new(FakeFs::default()),
    }
}

/// Monta o grafo de objetos.
fn wire(app: AppHandle) -> Arc<Core> {
    let p = plat();
    let events = Arc::new(TauriEvents::new(app.clone()));
    let chan = Arc::new(PickChan::new());
    let picker = Arc::new(TauriPicker::new(app.clone(), chan.clone()));
    let cam = Arc::new(TauriCam::new(app));
    Arc::new(Core {
        rec: Arc::new(Recorder::new(
            p.cap.clone(),
            p.enc,
            p.clock,
            p.fs,
            events.clone(),
        )),
        lister: Lister::new(p.targets, p.devs),
        cam: CamCtl::new(cam),
        pick: RegionPick::new(picker),
        meter: MicMeter::new(p.cap, events),
        chan,
    })
}

/// Centraliza a barra no topo do monitor atual.
fn place_toolbar(app: &AppHandle) {
    let Some(w) = app.get_webview_window("toolbar") else {
        return;
    };
    if let (Ok(Some(m)), Ok(s)) = (w.current_monitor(), w.outer_size()) {
        let x = m.position().x + (m.size().width as i32 - s.width as i32) / 2;
        let _ = w.set_position(PhysicalPosition::new(x, m.position().y + 16));
    }
    // A barra não aparece na gravação (a webcam, que é outra janela, sim).
    let _ = w.set_content_protected(true);
}

/// Ponto de entrada do app.
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let core = wire(app.handle().clone());
            // Tick de 1 s: reemite `rec-state` para o cronômetro da interface.
            let rec = core.rec.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));
                rec.tick();
            });
            app.manage(core);
            place_toolbar(app.handle());
            #[cfg(windows)]
            adapters::win::hotkey::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_targets,
            commands::list_devices,
            commands::toggle_cam,
            commands::pick_region,
            commands::submit_region,
            commands::start_rec,
            commands::pause_rec,
            commands::resume_rec,
            commands::stop_rec,
            commands::get_state,
            commands::mic_start,
            commands::mic_stop,
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o Cap");
}
