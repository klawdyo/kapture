//! Utilitários de COM/Media Foundation compartilhados pelos adaptadores.

use std::sync::Once;
use windows::core::{Error, PWSTR};
use windows::Win32::Media::MediaFoundation::{MFStartup, MFSTARTUP_FULL, MF_VERSION};
use windows::Win32::System::Com::{
    CoInitializeEx, CoTaskMemFree, COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED,
};

thread_local! {
    /// Inicializa o COM (MTA) uma vez por thread que usa as APIs.
    static COM: () = {
        // SAFETY: chamada padrão; "já inicializado" (S_FALSE) não é erro.
        if let Err(e) = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.ok() {
            log_err("CoInitializeEx", &e);
        }
    };
}

/// Registra uma falha do Windows: contexto, HRESULT em hexadecimal e texto.
pub fn log_err(ctx: &str, e: &Error) {
    crate::logging::get().error(&format!(
        "{ctx}: HRESULT {} ({})",
        cap_core::log::hresult(e.code().0),
        e.message().trim()
    ));
}

/// Inicializa o COM da thread principal em modo apartamento (STA), antes de qualquer janela e de
/// qualquer outra chamada de COM. O tao (OleInitialize) e o WebView2 exigem STA; se a thread já
/// estiver em MTA, o tao entra em pânico com RPC_E_CHANGED_MODE e o app some sem janela.
/// Não registra no log: ele ainda não existe nesse ponto (use `log_main_sta` depois).
pub fn init_main_sta() -> windows::core::Result<()> {
    // SAFETY: chamada padrão do COM na thread atual.
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.ok()
}

/// Registra no log o resultado de `init_main_sta`.
pub fn log_main_sta(r: windows::core::Result<()>) {
    match r {
        Ok(()) => crate::logging::get().info("COM da thread principal em modo STA"),
        Err(e) => log_err("CoInitializeEx(STA) na thread principal", &e),
    }
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
        if let Err(e) = unsafe { MFStartup(MF_VERSION, MFSTARTUP_FULL) } {
            log_err("MFStartup", &e);
        }
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
