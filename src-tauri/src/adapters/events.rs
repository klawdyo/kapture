//! Adaptador `Events`: publica eventos para as janelas via Tauri.

use cap_core::domain::{protected, RecState, UpdState};
use cap_core::ports::Events;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};

/// Carga do evento `mic-level`.
#[derive(Serialize, Clone)]
struct Level {
    level: f32,
}

/// Emite `mic-level`, `rec-state` e `update-state` para todas as janelas.
pub struct TauriEvents {
    app: AppHandle,
    /// Última proteção aplicada à barra (evita reaplicar a cada tique de 1 s).
    on: AtomicBool,
}

impl TauriEvents {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            on: AtomicBool::new(false),
        }
    }

    /// Liga ou desliga a exclusão da barra da captura conforme a regra pura `protected`.
    /// Só gravando a barra some do vídeo; ocioso, o Print Screen a enxerga.
    fn shield(&self, st: &RecState) {
        let on = protected(st.state);
        if self.on.swap(on, Ordering::SeqCst) != on {
            if let Some(w) = self.app.get_webview_window("toolbar") {
                let _ = w.set_content_protected(on);
            }
        }
    }
}

impl Events for TauriEvents {
    fn mic_level(&self, level: f32) {
        // Falha de emissão não deve interromper a gravação.
        let _ = self.app.emit("mic-level", Level { level });
    }

    fn rec_state(&self, st: &RecState) {
        self.shield(st);
        let _ = self.app.emit("rec-state", *st);
    }

    fn update_state(&self, st: &UpdState) {
        let _ = self.app.emit("update-state", st);
    }
}
