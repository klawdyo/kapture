//! Adaptadores que usam APIs do Windows (Win32/WinRT/Media Foundation).

mod capture;
mod clock;
mod com;
mod devs;
mod encoder;

mod fs;
pub mod hotkey;
mod opener;
mod sys;
mod targets;
mod worker;

pub use capture::WinCapture;
pub use clock::WinClock;
pub use com::{init_main_sta, log_main_sta};
pub use devs::WinDevs;
pub use encoder::MfEncoder;

pub use fs::WinFs;
pub use opener::WinOpener;
pub use sys::{alert, os_version};
pub use targets::WinTargets;
