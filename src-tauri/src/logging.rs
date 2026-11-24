//! Log global do app: arquivo em `%LOCALAPPDATA%\\kapture\\logs\\kapture.log`.
//!
//! Os casos de uso recebem a porta `Log` por injeção; os adaptadores de
//! baixo nível (callbacks do Windows, thread do atalho) e o panic hook
//! não têm como receber dependências, então usam este acesso global.

use cap_core::log::{log_path_line, panic_text, payload_str, start_line};
use cap_core::logfile::FileLog;
use cap_core::ports::Log;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

/// Log do processo, a pasta realmente usada e as notas de escolha da pasta.
struct Global {
    log: Arc<dyn Log>,
    dir: PathBuf,
    notes: Vec<String>,
}

static LOG: OnceLock<Global> = OnceLock::new();

/// Pastas candidatas em ordem de preferência. Sem COM aqui: este código roda dentro da criação do
/// log, e uma chamada de COM que falhasse tentaria registrar no log ainda inexistente (deadlock).
/// `LOCALAPPDATA` vem sempre definida em processos de usuário; a pasta temporária é a reserva.
fn candidates() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let base = std::env::var("LOCALAPPDATA").ok();
        let tmp = std::env::temp_dir();
        cap_core::log::logs_dirs(base.as_deref(), &tmp.to_string_lossy())
            .into_iter()
            .map(PathBuf::from)
            .collect()
    }
    // Outros SOs (só desenvolvimento da interface).
    #[cfg(not(windows))]
    {
        vec![std::env::temp_dir().join("kapture").join("logs")]
    }
}

fn global() -> &'static Global {
    LOG.get_or_init(|| {
        let (log, notes) = FileLog::first_usable(&candidates());
        Global {
            dir: log.dir().to_path_buf(),
            log: Arc::new(log),
            notes,
        }
    })
}

/// Pasta de logs REALMENTE usada (a preferida ou, se ela falhou, a reserva).
pub fn dir() -> PathBuf {
    global().dir.clone()
}

/// Log do processo (criado na primeira chamada).
pub fn get() -> Arc<dyn Log> {
    global().log.clone()
}

/// Instala o panic hook. O release usa `panic = "abort"`, então a
/// mensagem, o local e o backtrace são gravados antes de o processo abortar.
pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let loc = info.location().map(|l| (l.file(), l.line()));
        let th = std::thread::current();
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        let text = panic_text(
            &payload_str(info.payload()),
            loc,
            th.name().unwrap_or("sem nome"),
            &bt,
        );
        get().error(&text);
        // Sem isto o app sumia em silêncio: avisa o usuário e diz onde está o log.
        #[cfg(windows)]
        crate::adapters::win::alert(
            "kapture",
            &format!(
                "O kapture encontrou um erro e vai fechar.\n\nO detalhe foi salvo em:\n{}",
                dir().join("kapture.log").display()
            ),
        );
    }));
}

/// Primeira linha da sessão: versão, Windows e argumentos.
pub fn log_start() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let os = os_version();
    get().info(&start_line(env!("CARGO_PKG_VERSION"), &os, &args));
    // Caminho completo do arquivo e, se a pasta preferida falhou, o motivo da troca.
    get().info(&log_path_line(&dir().to_string_lossy()));
    for n in &global().notes {
        get().warn(n);
    }
}

/// Versão do Windows (`maior.menor.build`).
fn os_version() -> String {
    #[cfg(windows)]
    {
        crate::adapters::win::os_version()
    }
    #[cfg(not(windows))]
    {
        "n/d (não é Windows)".to_string()
    }
}
