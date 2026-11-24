//! Portas: capacidades externas (objeto-seguras, `Send + Sync`).
//!
//! Atenção: `Targets` aqui é a porta; o tipo de dados é `domain::Targets`.

use crate::domain::{Devices, Frame, RecCfg, RecState, Region, Stamp};
use crate::Res;
use std::sync::Arc;

/// Função chamada com amostras PCM de 16 bits do microfone.
pub type MicSink = Arc<dyn Fn(&[i16]) + Send + Sync>;

/// Lista monitores e janelas.
pub trait Targets: Send + Sync {
    fn list(&self) -> Res<crate::domain::Targets>;
}

/// Lista câmeras e microfones.
pub trait Devs: Send + Sync {
    fn list(&self) -> Res<Devices>;
}

/// Captura de tela/áudio; alimenta o `Encoder` durante a gravação.
pub trait Capture: Send + Sync {
    fn start(&self, cfg: &RecCfg, sink: Arc<dyn Encoder>) -> Res<()>;
    fn pause(&self) -> Res<()>;
    fn resume(&self) -> Res<()>;
    fn stop(&self) -> Res<()>;
    /// Abre o microfone só para medir o nível (VU meter).
    fn mic_start(&self, mic_id: &str, sink: MicSink) -> Res<()>;
    fn mic_stop(&self) -> Res<()>;
}

/// Codifica vídeo/áudio em MP4.
pub trait Encoder: Send + Sync {
    fn open(&self, path: &str, cfg: &RecCfg) -> Res<()>;
    fn write_video(&self, frame: &Frame) -> Res<()>;
    fn write_audio(&self, pcm: &[i16], ts_ms: u64) -> Res<()>;
    fn finish(&self) -> Res<()>;
}

/// Relógio injetável.
pub trait Clock: Send + Sync {
    /// Data/hora local.
    fn now(&self) -> Stamp;
    /// Milissegundos monotônicos (para o cronômetro).
    fn mono_ms(&self) -> u64;
}

/// Sistema de arquivos.
pub trait Fs: Send + Sync {
    fn ensure_dir(&self, dir: &str) -> Res<()>;
    /// Pasta de vídeos padrão do usuário.
    fn videos_dir(&self) -> Res<String>;
    /// Junta diretório e nome com o separador da plataforma.
    fn join(&self, dir: &str, name: &str) -> String;
}

/// Janela da webcam.
pub trait Cam: Send + Sync {
    fn show(&self, dev_id: Option<&str>) -> Res<()>;
    fn hide(&self) -> Res<()>;
}

/// Seletor de região (None = cancelado).
pub trait Picker: Send + Sync {
    fn pick(&self) -> Res<Option<Region>>;
}

/// Emissão de eventos para a interface.
pub trait Events: Send + Sync {
    fn mic_level(&self, level: f32);
    fn rec_state(&self, st: &RecState);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Garante que todas as portas são objeto-seguras e Send + Sync.
    #[test]
    fn portas_sao_objeto_seguras() {
        fn ok<T: ?Sized + Send + Sync>() {}
        ok::<dyn Targets>();
        ok::<dyn Devs>();
        ok::<dyn Capture>();
        ok::<dyn Encoder>();
        ok::<dyn Clock>();
        ok::<dyn Fs>();
        ok::<dyn Cam>();
        ok::<dyn Picker>();
        ok::<dyn Events>();
    }
}
