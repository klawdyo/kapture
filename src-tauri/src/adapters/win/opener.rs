//! Adaptador `Opener`: abre a URL no navegador padrão via `ShellExecuteW`.

use cap_core::ports::Opener;
use cap_core::Res;
use windows::core::{w, HSTRING};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// Abertura de URLs pelo shell do Windows.
pub struct WinOpener;

impl Opener for WinOpener {
    fn open(&self, url: &str) -> Res<()> {
        let url = HSTRING::from(url);
        // SAFETY: strings terminadas em NUL válidas durante a chamada; sem janela pai.
        let r = unsafe { ShellExecuteW(None, w!("open"), &url, None, None, SW_SHOWNORMAL) };
        // Por contrato, valores acima de 32 indicam sucesso.
        if r.0 as isize > 32 {
            return Ok(());
        }
        crate::logging::get().error(&format!("ShellExecuteW: código {}", r.0 as isize));
        Err("Não foi possível abrir o navegador.".into())
    }
}
