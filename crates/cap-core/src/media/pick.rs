//! Canal entre o comando `submit_region` e o seletor de região.

use crate::domain::Region;
use crate::lk;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

/// Ponto de encontro: quem pede a região espera no `Receiver`; a janela
/// do seletor responde via `submit`.
#[derive(Default)]
pub struct PickChan {
    tx: Mutex<Option<Sender<Option<Region>>>>,
}

impl PickChan {
    pub fn new() -> Self {
        Self::default()
    }

    /// Começa uma nova seleção (cancela a anterior, cujo receptor
    /// recebe erro de canal fechado) e devolve onde esperar a resposta.
    pub fn begin(&self) -> Receiver<Option<Region>> {
        let (tx, rx) = channel();
        *lk(&self.tx) = Some(tx);
        rx
    }

    /// Entrega a resposta (`None` = cancelado). Devolve `false` se não
    /// havia seleção pendente.
    pub fn submit(&self, r: Option<Region>) -> bool {
        match lk(&self.tx).take() {
            Some(tx) => tx.send(r).is_ok(),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: Region = Region {
        x: 1,
        y: 2,
        w: 30,
        h: 40,
    };

    #[test]
    fn entrega_regiao_e_cancelamento() {
        let c = PickChan::new();
        let rx = c.begin();
        assert!(c.submit(Some(R)));
        assert_eq!(rx.recv().unwrap(), Some(R));
        let rx = c.begin();
        assert!(c.submit(None));
        assert_eq!(rx.recv().unwrap(), None);
    }

    #[test]
    fn sem_pedido_pendente() {
        let c = PickChan::new();
        assert!(!c.submit(Some(R)));
        // Segunda resposta à mesma seleção também é ignorada.
        let _rx = c.begin();
        assert!(c.submit(None));
        assert!(!c.submit(None));
    }

    #[test]
    fn nova_selecao_fecha_a_anterior() {
        let c = PickChan::new();
        let velho = c.begin();
        let novo = c.begin();
        assert!(velho.recv().is_err());
        assert!(c.submit(Some(R)));
        assert_eq!(novo.recv().unwrap(), Some(R));
    }

    #[test]
    fn receptor_descartado() {
        let c = PickChan::new();
        drop(c.begin());
        assert!(!c.submit(Some(R)));
    }
}
