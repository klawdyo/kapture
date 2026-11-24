//! Caso de uso `Update`: ciclo de atualização (verificar, baixar sozinho,
//! aguardar a reinicialização) como máquina de estados que publica
//! `update-state` pela porta `Events`. O download real fica no `Updater`.

use crate::domain::UpdState;
use crate::lk;
use crate::ports::{Events, Log, Updater};
use crate::Res;
use std::sync::{Arc, Mutex};

/// Estado interno: o ciclo já começou? e o estado atual.
struct Inner {
    started: bool,
    st: UpdState,
}

/// Ciclo de atualização (roda uma única vez por execução do app).
pub struct Update {
    up: Arc<dyn Updater>,
    events: Arc<dyn Events>,
    log: Arc<dyn Log>,
    inner: Mutex<Inner>,
}

impl Update {
    pub fn new(up: Arc<dyn Updater>, events: Arc<dyn Events>, log: Arc<dyn Log>) -> Self {
        Update {
            up,
            events,
            log,
            inner: Mutex::new(Inner {
                started: false,
                st: UpdState::Idle,
            }),
        }
    }

    /// Estado atual do ciclo.
    pub fn state(&self) -> UpdState {
        lk(&self.inner).st.clone()
    }

    /// Guarda e publica um novo estado (repetições idênticas são ignoradas,
    /// para não inundar a interface com o mesmo percentual).
    fn set(&self, st: UpdState) {
        {
            let mut g = lk(&self.inner);
            if g.st == st {
                return;
            }
            g.st = st.clone();
        }
        self.events.update_state(&st);
    }

    /// Executa o ciclo, bloqueando (o chamador usa uma thread). Na segunda
    /// chamada não repete nada: só reemite o estado atual, para a interface
    /// que recarregou voltar a saber onde o ciclo está.
    pub fn run(&self) {
        {
            let mut g = lk(&self.inner);
            if g.started {
                let st = g.st.clone();
                drop(g);
                self.events.update_state(&st);
                return;
            }
            g.started = true;
        }
        self.set(UpdState::Checking);
        if !self.up.supported() {
            self.log
                .info("atualização: app fora do instalador, usando o aviso simples");
            self.set(UpdState::Fallback);
            return;
        }
        let found = match self.up.check() {
            Ok(Some(i)) => i,
            Ok(None) => {
                self.log.info("atualização: já está na versão mais nova");
                self.set(UpdState::None);
                return;
            }
            Err(e) => return self.fail("ao verificar", &e.0),
        };
        self.log
            .info(&format!("atualização: baixando v{}", found.version));
        self.set(UpdState::Downloading { pct: 0 });
        if let Err(e) = self
            .up
            .download(&|p| self.set(UpdState::Downloading { pct: p.min(100) }))
        {
            return self.fail("ao baixar", &e.0);
        }
        self.log.info(&format!(
            "atualização: v{} pronta para reiniciar",
            found.version
        ));
        self.set(UpdState::Ready {
            version: found.version,
        });
    }

    /// Falha de rede/feed: sem alarde na interface, só o log em disco.
    fn fail(&self, what: &str, msg: &str) {
        self.log.warn(&format!("atualização: falha {what}: {msg}"));
        self.set(UpdState::Err);
    }

    /// Reinicia aplicando a versão baixada (só com o estado `Ready`).
    pub fn apply(&self) -> Res<()> {
        if !matches!(self.state(), UpdState::Ready { .. }) {
            return Err("Nenhuma atualização pronta para aplicar.".into());
        }
        self.log.info("atualização: reiniciando para aplicar");
        let r = self.up.apply_and_restart();
        if let Err(e) = &r {
            self.log
                .error(&format!("atualização: erro ao reiniciar: {e}"));
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::UpdInfo;
    use crate::fakes::*;

    fn mk() -> (Arc<FakeUpdater>, Arc<FakeEvents>, Arc<FakeLog>, Update) {
        let up = Arc::new(FakeUpdater::default());
        let ev = Arc::new(FakeEvents::default());
        let log = Arc::new(FakeLog::default());
        let u = Update::new(up.clone(), ev.clone(), log.clone());
        (up, ev, log, u)
    }

    fn evs(ev: &FakeEvents) -> Vec<UpdState> {
        lk(&ev.updates).clone()
    }

    #[test]
    fn baixa_sozinho_ate_ficar_pronto() {
        let (up, ev, log, u) = mk();
        *lk(&up.found) = Some(UpdInfo {
            version: "0.2.0".into(),
        });
        *lk(&up.steps) = vec![0, 40, 40, 100, 250];
        assert_eq!(u.state(), UpdState::Idle);
        u.run();
        let d = |pct| UpdState::Downloading { pct };
        assert_eq!(
            evs(&ev),
            vec![
                UpdState::Checking,
                d(0),
                d(40),
                d(100),
                UpdState::Ready {
                    version: "0.2.0".into()
                }
            ]
        );
        assert_eq!(up.calls.all(), ["check", "download"]);
        assert!(log.has("baixando v0.2.0") && log.has("v0.2.0 pronta"));
        assert_eq!(u.state(), evs(&ev).pop().unwrap());
    }

    #[test]
    fn so_roda_uma_vez_e_reemite_o_estado() {
        let (up, ev, _log, u) = mk();
        u.run();
        assert_eq!(evs(&ev), vec![UpdState::Checking, UpdState::None]);
        u.run();
        assert_eq!(up.calls.all(), ["check"]); // não consultou de novo
        assert_eq!(evs(&ev).len(), 3);
        assert_eq!(evs(&ev)[2], UpdState::None);
    }

    #[test]
    fn sem_instalador_cai_no_aviso_simples() {
        let (up, ev, log, u) = mk();
        *lk(&up.supported) = false;
        u.run();
        assert_eq!(evs(&ev), vec![UpdState::Checking, UpdState::Fallback]);
        assert!(up.calls.all().is_empty());
        assert!(log.has("fora do instalador"));
    }

    #[test]
    fn falhas_so_vao_ao_log() {
        let (up, ev, log, u) = mk();
        up.fail.set("check", "sem rede");
        u.run();
        assert_eq!(evs(&ev), vec![UpdState::Checking, UpdState::Err]);
        assert!(log.has("falha ao verificar: sem rede"));

        let (up, ev, log, u) = mk();
        *lk(&up.found) = Some(UpdInfo {
            version: "0.3.0".into(),
        });
        up.fail.set("download", "disco cheio");
        u.run();
        assert_eq!(evs(&ev).last(), Some(&UpdState::Err));
        assert!(log.has("falha ao baixar: disco cheio"));
        assert!(u.apply().is_err()); // sem Ready não reinicia
        assert!(!up.calls.all().contains(&"apply".to_string()));
    }

    #[test]
    fn aplica_so_quando_pronto() {
        let (up, _ev, log, u) = mk();
        assert_eq!(
            u.apply().unwrap_err().0,
            "Nenhuma atualização pronta para aplicar."
        );
        *lk(&up.found) = Some(UpdInfo {
            version: "1.0.0".into(),
        });
        u.run();
        u.apply().unwrap();
        assert_eq!(up.calls.all().last().unwrap(), "apply");
        assert!(log.has("reiniciando para aplicar"));
        up.fail.set("apply", "x");
        assert_eq!(u.apply().unwrap_err().0, "x");
        assert!(log.has("erro ao reiniciar: x"));
    }
}
