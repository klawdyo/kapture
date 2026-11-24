//! Adaptador `Picker`: abre a janela `region-selector` e espera a
//! resposta que chega por `submit_region` através do canal puro.

use crate::logging;
use cap_core::domain::Region;
use cap_core::media::PickChan;
use cap_core::ports::Picker;
use cap_core::{CapErr, Res};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager, WebviewWindowBuilder, WindowEvent};

const LABEL: &str = "region-selector";
const ERR: &str = "Não foi possível abrir o seletor de área.";

/// Seletor de região em tela cheia.
pub struct TauriPicker {
    app: AppHandle,
    chan: Arc<PickChan>,
}

impl TauriPicker {
    pub fn new(app: AppHandle, chan: Arc<PickChan>) -> Self {
        Self { app, chan }
    }
}

impl Picker for TauriPicker {
    /// Bloqueia até a resposta; chame fora da thread principal.
    fn pick(&self) -> Res<Option<Region>> {
        // Se um seletor anterior ainda está fechando, espera a janela sumir.
        for _ in 0..25 {
            if self.app.get_webview_window(LABEL).is_none() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let rx = self.chan.begin();
        let cfg = self
            .app
            .config()
            .app
            .windows
            .iter()
            .find(|w| w.label == LABEL)
            .cloned()
            .ok_or_else(|| {
                logging::get().error("seletor de área: configuração da janela não encontrada");
                ERR
            })?;
        let win = WebviewWindowBuilder::from_config(&self.app, &cfg)
            .and_then(|b| b.build())
            .map_err(|e| {
                logging::get().error(&format!("seletor de área: falha ao criar a janela: {e}"));
                CapErr::from(ERR)
            })?;
        logging::get().info("seletor de área: janela criada");
        // Fechar a janela (Alt+F4) sem responder equivale a cancelar.
        let chan = self.chan.clone();
        win.on_window_event(move |e| {
            if matches!(e, WindowEvent::Destroyed) {
                chan.submit(None);
            }
        });
        let r = rx.recv().map_err(|_| {
            logging::get().error("seletor de área: canal de resposta encerrado");
            CapErr::from(ERR)
        });
        let _ = win.close();
        logging::get().info("seletor de área: janela fechada");
        r
    }
}
