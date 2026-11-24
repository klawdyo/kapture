//! Adaptador `Targets`: monitores (EnumDisplayMonitors) e janelas (EnumWindows).
//!
//! O `id` de monitor/janela é o próprio HMONITOR/HWND (os handles do
//! Windows usam só 32 bits significativos, mesmo em 64 bits).

use super::com::log_err;
use cap_core::domain::{Monitor, Targets as Data, Win};
use cap_core::ports::Targets;
use cap_core::{CapErr, Res};
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindow, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, GWL_EXSTYLE, GW_OWNER,
    MONITORINFOF_PRIMARY, WS_EX_TOOLWINDOW,
};

/// Lista de alvos real.
pub struct WinTargets;

/// Monitores conectados (coordenadas da área de trabalho virtual, em pixels físicos).
pub fn monitors() -> Vec<Monitor> {
    unsafe extern "system" fn cb(h: HMONITOR, _: HDC, _: *mut RECT, lp: LPARAM) -> BOOL {
        // SAFETY: `lp` aponta para o Vec vivo em `monitors()`.
        let v = unsafe { &mut *(lp.0 as *mut Vec<Monitor>) };
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        // SAFETY: `mi` tem `cbSize` correto.
        if unsafe { GetMonitorInfoW(h, &mut mi) }.as_bool() {
            let r = mi.rcMonitor;
            let primary = mi.dwFlags & MONITORINFOF_PRIMARY != 0;
            let n = v.len() + 1;
            let (w, h2) = ((r.right - r.left) as u32, (r.bottom - r.top) as u32);
            v.push(Monitor {
                id: h.0 as usize as u32,
                name: format!(
                    "Tela {n} ({w}x{h2}){}",
                    if primary { " - principal" } else { "" }
                ),
                w,
                h: h2,
                x: r.left,
                y: r.top,
                primary,
            });
        }
        TRUE
    }
    let mut v: Vec<Monitor> = Vec::new();
    // SAFETY: o callback só roda durante a chamada, com `v` vivo.
    unsafe {
        if let Err(e) =
            EnumDisplayMonitors(None, None, Some(cb), LPARAM(&mut v as *mut _ as isize)).ok()
        {
            log_err("EnumDisplayMonitors", &e);
        }
    }
    v
}

/// A janela deve aparecer na lista (visível, com título, de nível superior)?
fn listable(h: HWND, me: u32) -> bool {
    // SAFETY: consultas simples sobre um HWND recebido do EnumWindows.
    unsafe {
        if !IsWindowVisible(h).as_bool() || IsIconic(h).as_bool() {
            return false;
        }
        // Janelas-ferramenta e janelas "filhas" de outra (owned) não entram.
        let ex = GetWindowLongW(h, GWL_EXSTYLE) as u32;
        if ex & WS_EX_TOOLWINDOW.0 != 0 || GetWindow(h, GW_OWNER).is_ok_and(|o| !o.is_invalid()) {
            return false;
        }
        // Janelas "escondidas" do Windows 10/11 (UWP suspensos) estão cloaked.
        let mut cloaked = 0u32;
        let _ = DwmGetWindowAttribute(
            h,
            DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut _,
            std::mem::size_of::<u32>() as u32,
        );
        if cloaked != 0 || GetWindowTextLengthW(h) == 0 {
            return false;
        }
        // Não oferece as janelas do próprio kapture.
        let mut pid = 0u32;
        GetWindowThreadProcessId(h, Some(&mut pid));
        pid != me
    }
}

/// Janelas capturáveis.
pub fn windows_list() -> Vec<Win> {
    unsafe extern "system" fn cb(h: HWND, lp: LPARAM) -> BOOL {
        // SAFETY: `lp` aponta para o Vec vivo em `windows_list()`.
        let v = unsafe { &mut *(lp.0 as *mut Vec<Win>) };
        // SAFETY: sem pré-condições.
        let me = unsafe { GetCurrentProcessId() };
        if listable(h, me) {
            let mut buf = [0u16; 256];
            // SAFETY: buffer válido.
            let n = unsafe { GetWindowTextW(h, &mut buf) } as usize;
            let title = String::from_utf16_lossy(&buf[..n.min(buf.len())]);
            if title != "Program Manager" {
                v.push(Win {
                    id: h.0 as usize as u32,
                    title,
                });
            }
        }
        TRUE
    }
    let mut v: Vec<Win> = Vec::new();
    // SAFETY: o callback só roda durante a chamada, com `v` vivo.
    unsafe {
        if let Err(e) = EnumWindows(Some(cb), LPARAM(&mut v as *mut _ as isize)) {
            log_err("EnumWindows", &e);
        }
    }
    v
}

impl Targets for WinTargets {
    fn list(&self) -> Res<Data> {
        let monitors = monitors();
        if monitors.is_empty() {
            return Err(CapErr::from("Nenhum monitor foi encontrado."));
        }
        Ok(Data {
            monitors,
            wins: windows_list(),
        })
    }
}
