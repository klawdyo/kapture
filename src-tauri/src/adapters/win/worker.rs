//! Thread de trabalho com inicialização síncrona (os objetos COM/WinRT
//! nascem e morrem dentro da própria thread, então não precisam ser `Send`).

use super::com::init;
use cap_core::{CapErr, Res};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Laço periódico numa thread própria.
pub struct Worker {
    stop: Arc<AtomicBool>,
    h: Option<JoinHandle<()>>,
}

impl Worker {
    /// Roda `init` na nova thread; se der certo, chama `step` a cada
    /// `every` (com correção de atraso) até `stop()`. Só retorna depois
    /// de o `init` terminar, devolvendo o erro dele se houver.
    pub fn start<L, I, S>(every: Duration, init: I, mut step: S) -> Res<Worker>
    where
        L: 'static,
        I: FnOnce() -> Res<L> + Send + 'static,
        S: FnMut(&mut L) + Send + 'static,
    {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let (tx, rx) = channel::<Res<()>>();
        let h = std::thread::spawn(move || {
            init_com();
            let mut st = match init() {
                Ok(s) => {
                    let _ = tx.send(Ok(()));
                    s
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                    return;
                }
            };
            let mut next = Instant::now();
            while !flag.load(Ordering::Relaxed) {
                step(&mut st);
                next += every;
                let now = Instant::now();
                if next > now {
                    std::thread::sleep(next - now);
                } else {
                    // Atrasou: não tenta recuperar em rajada.
                    next = now;
                }
            }
            // `st` é descartado aqui, na própria thread.
        });
        match rx.recv() {
            Ok(Ok(())) => Ok(Worker { stop, h: Some(h) }),
            Ok(Err(e)) => {
                let _ = h.join();
                Err(e)
            }
            Err(_) => Err(CapErr::from("Falha interna ao iniciar a captura.")),
        }
    }

    /// Pede o fim e espera a thread terminar.
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.h.take() {
            let _ = h.join();
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
    }
}

fn init_com() {
    init();
}
