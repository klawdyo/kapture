//! Guards de regressão: leitura estática de arquivos do repositório.
//!
//! Cada teste protege um problema já ocorrido e registrado em `docs/specs/MEMORY.md`.
//! Rodam no Linux do CI, sem Windows: só leem texto e JSON. Não entram na cobertura do `src`.

use std::path::PathBuf;
use std::process::Command;

/// Raiz do repositório (`crates/cap-core/../..`).
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Lê um arquivo do repositório pelo caminho relativo à raiz.
fn read(rel: &str) -> String {
    let p = root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("não consegui ler {rel}: {e}"))
}

/// Remove comentários de linha (`//`), para checar só o código.
fn code(src: &str) -> String {
    src.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Corpo de `pub fn run()` em `src-tauri/src/lib.rs`, sem comentários.
fn run_body() -> String {
    let src = code(&read("src-tauri/src/lib.rs"));
    let at = src.find("pub fn run()").expect("lib.rs sem pub fn run()");
    src[at..].to_string()
}

/// P01: o STA da thread principal vem antes do log e do panic hook.
#[test]
fn sta_first() {
    let body = run_body();
    let sta = body.find("init_main_sta").expect("run() sem init_main_sta");
    for later in ["install_panic_hook", "log_start"] {
        let pos = body
            .find(later)
            .unwrap_or_else(|| panic!("run() sem {later}"));
        assert!(
            sta < pos,
            "P01: init_main_sta deve vir antes de {later} em run() (COM MTA antes do STA do tao \
             derruba o app com RPC_E_CHANGED_MODE). Veja docs/specs/MEMORY.md#p01"
        );
    }
}

/// P01: o caminho da pasta de log não pode usar COM.
#[test]
fn log_no_com() {
    let src = code(&read("src-tauri/src/logging.rs"));
    for bad in ["local_app_data", "com::init"] {
        assert!(
            !src.contains(bad),
            "P01: logging.rs não pode usar {bad} (inicia COM em MTA na thread principal). \
             Use std::env LOCALAPPDATA. Veja docs/specs/MEMORY.md#p01"
        );
    }
}

/// Janelas declaradas em `tauri.conf.json`.
fn windows() -> Vec<serde_json::Value> {
    let v: serde_json::Value =
        serde_json::from_str(&read("src-tauri/tauri.conf.json")).expect("tauri.conf.json inválido");
    v["app"]["windows"]
        .as_array()
        .expect("sem app.windows")
        .clone()
}

/// P02: a janela da barra não tem sombra nem efeito mica.
#[test]
fn toolbar_flat() {
    let w = windows()
        .into_iter()
        .find(|w| w["label"] == "toolbar")
        .expect("sem janela toolbar");
    assert_eq!(
        w["shadow"],
        serde_json::Value::Bool(false),
        "P02: a janela toolbar precisa de shadow:false (borda preta nos cantos arredondados). \
         Veja docs/specs/MEMORY.md#p02"
    );
    assert!(
        w.get("windowEffects").is_none(),
        "P02: a janela toolbar não pode ter windowEffects (mica em janela transparente). \
         Veja docs/specs/MEMORY.md#p02"
    );
}

/// P04: nenhuma janela fixa do seletor no tauri.conf (são criadas por código, uma por monitor).
#[test]
fn no_fixed_selector() {
    for w in windows() {
        let label = w["label"].as_str().unwrap_or("");
        assert!(
            !label.starts_with("region-selector"),
            "P04: o tauri.conf não pode declarar a janela '{label}': o seletor é criado por \
             código, uma por monitor. Veja docs/specs/MEMORY.md#p04"
        );
    }
}

/// P04: a capability cobre todas as janelas `region-selector-N`.
#[test]
fn selector_capability() {
    let v: serde_json::Value = serde_json::from_str(&read("src-tauri/capabilities/default.json"))
        .expect("capability inválida");
    let ok = v["windows"]
        .as_array()
        .expect("capability sem windows")
        .iter()
        .any(|w| w == "region-selector-*");
    assert!(
        ok,
        "P04: a capability precisa cobrir \"region-selector-*\". Veja docs/specs/MEMORY.md#p04"
    );
}

/// P06: o áudio não usa flags que causam E_INVALIDARG.
#[test]
fn audio_no_autoconvert() {
    let src = code(&read("src-tauri/src/adapters/win/capture/audio.rs"));
    for bad in ["AUTOCONVERTPCM", "SRC_DEFAULT_QUALITY"] {
        assert!(
            !src.contains(bad),
            "P06: audio.rs não pode usar {bad} (HRESULT 0x80070057 com LOOPBACK). Use o formato \
             nativo e converta em software. Veja docs/specs/MEMORY.md#p06"
        );
    }
}

/// P09: node_modules/ e target/ nunca são versionados.
#[test]
fn no_tracked_deps() {
    let out = match Command::new("git")
        .arg("-C")
        .arg(root())
        .args(["ls-files"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        // Sem git ou fora de um clone: nada a verificar.
        _ => return,
    };
    let list = String::from_utf8_lossy(&out.stdout);
    let bad: Vec<&str> = list
        .lines()
        .filter(|f| {
            f.starts_with("node_modules/")
                || f.starts_with("target/")
                || f.contains("/node_modules/")
                || f.contains("/target/")
        })
        .take(5)
        .collect();
    assert!(
        bad.is_empty(),
        "P09: arquivos de dependência/build versionados: {bad:?}. Remova do índice e use o \
         .gitignore. Veja docs/specs/MEMORY.md#p09"
    );
}
