//! Adaptador `Fs`: pasta Vídeos do usuário e criação de diretórios.

use super::com::{init, take_pwstr};
use cap_core::media::join_win;
use cap_core::ports::Fs;
use cap_core::{CapErr, Res};
use windows::Win32::UI::Shell::{FOLDERID_Videos, SHGetKnownFolderPath, KF_FLAG_DEFAULT};

/// Sistema de arquivos real.
pub struct WinFs;

impl Fs for WinFs {
    fn ensure_dir(&self, dir: &str) -> Res<()> {
        std::fs::create_dir_all(dir)
            .map_err(|e| CapErr::from(format!("Não foi possível criar a pasta de vídeos: {e}")))
    }

    fn videos_dir(&self) -> Res<String> {
        init();
        // SAFETY: devolve string alocada pelo COM, liberada em `take_pwstr`.
        let p = unsafe { SHGetKnownFolderPath(&FOLDERID_Videos, KF_FLAG_DEFAULT, None) }
            .map_err(|_| CapErr::from("Não foi possível localizar a pasta Vídeos."))?;
        Ok(take_pwstr(p))
    }

    fn join(&self, dir: &str, name: &str) -> String {
        join_win(dir, name)
    }
}
