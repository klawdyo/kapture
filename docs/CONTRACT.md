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
crates/cap-core/            # Rust puro: domain, app (use cases), ports, fakes. 100% cobertura.
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
Erros: string em português.

## Eventos (emit)
`mic-level` `{level:number 0..1}` | `rec-state` RecState | `hotkey` (Ctrl+Shift+R alterna gravação)

## Janelas
`toolbar` (pílula, sem moldura, transparente, topo), `webcam` (quadrada 1:1, transparente, topo, arrastável/redimensionável), `region-selector` (fullscreen, sob demanda). Entradas Vite: `index.html` (toolbar), `webcam.html`, `region.html`.
