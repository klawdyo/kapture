//! Núcleo do Cap (Rust puro, sem Tauri/Windows).
//!
//! Camadas: `domain` (tipos e regras), `ports` (traits das capacidades
//! externas), `app` (casos de uso), `media` (cálculos puros de vídeo/áudio)
//! e `fakes` (implementações de teste).

pub mod app;
pub mod domain;
pub mod error;
pub mod fakes;
pub mod media;
pub mod ports;

pub use error::{CapErr, Res};

use std::sync::{Mutex, MutexGuard};

/// Trava um `Mutex` ignorando envenenamento (um pânico em outra thread
/// não deve derrubar a gravação).
pub(crate) fn lk<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn lk_ignora_envenenamento() {
        let m = Arc::new(Mutex::new(1));
        let m2 = m.clone();
        let _ = std::thread::spawn(move || {
            let _g = m2.lock().unwrap();
            panic!("envenena");
        })
        .join();
        assert!(m.is_poisoned());
        assert_eq!(*lk(&m), 1);
    }
}
