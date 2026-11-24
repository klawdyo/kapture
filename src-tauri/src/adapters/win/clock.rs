//! Adaptador `Clock`: hora local do Windows e relógio monotônico.

use cap_core::domain::Stamp;
use cap_core::ports::Clock;
use std::time::Instant;
use windows::Win32::System::SystemInformation::GetLocalTime;

/// Relógio real.
pub struct WinClock {
    t0: Instant,
}

impl WinClock {
    pub fn new() -> Self {
        Self { t0: Instant::now() }
    }
}

impl Clock for WinClock {
    fn now(&self) -> Stamp {
        // SAFETY: GetLocalTime não tem pré-condições.
        let t = unsafe { GetLocalTime() };
        Stamp {
            y: t.wYear,
            mo: t.wMonth as u8,
            d: t.wDay as u8,
            h: t.wHour as u8,
            mi: t.wMinute as u8,
            s: t.wSecond as u8,
        }
    }

    fn mono_ms(&self) -> u64 {
        self.t0.elapsed().as_millis() as u64
    }
}
