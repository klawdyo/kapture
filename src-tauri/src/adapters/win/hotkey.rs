//! Atalho global Ctrl+Shift+R via `RegisterHotKey` (mais leve que um plugin).

use super::com::log_err;
use crate::logging;
use tauri::{AppHandle, Emitter};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

/// Inicia a thread com o laço de mensagens do atalho; cada acionamento
/// emite o evento `hotkey`.
pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        // Se outro programa já usa o atalho, seguimos sem ele.
        // SAFETY: registra na fila de mensagens desta própria thread.
        if let Err(e) =
            unsafe { RegisterHotKey(None, 1, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, 'R' as u32) }
        {
            log_err(
                "atalho Ctrl+Shift+R: RegisterHotKey falhou (já em uso?)",
                &e,
            );
            return;
        }
        logging::get().info("atalho Ctrl+Shift+R registrado");
        let mut msg = MSG::default();
        // SAFETY: `msg` é válido; GetMessageW devolve 0/-1 ao encerrar.
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.0 > 0 {
            if msg.message == WM_HOTKEY {
                logging::get().info("atalho Ctrl+Shift+R acionado");
                let _ = app.emit("hotkey", ());
            }
        }
    });
}
