# Cap

Gravador de tela leve para Windows. Grava a tela inteira, uma janela ou uma área, com webcam em círculo, microfone e áudio do sistema, e salva em MP4.

![Logo do Cap](assets/logo-light.svg)

## Como usar

1. Abra o Cap. A barra flutuante aparece no topo da tela.
2. Escolha o modo: tela, janela ou área (a área é selecionada arrastando o mouse).
3. Opcional: ligue a webcam e escolha o microfone e o áudio do sistema.
4. Clique em gravar, ou use o atalho **Ctrl+Shift+R** para iniciar e parar a gravação de qualquer lugar.
5. Ao parar, o caminho do arquivo MP4 é mostrado.

## Como baixar

Baixe o instalador `.msi` na aba **Releases** do repositório e execute-o.

> **Aviso:** o MSI não é assinado digitalmente. O Windows SmartScreen pode exibir um alerta; clique em "Mais informações" e depois em "Executar assim mesmo". Confira o `SHA256SUMS` publicado junto, se quiser verificar a integridade.

Também é publicado um `.msix` sem assinatura, destinado ao envio à Microsoft Store.

## Decisão técnica: sem FFmpeg

O Cap usa o encoder nativo do Windows, o **Media Foundation** (H.264 + AAC em MP4), em vez de embutir o FFmpeg. Isso reduz o instalador em dezenas de megabytes. A webcam entra no vídeo por captura direta da janela dela, sem filtros de overlay.

## Desenvolvimento

Requisitos: Rust estável, Node 20+ e, no Windows, WebView2 e o Visual Studio Build Tools.

```
npm ci
npx tauri dev          # app em modo desenvolvimento (Windows)
cargo test -p cap-core # núcleo, 100% de cobertura
npx vitest run         # testes do front-end
```

A arquitetura está em [docs/CONTRACT.md](docs/CONTRACT.md) e o empacotamento em [docs/BUILD.md](docs/BUILD.md).

## Licença

Veja [LICENSE](LICENSE).
