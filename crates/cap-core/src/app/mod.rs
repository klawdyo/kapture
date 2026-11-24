//! Casos de uso: recebem as portas por injeção (`Arc<dyn Porta>`).

mod logged;
mod misc;
mod recorder;

pub use logged::*;
pub use misc::*;
pub use recorder::*;
