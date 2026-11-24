//! Adaptador `Cam`: cria/oculta a janela da webcam (a câmera é aberta
//! pela própria página via getUserMedia; aqui só passamos `?dev=`).

use crate::lk;
use crate::logging;
use cap_core::media::cam_page;
use cap_core::ports::Cam;
use cap_core::{CapErr, Res};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};

const LABEL: &str = "webcam";
const ERR: &str = "Não foi possível abrir a janela da webcam.";

/// Registra o erro do Tauri e devolve a mensagem para o usuário.
fn bad<E: std::fmt::Display>(ctx: &'static str) -> impl FnOnce(E) -> CapErr {
    move |e| {
        logging::get().error(&format!("janela da webcam: {ctx}: {e}"));
        CapErr::from(ERR)
    }
}

/// Controla a janela `webcam`.
pub struct TauriCam {
    app: AppHandle,
    /// URL base (origem do app) descoberta na primeira criação.
    base: Mutex<Option<Url>>,
}

impl TauriCam {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            base: Mutex::new(None),
        }
    }

    /// Cria a janela a partir da configuração (`create: false` no conf),
    /// apontando direto para a página com o `?dev=` desejado.
    fn build(&self, dev_id: Option<&str>) -> Res<()> {
        let mut cfg = self
            .app
            .config()
            .app
            .windows
            .iter()
            .find(|w| w.label == LABEL)
            .cloned()
            .ok_or_else(|| {
                logging::get().error("janela da webcam: configuração não encontrada");
                ERR
            })?;
        cfg.url = WebviewUrl::App(cam_page(dev_id).into());
        cfg.visible = true;
        let w = WebviewWindowBuilder::from_config(&self.app, &cfg)
            .and_then(|b| b.build())
            .map_err(bad("criar"))?;
        *lk(&self.base) = w.url().ok();
        logging::get().info(&format!(
            "janela da webcam: criada (url {:?})",
            w.url().ok()
        ));
        Ok(())
    }
}

impl Cam for TauriCam {
    fn show(&self, dev_id: Option<&str>) -> Res<()> {
        let Some(w) = self.app.get_webview_window(LABEL) else {
            return self.build(dev_id);
        };
        // Janela já existe (estava oculta): navega para a câmera pedida.
        let base = lk(&self.base).clone().ok_or_else(|| {
            logging::get().error("janela da webcam: URL base desconhecida");
            ERR
        })?;
        let mut u = base;
        u.set_path("/webcam.html");
        u.set_query(cam_page(dev_id).split_once('?').map(|(_, q)| q));
        w.navigate(u).map_err(bad("navegar"))?;
        logging::get().info("janela da webcam: reexibida");
        w.show().map_err(bad("mostrar"))
    }

    fn hide(&self) -> Res<()> {
        if let Some(w) = self.app.get_webview_window(LABEL) {
            // Página em branco libera a câmera (some a luz indicadora).
            if let Ok(u) = Url::parse("about:blank") {
                let _ = w.navigate(u);
            }
            w.hide().map_err(bad("ocultar"))?;
            logging::get().info("janela da webcam: ocultada");
        }
        Ok(())
    }
}
