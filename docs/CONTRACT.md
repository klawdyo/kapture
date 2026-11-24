# Contrato de arquitetura do kapture (fonte única de verdade)

> **Visual:** a fonte de verdade do design atual está em [`docs/design/`](design/README.md). O PDF original e `docs/ui-reference.png` são históricos e não devem ser usados como modelo de interface.


## Princípios
- Comentários e textos de UI em português; identificadores em inglês, nomes curtos (`price`, não `salePrice`).
- Clean Architecture: domínio e casos de uso não conhecem Tauri nem Windows.
- Toda capacidade externa é uma PORTA (trait/interface) com implementação REAL e FAKE.
- Meta: menor executável possível (distribuição só pelo `kapture.exe` solto). Sem FFmpeg: encoder = Media Foundation (H.264+AAC -> MP4).
  A webcam entra no vídeo por captura direta da janela dela (sem overlay filter).

## Layout
```
Cargo.toml                  # workspace: crates/cap-core, src-tauri
crates/cap-core/            # Rust puro: domain, app (use cases), ports, fakes, log/logfile. 100% cobertura.
src-tauri/                  # adaptadores Windows (cfg(windows)) + comandos Tauri finos
src/                        # Vue 3 + TS: ui/ (widgets genéricos), domain/, ports/, windows/, composables/
packaging/msix/             # AppxManifest + script MakeAppx
.github/workflows/          # ci.yml (testes) e release.yml (exe solto; MSIX desligado)
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
| start_rec | `{cfg: RecCfg}` | string (aviso em português, vazio se tudo certo; ex.: "Não foi possível usar o microfone; a gravação segue sem ele.": a fonte de áudio que não abriu é dispensada e o vídeo continua; a interface mostra o aviso na barra) |
| pause_rec / resume_rec | - | void |
| stop_rec | - | string (caminho do MP4) |
| get_state | - | RecState |
| mic_start | `{micId:string}` | void (liga o VU meter; emite `mic-level`) |
| mic_stop | - | void (desliga o VU meter e zera o nível) |
| log_ui | `{lvl:"debug"\|"info"\|"warn"\|"error", msg:string}` | void (grava `[ui] msg` no log em disco; `lvl` desconhecido vira `info`) |
| open_logs | - | void (abre a pasta de logs no Explorer; erro em português se não abrir) |
| open_url | `{url:string}` | void (abre no navegador padrão via ShellExecuteW; SÓ aceita `https://github.com/klawdyo/kapture/releases/*` sem espaços, aspas, `\` nem `..`; outra URL devolve "Endereço não permitido."; porta `Opener`, caso de uso `UrlOpen`) |
| update_start | - | void (retorna na hora; inicia numa thread o ciclo de atualização, UMA vez por execução: porta `Updater`, caso de uso `Update`; o progresso chega pelo evento `update-state`; chamadas seguintes só reemitem o estado atual) |
| update_apply | - | void (só com o estado `ready`; encerra o app, aplica a versão baixada e reabre; sem versão pronta devolve "Nenhuma atualização pronta para aplicar.") |
| fit_toolbar | w,h (px lógicos) | void (redimensiona a janela `toolbar` mantendo centro e base; `Backend.fitBar`) |
Erros: string em português.

Áudio (WASAPI, `adapters/win/capture/audio.rs`): cada fonte (microfone e som do sistema por loopback) é aberta em modo
compartilhado no formato NATIVO do dispositivo (`GetMixFormat`; o loopback só leva `AUDCLNT_STREAMFLAGS_LOOPBACK`, o
microfone nenhuma flag: `AUTOCONVERTPCM` com `LOOPBACK` dava `E_INVALIDARG`). A conversão para i16 estéreo 48 kHz é feita
em software por `cap_core::media::Conv` (PCM 16/24/32 e float 32, qualquer número de canais e taxa, reamostragem linear,
saturação). Uma fonte que falha ao abrir não derruba a gravação: o erro (etapa, nome do dispositivo, formato e HRESULT em
hexadecimal) vai ao log e `Capture::start` devolve um aviso para o usuário.

Regras do VU meter (`MicMeter`, testadas em cap-core): no máximo um medidor ativo.
`mic_start` com o mesmo microfone já medido é no-op; com outro, reinicia sem erro.
`start_rec` reaproveita o medidor ligado (e o inicia se não houver); `stop_rec` NÃO o desliga:
quem liga/desliga é a barra (ao escolher/alterar o microfone, 'nenhum' ou ao fechar).

Câmera: o `id` de `Dev` da câmera é o nome amigável; o deviceId do getUserMedia (hash no WebView2)
é resolvido no frontend casando o `label` de `enumerateDevices()` com esse nome (domain/cam.ts).

## Logs em disco
Porta `Log` (`ports.rs`: `write(lvl, msg)` mais `debug/info/warn/error`), nunca falha nem bloqueia.
- REAL: `cap_core::logfile::FileLog` (só `std`, sem `log`/`tracing`), grava e dá `flush` a cada linha em
  `%LOCALAPPDATA%\kapture\logs\kapture.log` (env `LOCALAPPDATA`, sem COM; `FileLog::first_usable` testa criar a pasta e abrir o
  arquivo e, se falhar, cai para `<temp>\kapture\logs`). A primeira linha da sessão traz a versão; a segunda, o caminho completo
  do arquivo (`arquivo de log: ...`); se a pasta preferida falhou, uma linha WARN diz qual e por quê. `open_logs` abre a pasta
  REALMENTE usada. Todo erro de comando IPC vai ao log com o nome do comando (`ipc <nome>: erro: ...`); `fit_toolbar` só registra falhas.
- FAKE: `fakes::FakeLog` (guarda `(Lvl, String)`; `msgs()`, `has()`).
- Lógica pura em `cap_core::log` (100% de cobertura): linha `2026-01-02T03:04:05.678Z INFO  mensagem` (UTC, nível com 5
  colunas, quebras de linha viram continuação indentada), mensagens truncadas em 4000 bytes, rotação por tamanho
  (`kapture.log` passa a `kapture.1.log` ao passar de 512 KB, no máximo 3 arquivos: `kapture.log`, `kapture.1.log`, `kapture.2.log`),
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

## Atualizações
Porta `Updater` (cap-core): `supported()`, `check() -> Option<UpdInfo{version}>`, `download(progress 0..=100)`, `apply_and_restart()`. Adaptador de produção, em todos os SOs: `NoUpdater` (`src-tauri/src/adapters/noupd.rs`, `supported=false`, demais métodos devolvem erro); `FakeUpdater` nos testes. O caso de uso `Update` (máquina de estados pura) roda o ciclo uma vez e emite `update-state`:
`{kind:'idle'|'checking'|'none'|'fallback'|'err'}`, `{kind:'downloading', pct:0..100}`, `{kind:'ready', version}`. `fallback` = sem atualização automática (`supported() = false`); `err` = falha de rede/feed/download (só vai ao log em disco como warn).

Frontend: `Backend.updateStart()` (`update_start`), `Backend.updateApply()` (`update_apply`) e `Backend.onUpdate(cb)` (`update-state`), reais e FAKE. `useUpdate.init` busca `version()`, assina `onUpdate` e chama `updateStart`
(uma vez ao montar a toolbar). Fase (`Phase`): `idle`, `checking`, `downloading`, `ready`, `available`. Textos exatos (`updText` em `domain/upd.ts`): "Verificando atualizações…", "Atualizando…" (com " 42%" quando `pct > 0`),
"Reinicie para atualizar", "Atualização disponível".
- `UpdateStatus` (components; compõe `ui/Txt`, `ui/TxtBtn`, `ui/Dot` e `UpdateDot`) fica na barra de título, antes de minimizar/fechar: texto fino cinza 11 px sem negrito, em região `role="status"` `aria-live="polite"`, sem empurrar o layout e sem atrapalhar o arrasto.
  `ready`: botão (foco visível) com bolinha vermelha piscando e o texto; o clique chama `update_apply`. Nada aparece no modo compacto de gravação.
- Sem atualização (`none`) ou erro (`err`): o texto some; `[update] ...` no log em disco.
- Fallback (`fallback`, ou se o comando falhar): `useUpdate` consulta `Updates.check(current)` (REAL `realUpdates`: `fetch` GET com timeout de 5 s a `https://api.github.com/repos/klawdyo/kapture/releases/latest`; FAKE `FakeUpdates`),
  mostra "Atualização disponível" e `UpdateDot` (compõe `ui/Dot` e `ui/Pop`) com a mensagem de versões instalada e nova e **Baixar** (`Backend.openUrl` -> `open_url`) e **Depois**. Regras puras em `domain/upd.ts`: semver X.Y.Z ignorando o prefixo `v`,
  prerelease e rascunho ignorados, escolha da mais nova, `html_url` começando em `https://github.com/klawdyo/kapture/releases/`. Bolinha estática com `prefers-reduced-motion`.
- CSP: `connect-src 'self' ipc: http://ipc.localhost https://api.github.com`. Capability `core:app:allow-version`.

## Eventos (emit)
`mic-level` `{level:number 0..1}` | `rec-state` RecState (o adaptador também liga `set_content_protected` na `toolbar` quando `protected(state)` é verdadeiro, isto é, em `rec`/`paused`, e desliga em `idle`) | `update-state` UpdState (ver Atualizações) | `hotkey` (Ctrl+Shift+R alterna gravação)

## Janelas
`toolbar` (painel 720x136 px, cantos arredondados, sem moldura, transparente, sem sombra nem `windowEffects` (o DWM desenhava uma borda preta de canto reto em volta do painel arredondado; borda fina e fundo semi-opaco são só CSS), centralizada na base da área de trabalho; cresce +48 px com a linha extra de alvo/configurações e vira pílula 150x48 gravando, via `fit_toolbar`), na pílula só há ponto indicador, cronômetro e Parar (sem botão de pausa na UI; `pause_rec`/`resume_rec` seguem no backend), semitransparente (opacity .72, 1 em hover/foco) e arrastável; `webcam` (quadrada 1:1, transparente, topo, arrastável/redimensionável), `region-selector` (fullscreen, sob demanda; o retângulo desenha, move por dentro e redimensiona pelas 8 alças; setas movem 1 px, Shift 10 px, Alt+setas redimensionam; mínimo 16x16, preso à tela. Depois de desenhar, uma faixa junto da etiqueta de tamanho (badge informativa, sem borda nem hover) traz os botões **Gravar nesta área** (confirma) e **Cancelar**; Enter confirma e Esc cancela). Entradas Vite: `index.html` (toolbar), `webcam.html`, `region.html`.
