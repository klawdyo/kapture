# Contrato de arquitetura do Cap (fonte única de verdade)

## Princípios
- Comentários e textos de UI em português; identificadores em inglês, nomes curtos (`price`, não `salePrice`).
- Clean Architecture: domínio e casos de uso não conhecem Tauri nem Windows.
- Toda capacidade externa é uma PORTA (trait/interface) com implementação REAL e FAKE.
- Meta: menor instalador possível. Sem FFmpeg: encoder = Media Foundation (H.264+AAC -> MP4).
  A webcam entra no vídeo por captura direta da janela dela (sem overlay filter).

## Layout
```
Cargo.toml                  # workspace: crates/cap-core, src-tauri
crates/cap-core/            # Rust puro: domain, app (use cases), ports, fakes, log/logfile. 100% cobertura.
src-tauri/                  # adaptadores Windows (cfg(windows)) + comandos Tauri finos
src/                        # Vue 3 + TS: ui/ (widgets genéricos), domain/, ports/, windows/, composables/
packaging/msix/             # AppxManifest + script MakeAppx
.github/workflows/          # ci.yml (testes) e release.yml (MSI + MSIX)
```

## Tipos (JSON camelCase)
- Region `{x,y,w,h}` (inteiros, pixels físicos)
- Mode `"screen"|"window"|"area"`
- Monitor `{id,name,w,h,x,y,primary}`; Win `{id,title}`; Targets `{monitors:Monitor[],wins:Win[]}`
- Dev `{id,name}`; Devices `{cams:Dev[],mics:Dev[]}`
- RecCfg `{mode, targetId?:number, region?:Region, micId?:string, sys:boolean, fps:number, dir?:string}`
- RecState `{state:"idle"|"rec"|"paused", secs:number}`

## Comandos IPC (invoke)
| comando | args | retorno |
|---|---|---|
| list_targets | - | Targets |
| list_devices | - | Devices |
| toggle_cam | `{visible:boolean, devId?:string}` | void |
| pick_region | - | Region \| null (abre janela `region-selector`) |
| submit_region | `{region: Region \| null}` | void (chamado pelo seletor) |
| start_rec | `{cfg: RecCfg}` | void |
| pause_rec / resume_rec | - | void |
| stop_rec | - | string (caminho do MP4) |
| get_state | - | RecState |
| mic_start | `{micId:string}` | void (liga o VU meter; emite `mic-level`) |
| mic_stop | - | void (desliga o VU meter e zera o nível) |
| log_ui | `{lvl:"debug"\|"info"\|"warn"\|"error", msg:string}` | void (grava `[ui] msg` no log em disco; `lvl` desconhecido vira `info`) |
| open_logs | - | void (abre a pasta de logs no Explorer; erro em português se não abrir) |
| fit_toolbar | w,h (px lógicos) | void (redimensiona a janela `toolbar` mantendo centro e base; `Backend.fitBar`) |
Erros: string em português.

Regras do VU meter (`MicMeter`, testadas em cap-core): no máximo um medidor ativo.
`mic_start` com o mesmo microfone já medido é no-op; com outro, reinicia sem erro.
`start_rec` reaproveita o medidor ligado (e o inicia se não houver); `stop_rec` NÃO o desliga:
quem liga/desliga é a barra (ao escolher/alterar o microfone, 'nenhum' ou ao fechar).

Câmera: o `id` de `Dev` da câmera é o nome amigável; o deviceId do getUserMedia (hash no WebView2)
é resolvido no frontend casando o `label` de `enumerateDevices()` com esse nome (domain/cam.ts).

## Logs em disco
Porta `Log` (`ports.rs`: `write(lvl, msg)` mais `debug/info/warn/error`), nunca falha nem bloqueia.
- REAL: `cap_core::logfile::FileLog` (só `std`, sem `log`/`tracing`), grava e dá `flush` a cada linha em
  `%LOCALAPPDATA%\Cap\logs\cap.log` (SHGetKnownFolderPath, depois a env `LOCALAPPDATA`, depois a pasta temporária).
- FAKE: `fakes::FakeLog` (guarda `(Lvl, String)`; `msgs()`, `has()`).
- Lógica pura em `cap_core::log` (100% de cobertura): linha `2026-01-02T03:04:05.678Z INFO  mensagem` (UTC, nível com 5
  colunas, quebras de linha viram continuação indentada), mensagens truncadas em 4000 bytes, rotação por tamanho
  (`cap.log` passa a `cap.1.log` ao passar de 512 KB, no máximo 3 arquivos: `cap.log`, `cap.1.log`, `cap.2.log`),
  HRESULT em hexadecimal (`0x80070005`), texto do pânico e linhas de IPC/UI.
- Quem registra: os casos de uso recebem a porta por injeção (`Recorder`: início/pausa/retomada/fim com a cfg, sem o id do
  microfone; `Lister`: contagens e nomes; `CamCtl`, `RegionPick`, `MicMeter`) e o decorador `LoggedEnc` envolve o
  `Encoder` (abertura, resolução/fps/bitrate no 1º quadro, progresso a cada 5 s, falhas sem repetição). Cada comando IPC
  grava `ipc <nome>: ok|erro (N ms)`. Os adaptadores Windows gravam HRESULT e contexto por um acesso global
  (`src-tauri/src/logging.rs`), pois vivem em callbacks sem injeção. O `panic hook` (o release usa `panic=abort`) grava
  mensagem, `arquivo:linha` e backtrace antes de abortar.
- Frontend: `window` `error`, `unhandledrejection`, erros de componentes Vue (`src/boot.ts`), falhas de `invoke`
  (`ports/real.ts`) e falhas da câmera chegam ao backend por `log_ui` (porta `Backend.logUi`, real e fake). No máximo
  100 mensagens por janela.
- Minimizar e fechar usam a API de janela (`Backend.minimize`/`close`, permissões `core:window:allow-minimize`/`allow-close`); fechar a `toolbar` encerra o app.
- A toolbar tem, na linha extra (engrenagem), o botão "Abrir logs" (`Backend.openLogs` -> `open_logs`).

## Eventos (emit)
`mic-level` `{level:number 0..1}` | `rec-state` RecState | `hotkey` (Ctrl+Shift+R alterna gravação)

## Janelas
`toolbar` (painel 720x136 px, cantos arredondados, sem moldura, transparente com `windowEffects` mica e fundo semi-opaco por CSS como fallback, centralizada na base da área de trabalho; cresce +48 px com a linha extra de alvo/configurações e vira pílula 260x56 gravando, via `fit_toolbar`), `webcam` (quadrada 1:1, transparente, topo, arrastável/redimensionável), `region-selector` (fullscreen, sob demanda). Entradas Vite: `index.html` (toolbar), `webcam.html`, `region.html`.
