//! Adaptadores reais das portas do `cap-core`.
//!
//! Os que só usam a API do Tauri compilam em qualquer SO; os que tocam
//! APIs do Windows ficam atrás de `#[cfg(windows)]`.

mod cam;
mod events;
mod noupd;
mod picker;

pub use cam::TauriCam;
pub use events::TauriEvents;
pub use noupd::NoUpdater;
pub use picker::TauriPicker;

#[cfg(windows)]
pub mod win;
