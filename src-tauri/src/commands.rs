//! Camada fina de comandos Tauri: só traduz argumentos/erros e delega
//! aos casos de uso do `cap-core`. Erros chegam à interface como texto
//! em português (contrato).

use cap_core::app::{CamCtl, Lister, MicMeter, Recorder, RegionPick, UrlOpen};
use cap_core::domain::{Devices, RecCfg, RecState, Region, Targets};
use cap_core::log::{log_ipc, log_ui as ui_line};
use cap_core::media::PickChan;
use cap_core::ports::Log;
use cap_core::Res;
use std::sync::Arc;
use std::time::Instant;
use tauri::State;

/// Raiz de composição entregue aos comandos (`app.manage`).
pub struct Core {
    pub rec: Arc<Recorder>,
    pub lister: Lister,
    pub cam: CamCtl,
    pub pick: RegionPick,
    pub meter: MicMeter,
    pub url: UrlOpen,
    pub chan: Arc<PickChan>,
    pub log: Arc<dyn Log>,
}

/// Estado compartilhado nos comandos.
type St<'a> = State<'a, Arc<Core>>;

/// Roda um caso de uso numa thread de trabalho (nunca na principal, pois
/// criar janelas ou esperar o usuário ali travaria o app), converte o erro
/// e registra o comando no log com o resultado e a duração.
async fn blk<T, F>(st: St<'_>, name: &'static str, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&Core) -> Res<T> + Send + 'static,
{
    let t0 = Instant::now();
    let core = st.inner().clone();
    let r = tauri::async_runtime::spawn_blocking(move || f(&core).map_err(|e| e.0))
        .await
        .map_err(|e| {
            st.log
                .error(&format!("ipc {name}: a thread de trabalho falhou: {e}"));
            "Falha interna ao executar a operação.".to_string()
        })
        .and_then(|r| r);
    log_ipc(&*st.log, name, t0.elapsed().as_millis() as u64, &r);
    r
}

#[tauri::command]
pub async fn list_targets(st: St<'_>) -> Result<Targets, String> {
    blk(st, "list_targets", |c| c.lister.targets()).await
}

#[tauri::command]
pub async fn list_devices(st: St<'_>) -> Result<Devices, String> {
    blk(st, "list_devices", |c| c.lister.devices()).await
}

#[tauri::command]
pub async fn toggle_cam(st: St<'_>, visible: bool, dev_id: Option<String>) -> Result<(), String> {
    blk(st, "toggle_cam", move |c| {
        c.cam.toggle(visible, dev_id.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn pick_region(st: St<'_>) -> Result<Option<Region>, String> {
    blk(st, "pick_region", |c| c.pick.pick()).await
}

/// Chamado pela janela do seletor (`None` = cancelou).
#[tauri::command]
pub fn submit_region(st: St<'_>, region: Option<Region>) {
    st.log.info(&format!("ipc submit_region: {region:?}"));
    st.chan.submit(region);
}

#[tauri::command]
pub async fn start_rec(st: St<'_>, mut cfg: RecCfg) -> Result<(), String> {
    // Microfone vazio significa "sem microfone".
    cfg.mic_id = cfg.mic_id.filter(|m| !m.is_empty());
    blk(st, "start_rec", move |c| {
        c.rec.start(&cfg)?;
        // O VU meter é só visual: se falhar, a gravação segue. Se já mede
        // este microfone (ligado pela barra), `start` reaproveita sem erro.
        if let Some(m) = &cfg.mic_id {
            let _ = c.meter.start(m);
        }
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn pause_rec(st: St<'_>) -> Result<(), String> {
    blk(st, "pause_rec", |c| c.rec.pause()).await
}

#[tauri::command]
pub async fn resume_rec(st: St<'_>) -> Result<(), String> {
    blk(st, "resume_rec", |c| c.rec.resume()).await
}

#[tauri::command]
pub async fn stop_rec(st: St<'_>) -> Result<String, String> {
    // O medidor continua ligado: quem o desliga é `mic_stop` (a barra).
    blk(st, "stop_rec", |c| c.rec.stop()).await
}

/// Liga o VU meter (reinicia sem erro se já houver um ativo).
#[tauri::command]
pub async fn mic_start(st: St<'_>, mic_id: String) -> Result<(), String> {
    blk(st, "mic_start", move |c| c.meter.start(&mic_id)).await
}

/// Desliga o VU meter e zera o nível.
#[tauri::command]
pub async fn mic_stop(st: St<'_>) -> Result<(), String> {
    blk(st, "mic_stop", |c| c.meter.stop()).await
}

#[tauri::command]
pub fn get_state(st: St<'_>) -> RecState {
    st.rec.state()
}

/// Recebe uma mensagem da interface (erros do WebView) e grava no log.
#[tauri::command]
pub fn log_ui(st: St<'_>, lvl: String, msg: String) {
    ui_line(&*st.log, &lvl, &msg);
}

/// Abre a pasta de logs no Explorer (cria a pasta se ainda não existir).
#[tauri::command]
pub fn open_logs(st: St<'_>) -> Result<(), String> {
    let dir = crate::logging::dir();
    let _ = std::fs::create_dir_all(&dir);
    let r = std::process::Command::new("explorer")
        .arg(&dir)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Não foi possível abrir a pasta de logs: {e}"));
    log_ipc(&*st.log, "open_logs", 0, &r);
    r
}

/// Abre no navegador padrão uma página de release do kapture (só
/// `https://github.com/klawdyo/kapture/releases/*`; o resto é recusado).
#[tauri::command]
pub fn open_url(st: St<'_>, url: String) -> Result<(), String> {
    st.url.open(&url).map_err(|e| e.0)
}

/// Redimensiona a janela da barra (px lógicos), mantendo centro e base.
#[tauri::command]
pub fn fit_toolbar(app: tauri::AppHandle, w: f64, h: f64) -> Result<(), String> {
    crate::fit_toolbar(&app, w, h)
}
