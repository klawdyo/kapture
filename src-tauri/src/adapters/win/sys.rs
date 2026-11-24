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
