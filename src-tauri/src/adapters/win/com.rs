//! Utilitários de COM/Media Foundation compartilhados pelos adaptadores.

use std::sync::Once;
use windows::core::PWSTR;
use windows::Win32::Media::MediaFoundation::{MFStartup, MFSTARTUP_FULL, MF_VERSION};
use windows::Win32::System::Com::{CoInitializeEx, CoTaskMemFree, COINIT_MULTITHREADED};

thread_local! {
    /// Inicializa o COM (MTA) uma vez por thread que usa as APIs.
    static COM: () = {
        // SAFETY: chamada padrão; "já inicializado" é ignorado de propósito.
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    };
}

/// Garante COM inicializado na thread atual.
pub fn init() {
    COM.with(|_| {});
}

/// Garante `MFStartup` (uma vez por processo; fica ativo até o fim).
pub fn mf_init() {
    static MF: Once = Once::new();
    init();
    MF.call_once(|| {
        // SAFETY: inicialização global do Media Foundation.
        let _ = unsafe { MFStartup(MF_VERSION, MFSTARTUP_FULL) };
    });
}

/// Converte uma `PWSTR` alocada pelo COM em `String` e a libera.
pub fn take_pwstr(p: PWSTR) -> String {
    if p.is_null() {
        return String::new();
    }
    // SAFETY: `p` é string terminada em nulo alocada por COM; liberada uma vez.
    unsafe {
        let s = p.to_string().unwrap_or_default();
        CoTaskMemFree(Some(p.0 as *const _));
        s
    }
}

/// Texto UTF-16 terminado em nulo (para passar a APIs `PCWSTR`).
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
