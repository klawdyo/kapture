//! Informações do sistema: versão do Windows.

use windows::Wdk::System::SystemServices::RtlGetVersion;
use windows::Win32::System::SystemInformation::OSVERSIONINFOW;

/// Versão real do Windows (`maior.menor.build`). `RtlGetVersion` não é
/// afetada pelo "mascaramento" de versão que `GetVersionEx` sofre.
pub fn os_version() -> String {
    let mut v = OSVERSIONINFOW {
        dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };
    // SAFETY: `v` é válido e tem `dwOSVersionInfoSize` correto.
    let st = unsafe { RtlGetVersion(&mut v) };
    if st.is_ok() {
        format!(
            "{}.{}.{}",
            v.dwMajorVersion, v.dwMinorVersion, v.dwBuildNumber
        )
    } else {
        "desconhecida".to_string()
    }
}

/// Mostra uma caixa de erro do Windows (o app não tem console nem janela nessa hora).
pub fn alert(title: &str, msg: &str) {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    let t: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let m: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: os dois buffers terminam em zero e vivem até o fim da chamada.
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(m.as_ptr()),
            PCWSTR(t.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}
