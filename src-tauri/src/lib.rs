//! kapture: raiz de composição. Monta as portas reais, injeta nos casos de
//! uso do `cap-core` e registra os comandos Tauri.

mod adapters;
mod commands;
mod logging;

use adapters::{TauriCam, TauriEvents, TauriPicker};
use cap_core::app::{CamCtl, Lister, LoggedEnc, MicMeter, Recorder, RegionPick, Update, UrlOpen};
use cap_core::media::PickChan;
use cap_core::ports::{Capture, Clock, Devs, Encoder, Fs, Log, Opener, Targets, Updater};
use commands::Core;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, WindowEvent};

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
    opener: Arc<dyn Opener>,
    updater: Arc<dyn Updater>,
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
        opener: Arc::new(WinOpener),
        updater: Arc::new(VpkUpdater::new()),
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
        opener: Arc::new(FakeOpener::default()),
        // Sem instalador: a interface cai no aviso simples (consulta ao GitHub).
        updater: Arc::new(FakeUpdater {
            supported: Mutex::new(false),
            ..Default::default()
        }),
    }
}

/// Monta o grafo de objetos.
fn wire(app: AppHandle) -> Arc<Core> {
    let p = plat();
    let log: Arc<dyn Log> = logging::get();
    let events = Arc::new(TauriEvents::new(app.clone()));
    let chan = Arc::new(PickChan::new());
    let picker = Arc::new(TauriPicker::new(app.clone(), chan.clone()));
    let cam = Arc::new(TauriCam::new(app));
    // O encoder passa por um decorador que registra o progresso e as falhas.
    let enc = Arc::new(LoggedEnc::new(p.enc, log.clone(), p.clock.clone()));
    Arc::new(Core {
        rec: Arc::new(Recorder::new(
            p.cap.clone(),
            enc,
            p.clock,
            p.fs,
            events.clone(),
            log.clone(),
        )),
        lister: Lister::new(p.targets, p.devs, log.clone()),
        cam: CamCtl::new(cam, log.clone()),
        pick: RegionPick::new(picker, log.clone()),
        meter: MicMeter::new(p.cap, events.clone(), log.clone()),
        url: UrlOpen::new(p.opener, log.clone()),
        upd: Update::new(p.updater, events, log.clone()),
        chan,
        log,
    })
}

/// Folga (px lógicos) entre a barra e a borda inferior da área de trabalho.
const BAR_GAP: f64 = 28.0;

/// Centraliza a barra na base da área de trabalho (acima da barra de tarefas).
fn place_toolbar(app: &AppHandle) {
    let Some(w) = app.get_webview_window("toolbar") else {
        return;
    };
    if let (Ok(Some(m)), Ok(s)) = (w.current_monitor(), w.outer_size()) {
        let wa = m.work_area();
        let gap = (BAR_GAP * m.scale_factor()) as i32;
        let x = wa.position.x + (wa.size.width as i32 - s.width as i32) / 2;
        let y = wa.position.y + wa.size.height as i32 - s.height as i32 - gap;
        let _ = w.set_position(PhysicalPosition::new(x, y));
    }
    // A barra não aparece na gravação (a webcam, que é outra janela, sim).
    let _ = w.set_content_protected(true);
}

/// Redimensiona a barra (px lógicos) mantendo o centro e a borda inferior.
pub(crate) fn fit_toolbar(app: &AppHandle, w: f64, h: f64) -> Result<(), String> {
    let win = app
        .get_webview_window("toolbar")
        .ok_or("Janela da barra não encontrada.")?;
    let e = |e: tauri::Error| e.to_string();
    let (pos, old) = (
        win.outer_position().map_err(e)?,
        win.outer_size().map_err(e)?,
    );
    let new: tauri::PhysicalSize<i32> =
        LogicalSize::new(w, h).to_physical(win.scale_factor().map_err(e)?);
    win.set_size(LogicalSize::new(w, h)).map_err(e)?;
    let x = pos.x + (old.width as i32 - new.width) / 2;
    let y = pos.y + old.height as i32 - new.height;
    win.set_position(PhysicalPosition::new(x, y)).map_err(e)
}

/// Ponto de entrada do app.
pub fn run() {
    // Primeiro o log: assim qualquer falha de inicialização fica registrada.
    logging::install_panic_hook();
    logging::log_start();
    // COM da thread principal em STA antes de qualquer janela (evita o pânico do OleInitialize).
    #[cfg(windows)]
    adapters::win::init_main_sta();
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
            logging::get().info("app pronto (janela da barra criada)");
            Ok(())
        })
        // Fechar a barra encerra o app (a janela da webcam, oculta, o manteria vivo).
        .on_window_event(|win, ev| {
            if win.label() == "toolbar" && matches!(ev, WindowEvent::Destroyed) {
                win.app_handle().exit(0);
            }
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
            commands::log_ui,
            commands::open_logs,
            commands::open_url,
            commands::update_start,
            commands::update_apply,
            commands::fit_toolbar,
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o kapture");
    logging::get().info("kapture encerrado normalmente");
}
