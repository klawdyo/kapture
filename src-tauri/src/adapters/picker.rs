//! Adaptador `Picker`: abre uma janela de seleção POR MONITOR (`region-selector-0`, `-1`...) e
//! espera a resposta que chega por `submit_region` através do canal puro. A resposta de
//! qualquer monitor encerra e destrói as janelas de todos.

use crate::logging;
use cap_core::domain::{is_sel, sel_label, sel_url, Region};
use cap_core::media::PickChan;
use cap_core::ports::Picker;
use cap_core::{CapErr, Res};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, WindowEvent,
};

/// Mensagem base quando o seletor não abre (a causa real vai para o log e para o fim do texto).
const ERR: &str = "Não foi possível abrir o seletor de área";

/// Seletor de região em tela cheia, um por monitor.
pub struct TauriPicker {
    app: AppHandle,
    chan: Arc<PickChan>,
    /// Geração do seletor atual: só a geração vigente pode cancelar ou fechar janelas,
    /// assim o fechamento de um seletor antigo nunca derruba o seguinte.
    gen: Arc<AtomicU64>,
}

impl TauriPicker {
    pub fn new(app: AppHandle, chan: Arc<PickChan>) -> Self {
        Self {
            app,
            chan,
            gen: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Destrói todas as janelas do seletor e espera (até 1 s) elas sumirem.
    fn close_all(&self) {
        for (label, w) in self.app.webview_windows() {
            if is_sel(&label) {
                if let Err(e) = w.destroy() {
                    logging::get().warn(&format!("seletor de área: falha ao fechar {label}: {e}"));
                }
            }
        }
        for _ in 0..50 {
            if !self.app.webview_windows().keys().any(|l| is_sel(l)) {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        logging::get().warn("seletor de área: janela antiga ainda existe após 1 s");
    }

    /// Cria a janela transparente, sem sombra e sem barra de tarefas que cobre o monitor `i`.
    fn open(&self, i: usize, m: &tauri::Monitor, g: u64) -> tauri::Result<WebviewWindow> {
        let (p, s) = (*m.position(), *m.size());
        let url = WebviewUrl::App(PathBuf::from(sel_url(p.x, p.y, m.scale_factor())));
        let win = WebviewWindowBuilder::new(&self.app, sel_label(i), url)
            .title("Selecionar área")
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .drag_and_drop(false)
            .visible(false)
            .build()?;
        // Posição e tamanho em pixels FÍSICOS: vão para o monitor certo mesmo com escalas diferentes.
        // Posiciona, dimensiona e reposiciona: a mudança de monitor pode reescalar a janela.
        win.set_position(PhysicalPosition::new(p.x, p.y))?;
        win.set_size(PhysicalSize::new(s.width, s.height))?;
        win.set_position(PhysicalPosition::new(p.x, p.y))?;
        // Fechar uma janela (Alt+F4) sem responder equivale a cancelar.
        let (chan, gen) = (self.chan.clone(), self.gen.clone());
        win.on_window_event(move |e| {
            if matches!(e, WindowEvent::Destroyed) && gen.load(Ordering::SeqCst) == g {
                chan.submit(None);
            }
        });
        win.show()?;
        Ok(win)
    }
}

impl Picker for TauriPicker {
    /// Bloqueia até a resposta; chame fora da thread principal.
    fn pick(&self) -> Res<Option<Region>> {
        // Nova geração (desarma os avisos das janelas antigas) e limpa qualquer janela anterior.
        let g = self.gen.fetch_add(1, Ordering::SeqCst) + 1;
        self.close_all();
        let mons = self.app.available_monitors().map_err(|e| {
            logging::get().error(&format!("seletor de área: falha ao listar monitores: {e}"));
            CapErr::from(format!("{ERR}: não foi possível listar os monitores."))
        })?;
        if mons.is_empty() {
            logging::get().error("seletor de área: nenhum monitor encontrado");
            return Err(CapErr::from(format!("{ERR}: nenhum monitor encontrado.")));
        }
        let rx = self.chan.begin();
        for (i, m) in mons.iter().enumerate() {
            if let Err(e) = self.open(i, m, g) {
                logging::get().error(&format!(
                    "seletor de área: falha ao criar a janela {} (monitor {i}): {e}",
                    sel_label(i)
                ));
                self.finish(g);
                return Err(CapErr::from(format!("{ERR}: {e}")));
            }
        }
        logging::get().info(&format!(
            "seletor de área: {} janela(s) criada(s)",
            mons.len()
        ));
        let r = rx.recv().map_err(|_| {
            // Outro seletor assumiu o canal: não fecha as janelas dele.
            logging::get().error("seletor de área: canal de resposta encerrado");
            CapErr::from(format!("{ERR}: o pedido foi substituído por outro."))
        });
        self.finish(g);
        logging::get().info("seletor de área: janelas fechadas");
        r
    }
}

impl TauriPicker {
    /// Encerra o seletor da geração `g` destruindo todas as janelas; se outra geração já
    /// assumiu, não mexe nas janelas dela.
    fn finish(&self, g: u64) {
        if self
            .gen
            .compare_exchange(g, g + 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.close_all();
        }
    }
}
