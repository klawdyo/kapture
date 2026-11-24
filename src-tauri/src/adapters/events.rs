//! Adaptador `Events`: publica eventos para as janelas via Tauri.

use cap_core::domain::{RecState, UpdState};
use cap_core::ports::Events;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Carga do evento `mic-level`.
#[derive(Serialize, Clone)]
struct Level {
    level: f32,
}

/// Emite `mic-level`, `rec-state` e `update-state` para todas as janelas.
pub struct TauriEvents {
    app: AppHandle,
}

impl TauriEvents {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Events for TauriEvents {
    fn mic_level(&self, level: f32) {
        // Falha de emissão não deve interromper a gravação.
        let _ = self.app.emit("mic-level", Level { level });
    }

    fn rec_state(&self, st: &RecState) {
        let _ = self.app.emit("rec-state", *st);
    }

    fn update_state(&self, st: &UpdState) {
        let _ = self.app.emit("update-state", st);
    }
}
