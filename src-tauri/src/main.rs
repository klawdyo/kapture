// Sem janela de console no Windows em release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Velopack: PRIMEIRA coisa do processo. Trata os argumentos do instalador/atualizador
    // (--veloapp-install etc.) e encerra; no uso normal retorna na hora.
    #[cfg(windows)]
    velopack::VelopackApp::build().run();

    cap_lib::run();
}
