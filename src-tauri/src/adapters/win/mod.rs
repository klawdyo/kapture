//! Adaptadores que usam APIs do Windows (Win32/WinRT/Media Foundation).

mod capture;
mod clock;
mod com;
mod devs;
mod encoder;

mod fs;
pub mod hotkey;
mod targets;
mod worker;

pub use capture::WinCapture;
pub use clock::WinClock;
pub use devs::WinDevs;
pub use encoder::MfEncoder;

pub use fs::WinFs;
pub use targets::WinTargets;
