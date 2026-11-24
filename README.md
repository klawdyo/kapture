# kapture

Gravador de tela leve para Windows. Grava a tela inteira, uma janela ou uma área, com webcam, microfone e áudio do sistema, e salva em MP4.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/logo_light_transparent.png">
    <img src="assets/logo_dark_transparent.png" alt="Logo do kapture" width="160">
  </picture>
</p>

Site: https://klawdyo.github.io/kapture/ | Download: [última versão](https://github.com/klawdyo/kapture/releases/latest)

## Como usar

1. Abra o kapture. A barra flutuante aparece no topo da tela.
2. Escolha o modo: tela, janela ou área (a área é selecionada arrastando o mouse).
3. Opcional: ligue a webcam e escolha o microfone e o áudio do sistema.
4. Clique em gravar, ou use o atalho **Ctrl+Shift+R** para iniciar e parar a gravação de qualquer lugar.
5. Ao parar, o caminho do arquivo MP4 é mostrado.

## Como baixar

Baixe o `kapture-win-Setup.exe` na aba **Releases** do repositório e execute-o. A instalação é de um clique e **por usuário**: o kapture vai para `%LOCALAPPDATA%\kapture`, sem pedir administrador, e cria atalhos na área de trabalho e no menu iniciar. Para desinstalar, use **Configurações > Aplicativos > Aplicativos instalados** (Apps e recursos).

> **Aviso:** o instalador não é assinado digitalmente. O Windows SmartScreen pode exibir um alerta; clique em "Mais informações" e depois em "Executar assim mesmo". Confira o `SHA256SUMS` publicado junto, se quiser verificar a integridade.

O `.msi` continua nos Releases como alternativa temporária, até o novo instalador ser aprovado.

O `.msix` para a Microsoft Store está desligado por enquanto (veja [docs/msix-store.md](docs/msix-store.md)).

## Decisão técnica: sem FFmpeg

O kapture usa o encoder nativo do Windows, o **Media Foundation** (H.264 + AAC em MP4), em vez de embutir o FFmpeg. Isso reduz o instalador em dezenas de megabytes. A webcam entra no vídeo por captura direta da janela dela, sem filtros de overlay.

## Onde ficam os logs

O kapture grava um diário de diagnóstico em `%LOCALAPPDATA%\kapture\logs\kapture.log`, normalmente `C:\Users\<usuário>\AppData\Local\kapture\logs\kapture.log`. Se essa pasta não puder ser localizada, ele usa a pasta temporária do Windows. Cada linha tem data e hora em UTC e o nível (`INFO`, `WARN`, `ERROR`).

O arquivo é trocado ao passar de 512 KB: `kapture.log` vira `kapture.1.log`, depois `kapture.2.log`, e o mais antigo é apagado (no máximo 3 arquivos). Ele registra o início do app (versão, versão do Windows), cada comando com o resultado e a duração, os dispositivos encontrados, o ciclo da gravação, erros do Windows com o código HRESULT e erros inesperados da interface ou do próprio programa.

Para abrir a pasta, clique na engrenagem da barra e depois em **Abrir logs**. Se algo der errado, envie o `kapture.log` (e o `kapture.1.log`, se existir).

## Atualizações

Ao abrir, o kapture consulta uma única vez a página de Releases do GitHub (`api.github.com/repos/klawdyo/kapture/releases/latest`) para saber se existe versão mais nova. A consulta é um GET simples, sem enviar nenhum dado seu, com limite de 5 segundos. Se houver atualização, uma bolinha vermelha piscando aparece na barra de título; ao clicar, o app mostra as versões instalada e nova, com os botões **Baixar** (abre a página do release no navegador) e **Depois**. Sem atualização ou sem internet, nada aparece (o resultado só vai para o log). Com a opção de movimento reduzido do Windows ligada, a bolinha fica parada. O app não baixa nem instala nada sozinho. Não há opção para desativar a consulta; para bloquear, negue a conexão do app a `api.github.com` no firewall.

## Desenvolvimento

Requisitos: Rust estável, Node 20+ e, no Windows, WebView2 e o Visual Studio Build Tools.

```
npm ci
npx tauri dev          # app em modo desenvolvimento (Windows)
cargo test -p cap-core # núcleo, 100% de cobertura
npx vitest run         # testes do front-end
```

Fluxo: o desenvolvimento acontece na `dev`; a `main` só recebe releases (merge da `dev`), que criam a tag e o GitHub Release automaticamente. Os commits seguem Conventional Commits e a versão é calculada a partir deles. Veja [CONTRIBUTING.md](CONTRIBUTING.md) e [docs/versioning.md](docs/versioning.md).

A arquitetura está em [docs/CONTRACT.md](docs/CONTRACT.md) e o empacotamento em [docs/BUILD.md](docs/BUILD.md).

## Licença

Veja [LICENSE](LICENSE).
