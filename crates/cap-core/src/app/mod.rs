//! Casos de uso: recebem as portas por injeção (`Arc<dyn Porta>`).

mod logged;
mod misc;
mod recorder;
mod upd;

pub use logged::*;
pub use misc::*;
pub use recorder::*;
pub use upd::*;
