# Memória de problemas do kapture

Registro do que já deu errado, para ninguém repetir. Leia antes de mexer em COM, janelas, áudio, log, CI ou release.

## Regra

**Todo problema novo é registrado aqui no mesmo commit da correção, com teste automatizado** que impeça a regressão (caminho do arquivo e nome do teste). Quando não dá para testar em código (diagnóstico, processo, CI), registre a causa e diga por quê.

Os guards estáticos (leitura de arquivos do repositório, rodam no Linux do CI) ficam em `crates/cap-core/tests/guards.rs`. As mensagens de falha apontam o ID daqui.

## Modelo de entrada

```md
## Pnn Título curto
- **Sintoma:** o que o usuário ou o log mostrou.
- **Causa raiz:** por que aconteceu.
- **Correção:** o que mudou e onde (arquivo, versão).
- **Teste:** `caminho/do/arquivo`, nome do teste (ou "sem teste de código" e o motivo).
```

## Problemas

<a id="p01"></a>
## P01 App não abria (panic do tao)
- **Sintoma:** panic `OleInitialize failed RPC_E_CHANGED_MODE` ao criar a janela.
- **Causa raiz:** o log chamava `local_app_data()` (`SHGetKnownFolderPath`), que faz `com::init()` em MTA na thread principal antes do STA do tao.
- **Correção (v0.6.3):** `init_main_sta()` é a PRIMEIRA coisa de `run()` em `src-tauri/src/lib.rs`; `logging::dir()` usa `std::env` `LOCALAPPDATA`, sem COM. Nada de COM MTA na thread principal.
- **Teste:** `crates/cap-core/tests/guards.rs`, `sta_first` (ordem em `run()`) e `log_no_com` (sem `local_app_data` nem `com::init` em `logging.rs`).

<a id="p02"></a>
## P02 Borda preta nos cantos arredondados da barra
- **Sintoma:** contorno preto nos cantos arredondados da barra.
- **Causa raiz:** sombra e mica em janela transparente.
- **Correção:** `shadow: false` e sem `windowEffects` na janela `toolbar` em `src-tauri/tauri.conf.json`.
- **Teste:** `crates/cap-core/tests/guards.rs`, `toolbar_flat`.

<a id="p03"></a>
## P03 Print Screen não capturava a barra
- **Sintoma:** a barra não aparecia em capturas de tela comuns.
- **Causa raiz:** `set_content_protected` ficava sempre ligado.
- **Correção:** só durante gravação ou pausa, pela função pura `cap_core::domain::protected(St)`.
- **Teste:** `crates/cap-core/src/domain/rules.rs`, `barra_fora_da_captura_so_gravando` (cobre Idle, Rec e Paused).

<a id="p04"></a>
## P04 Seletor de área só no 1 monitor e janela duplicada
- **Sintoma:** seletor só no primeiro monitor e erro `a webview with label region-selector already exists`.
- **Causa raiz:** janela fixa no `tauri.conf.json` e criação repetida sem destruir a anterior.
- **Correção:** uma janela por monitor, `region-selector-N`, criada por código; as antigas são destruídas antes de criar; janela fixa removida do `tauri.conf.json`; capability `region-selector-*`.
- **Teste:** `crates/cap-core/tests/guards.rs`, `no_fixed_selector` e `selector_capability`; rótulos em `crates/cap-core/src/domain/rules.rs`, `janelas_do_seletor`.

<a id="p05"></a>
## P05 Seletor sem confirmação
- **Sintoma:** o seletor de área não pedia confirmação da região escolhida.
- **Causa raiz:** faltavam os botões de decisão.
- **Correção:** botões "Gravar nesta área" e "Cancelar" em `src/windows/RegionSelector.vue`.
- **Teste:** `src/windows/windows.test.ts`, `RegionSelector: botões` > `Gravar nesta área confirma e Cancelar devolve null`.

<a id="p06"></a>
## P06 Áudio falhava com E_INVALIDARG
- **Sintoma:** HRESULT `0x80070057` ao abrir microfone ou loopback.
- **Causa raiz:** `AUTOCONVERTPCM` junto de `LOOPBACK` e formato fixo.
- **Correção:** formato nativo via `GetMixFormat`; microfone sem flags; loopback só com `LOOPBACK`; conversão por software em `cap-core` (`parse_fmt`, `Conv`, `to_i16`); segue sem a fonte que falhar, devolvendo avisos.
- **Teste:** `crates/cap-core/src/media/audio.rs`, `interpreta_formatos`, `converte_cada_tipo_de_amostra`, `mixa_canais`, `reamostra_sem_emenda`, `silencio_e_sobra_parcial`, `saturacao_e_textos`; guard `audio_no_autoconvert` em `crates/cap-core/tests/guards.rs` (proíbe `AUTOCONVERTPCM` em `src-tauri/src/adapters/win/capture/audio.rs`).

<a id="p07"></a>
## P07 Logs mudos ou pouco úteis
- **Sintoma:** falhas sem rastro no log.
- **Causa raiz:** pasta de log única, sem caminho visível, sem erros de IPC e do frontend, panic sem aviso.
- **Mitigação:** pasta de reserva em temp verificada, caminho do log na 2ª linha, erros de IPC e do frontend (`[ui]`) no log, panic hook com caixa de erro.
- **Teste:** `crates/cap-core/src/logfile.rs`, `escolhe_a_primeira_pasta_utilizavel`, `falha_ao_abrir_nao_derruba`; `crates/cap-core/src/log.rs`, `pastas_de_log_com_reserva`, `textos_auxiliares`, `ipc_so_erro`, `payload_de_panico`, `ipc_e_ui`.

<a id="p08"></a>
## P08 Procedimento de diagnóstico: log de binário velho
- **Sintoma:** em 2026-11-24 o log mostrou `kapture 0.5.0` com o panic do P01, depois de a v0.6.3 já funcionar.
- **Causa raiz:** o usuário rodava o exe antigo. A versão é carimbada no release por `version.mjs apply`; o `Cargo.toml` fica em 0.1.0.
- **Lição:** ao receber log de "não abre", confira primeiro a versão (1ª linha do log) e a presença da linha "COM da thread principal em modo STA" antes de supor regressão.
- **Teste:** sem teste de código, é procedimento de diagnóstico.

<a id="p09"></a>
## P09 Processo: dependências versionadas, PR cedo, versão de cabeça
- **node_modules versionado por engano.** Guard: `crates/cap-core/tests/guards.rs`, `no_tracked_deps` (falha se `git ls-files` listar `node_modules/` ou `target/`).
- **PR aberto antes de enviar tudo** gerou várias execuções de Actions. Regra de processo, sem teste: só envie algo pronto e verificado; PR só quando o dono mandar (CLAUDE.md).
- **Versão calculada de cabeça** em vez de `node scripts/release-pr.mjs`. Coberto por `scripts/version.test.mjs` (`nextVersion`, `versões e tags`, `generateNotes`).

<a id="p10"></a>
## P10 CI: Rust novo e Pages
- **Clippy no Rust 1.100 do runner:** `chunks_exact` virou `as_chunks`. Use o Rust stable atual localmente.
- **Cobertura de linha** conta o `}` de `if` sem `else`: adicione testes de ramo.
- **Pages:** precisa estar habilitado com fonte GitHub Actions, e o deploy fica em job separado do build.
- **Teste:** sem teste de código (configuração do GitHub e do runner); a causa fica registrada aqui.

## Não validado em Windows real

Captura, microfone, webcam, gravação em MP4, mica, sombra, `fit_toolbar`, atalho global, ícone embutido no exe sem bundle, CSP e CORS da verificação de atualização. A lista oficial está em [CLAUDE.md](../../CLAUDE.md), seção "O que nunca foi validado em Windows real". Não afirme que funcionam sem teste numa máquina Windows.
