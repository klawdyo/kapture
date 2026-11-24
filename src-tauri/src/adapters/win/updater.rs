//! Adaptador `Updater` com Velopack: único ponto do app que conhece o crate
//! `velopack`. Para trocar de mecanismo basta outro tipo que implemente a
//! porta `Updater` e uma linha em `lib.rs`.
//!
//! O feed é o próprio Release mais recente do GitHub: `releases/latest/download/`
//! redireciona para os assets dele (`releases.win.json` e os `.nupkg`), que o
//! `vpk pack` gera e o `release.yml` anexa. O `ureq` do Velopack segue o redirect.

use crate::lk;
use cap_core::domain::UpdInfo;
use cap_core::ports::Updater;
use cap_core::Res;
use std::sync::mpsc;
use std::sync::Mutex;
use velopack::sources::HttpSource;
use velopack::{UpdateCheck, UpdateInfo, UpdateManager};

/// Feed do Velopack (diretório com `releases.win.json` e os pacotes).
pub const FEED: &str = "https://github.com/klawdyo/kapture/releases/latest/download/";

/// Atualização pelo Velopack. `mgr` é `None` quando o app não foi instalado
/// pelo Setup do Velopack (exe solto, MSI ou desenvolvimento).
pub struct VpkUpdater {
    mgr: Option<UpdateManager>,
    found: Mutex<Option<UpdateInfo>>,
}

impl VpkUpdater {
    pub fn new() -> Self {
        let mgr = match UpdateManager::new(HttpSource::new(FEED), None, None) {
            Ok(m) => Some(m),
            Err(e) => {
                crate::logging::get()
                    .info(&format!("velopack: app não instalado pelo Setup ({e})"));
                None
            }
        };
        VpkUpdater {
            mgr,
            found: Mutex::new(None),
        }
    }

    /// O gerenciador, ou o erro padrão de "não instalado".
    fn mgr(&self) -> Res<&UpdateManager> {
        self.mgr
            .as_ref()
            .ok_or_else(|| "O app não foi instalado pelo instalador do kapture.".into())
    }
}

/// Erro do Velopack como mensagem do núcleo.
fn err(e: velopack::Error) -> cap_core::CapErr {
    format!("Falha na atualização: {e}").into()
}

impl Updater for VpkUpdater {
    fn supported(&self) -> bool {
        self.mgr.is_some()
    }

    fn check(&self) -> Res<Option<UpdInfo>> {
        match self.mgr()?.check_for_updates().map_err(err)? {
            UpdateCheck::UpdateAvailable(i) => {
                let version = i.TargetFullRelease.Version.clone();
                *lk(&self.found) = Some(*i);
                Ok(Some(UpdInfo { version }))
            }
            // Feed vazio ou sem versão maior que a instalada.
            _ => Ok(None),
        }
    }

    fn download(&self, progress: &dyn Fn(u8)) -> Res<()> {
        let mgr = self.mgr()?;
        let info = lk(&self.found)
            .clone()
            .ok_or("Nenhuma atualização encontrada para baixar.")?;
        let (tx, rx) = mpsc::channel::<i16>();
        // O download bloqueia; o progresso chega pelo canal e é repassado aqui.
        std::thread::scope(|s| {
            let h = s.spawn(|| mgr.download_updates(&info, Some(tx)));
            for p in rx {
                progress(p.clamp(0, 100) as u8);
            }
            h.join()
                .map_err(|_| "A thread de download falhou.".into())
                .and_then(|r| r.map_err(err))
        })
    }

    fn apply_and_restart(&self) -> Res<()> {
        let mgr = self.mgr()?;
        let info = lk(&self.found)
            .clone()
            .ok_or("Nenhuma atualização baixada para aplicar.")?;
        // Em caso de sucesso o processo termina aqui; só volta se falhar.
        mgr.apply_updates_and_restart(&info).map_err(err)
    }
}
