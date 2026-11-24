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

Baixe o `kapture.exe` na aba **Releases** do repositório e execute-o. Não há instalação: é um único arquivo, que você pode guardar onde quiser (por exemplo, na área de trabalho), sem pedir administrador. Para remover, apague o arquivo (e, se quiser, a pasta de logs, veja abaixo).

> **Aviso:** o `kapture.exe` não é assinado digitalmente. O Windows SmartScreen pode exibir um alerta; clique em "Mais informações" e depois em "Executar assim mesmo". Confira o `SHA256SUMS` publicado junto, se quiser verificar a integridade.

O `.msix` para a Microsoft Store está desligado por enquanto (veja [docs/msix-store.md](docs/msix-store.md)).

## Decisão técnica: sem FFmpeg

O kapture usa o encoder nativo do Windows, o **Media Foundation** (H.264 + AAC em MP4), em vez de embutir o FFmpeg. Isso reduz o executável em dezenas de megabytes. A webcam entra no vídeo por captura direta da janela dela, sem filtros de overlay.

## Onde ficam os logs

O kapture grava um diário de diagnóstico em `%LOCALAPPDATA%\kapture\logs\kapture.log`, normalmente `C:\Users\<usuário>\AppData\Local\kapture\logs\kapture.log`. Se essa pasta não puder ser localizada, ele usa a pasta temporária do Windows. Cada linha tem data e hora em UTC e o nível (`INFO`, `WARN`, `ERROR`).

O arquivo é trocado ao passar de 512 KB: `kapture.log` vira `kapture.1.log`, depois `kapture.2.log`, e o mais antigo é apagado (no máximo 3 arquivos). Ele registra o início do app (versão, versão do Windows), cada comando com o resultado e a duração, os dispositivos encontrados, o ciclo da gravação, erros do Windows com o código HRESULT e erros inesperados da interface ou do próprio programa.

Para abrir a pasta, clique na engrenagem da barra e depois em **Abrir logs**. Se algo der errado, envie o `kapture.log` (e o `kapture.1.log`, se existir).

## Atualizações

Ao abrir, o kapture consulta o GitHub uma vez para saber se há versão nova. Se houver, aparece **Atualização disponível** com a bolinha, e o clique mostra as versões instalada e nova com os botões **Baixar** (abre a página do release no navegador, onde está o `kapture.exe` novo) e **Depois**. A troca é manual: baixe o novo `kapture.exe` e use-o no lugar do antigo. Se não houver atualização ou faltar internet, nada aparece (o motivo só vai para o log). Nada disso aparece durante a gravação.

O que é enviado: apenas a requisição comum a `api.github.com/repos/klawdyo/kapture/releases/latest`, sem nenhum dado seu além do que toda requisição HTTPS carrega. Não há opção para desativar; para bloquear, negue a conexão do app a `api.github.com` no firewall.

A atualização fica atrás da porta `Updater` do núcleo, então um futuro atualizador automático pode entrar sem mexer na interface.

## Desenvolvimento

Requisitos: Rust estável, Node 20+ e, no Windows, WebView2 e o Visual Studio Build Tools.

```
npm ci
npx tauri dev          # app em modo desenvolvimento (Windows)
cargo test -p cap-core # núcleo, 100% de cobertura
npx vitest run         # testes do front-end
```

Fluxo: o desenvolvimento acontece na `dev`; a `main` só recebe releases (merge da `dev`), que criam a tag e o GitHub Release automaticamente. Os commits seguem Conventional Commits e a versão é calculada a partir deles. Veja [CONTRIBUTING.md](CONTRIBUTING.md) e [docs/versioning.md](docs/versioning.md).

A arquitetura está em [docs/CONTRACT.md](docs/CONTRACT.md) e o empacotamento em [docs/BUILD.md](docs/BUILD.md). Os problemas já resolvidos e os testes que os impedem de voltar estão em [docs/specs/MEMORY.md](docs/specs/MEMORY.md).

## Privacidade

O kapture não coleta telemetria nem envia dados do usuário. Leia a política completa em [PRIVACY.md](PRIVACY.md).

## Licença

Veja [LICENSE](LICENSE).
