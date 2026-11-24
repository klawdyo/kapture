//! Camada fina de comandos Tauri: só traduz argumentos/erros e delega
//! aos casos de uso do `cap-core`. Erros chegam à interface como texto
//! em português (contrato).

use cap_core::app::{CamCtl, Lister, MicMeter, Recorder, RegionPick};
use cap_core::domain::{Devices, RecCfg, RecState, Region, Targets};
use cap_core::media::PickChan;
use cap_core::Res;
use std::sync::Arc;
use tauri::State;

/// Raiz de composição entregue aos comandos (`app.manage`).
pub struct Core {
    pub rec: Arc<Recorder>,
    pub lister: Lister,
    pub cam: CamCtl,
    pub pick: RegionPick,
    pub meter: MicMeter,
    pub chan: Arc<PickChan>,
}

/// Estado compartilhado nos comandos.
type St<'a> = State<'a, Arc<Core>>;

/// Roda um caso de uso numa thread de trabalho (nunca na principal, pois
/// criar janelas ou esperar o usuário ali travaria o app) e converte o erro.
async fn blk<T, F>(st: St<'_>, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&Core) -> Res<T> + Send + 'static,
{
    let core = st.inner().clone();
    tauri::async_runtime::spawn_blocking(move || f(&core).map_err(|e| e.0))
        .await
        .map_err(|_| "Falha interna ao executar a operação.".to_string())?
}

#[tauri::command]
pub async fn list_targets(st: St<'_>) -> Result<Targets, String> {
    blk(st, |c| c.lister.targets()).await
}

#[tauri::command]
pub async fn list_devices(st: St<'_>) -> Result<Devices, String> {
    blk(st, |c| c.lister.devices()).await
}

#[tauri::command]
pub async fn toggle_cam(st: St<'_>, visible: bool, dev_id: Option<String>) -> Result<(), String> {
    blk(st, move |c| c.cam.toggle(visible, dev_id.as_deref())).await
}

#[tauri::command]
pub async fn pick_region(st: St<'_>) -> Result<Option<Region>, String> {
    blk(st, |c| c.pick.pick()).await
}

/// Chamado pela janela do seletor (`None` = cancelou).
#[tauri::command]
pub fn submit_region(st: St<'_>, region: Option<Region>) {
    st.chan.submit(region);
}

#[tauri::command]
pub async fn start_rec(st: St<'_>, mut cfg: RecCfg) -> Result<(), String> {
    // Microfone vazio significa "sem microfone".
    cfg.mic_id = cfg.mic_id.filter(|m| !m.is_empty());
    blk(st, move |c| {
        c.rec.start(&cfg)?;
        // O VU meter é só visual: se falhar, a gravação segue.
        if let Some(m) = &cfg.mic_id {
            let _ = c.meter.start(m);
        }
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn pause_rec(st: St<'_>) -> Result<(), String> {
    blk(st, |c| c.rec.pause()).await
}

#[tauri::command]
pub async fn resume_rec(st: St<'_>) -> Result<(), String> {
    blk(st, |c| c.rec.resume()).await
}

#[tauri::command]
pub async fn stop_rec(st: St<'_>) -> Result<String, String> {
    blk(st, |c| {
        let r = c.rec.stop();
        let _ = c.meter.stop();
        r
    })
    .await
}

#[tauri::command]
pub fn get_state(st: St<'_>) -> RecState {
    st.rec.state()
}
