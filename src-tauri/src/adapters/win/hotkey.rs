//! Atalho global Ctrl+Shift+R via `RegisterHotKey` (mais leve que um plugin).

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
        if unsafe { RegisterHotKey(None, 1, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, 'R' as u32) }
            .is_err()
        {
            return;
        }
        let mut msg = MSG::default();
        // SAFETY: `msg` é válido; GetMessageW devolve 0/-1 ao encerrar.
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.0 > 0 {
            if msg.message == WM_HOTKEY {
                let _ = app.emit("hotkey", ());
            }
        }
    });
}
