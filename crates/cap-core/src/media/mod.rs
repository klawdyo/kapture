//! Mídia pura (sem Windows): cálculos de vídeo, mixagem de áudio e o canal
//! do seletor de região. Fica no núcleo para ser testada em qualquer SO;
//! os adaptadores reais apenas chamam estas funções.

mod audio;
mod pick;
mod plan;
mod time;
mod url;
mod video;

pub use audio::*;
pub use pick::*;
pub use plan::*;
pub use time::*;
pub use url::*;
pub use video::*;
