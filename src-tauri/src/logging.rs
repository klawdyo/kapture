//! Log global do app: arquivo em `%LOCALAPPDATA%\\kapture\\logs\\kapture.log`.
//!
//! Os casos de uso recebem a porta `Log` por injeção; os adaptadores de
//! baixo nível (callbacks do Windows, thread do atalho) e o panic hook
//! não têm como receber dependências, então usam este acesso global.

use cap_core::log::{panic_text, payload_str, start_line};
use cap_core::logfile::FileLog;
use cap_core::ports::Log;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

static LOG: OnceLock<Arc<dyn Log>> = OnceLock::new();

/// Pasta de logs: `%LOCALAPPDATA%\kapture\logs` (ou a pasta temporária).
pub fn dir() -> PathBuf {
    #[cfg(windows)]
    {
        use crate::adapters::win::local_app_data;
        let base = local_app_data().or_else(|| std::env::var("LOCALAPPDATA").ok());
        let tmp = std::env::temp_dir();
        PathBuf::from(cap_core::log::logs_dir(
            base.as_deref(),
            &tmp.to_string_lossy(),
        ))
    }
    // Outros SOs (só desenvolvimento da interface).
    #[cfg(not(windows))]
    {
        std::env::temp_dir().join("kapture").join("logs")
    }
}

/// Log do processo (criado na primeira chamada).
pub fn get() -> Arc<dyn Log> {
    LOG.get_or_init(|| Arc::new(FileLog::new(&dir()))).clone()
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
    get().info(&format!("pasta de logs: {}", dir().display()));
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
