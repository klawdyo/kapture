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

Ao abrir, o kapture procura uma versão nova sozinho. Na barra de título aparece, em cinza e letra pequena, **Verificando atualizações…**. Se houver versão nova, ela é baixada automaticamente (**Atualizando… 42%**) e, ao terminar, surge uma bolinha vermelha piscando com **Reinicie para atualizar**: um clique reinicia o app já na versão nova. Se não houver atualização ou faltar internet, o texto some sem aviso (o motivo só vai para o log). Nada disso aparece durante a gravação. Com a opção de movimento reduzido do Windows ligada, a bolinha fica parada.

Isso só funciona quando o app foi instalado pelo **Setup do Velopack** (`kapture-win-Setup.exe`). Rodando solto, pelo MSI ou em desenvolvimento, o app só avisa: aparece **Atualização disponível** com a bolinha, e o clique mostra as versões instalada e nova com os botões **Baixar** (abre a página do release no navegador) e **Depois**.

O que é enviado: apenas as requisições comuns ao GitHub (`github.com/klawdyo/kapture/releases/latest/download/` para o índice e o pacote, ou `api.github.com/repos/klawdyo/kapture/releases/latest` no aviso simples), sem enviar nenhum dado seu além do que toda requisição HTTPS carrega. A consulta é feita uma vez por abertura. Não há opção para desativar; para bloquear, negue a conexão do app a `github.com` e `api.github.com` no firewall.

O mecanismo de atualização (Velopack) fica atrás da porta `Updater` do núcleo, então pode ser trocado por outro sem mexer na interface.

## Desenvolvimento

Requisitos: Rust estável, Node 20+ e, no Windows, WebView2 e o Visual Studio Build Tools.

```
npm ci
npx tauri dev          # app em modo desenvolvimento (Windows)
cargo test -p cap-core # núcleo, 100% de cobertura
npx vitest run         # testes do front-end
```

Fluxo: o desenvolvimento acontece na `dev`; a `main` só recebe releases (merge da `dev`). O GitHub Release é criado manualmente (Actions, workflow Release), que calcula a versão, cria a tag e publica os instaladores. Os commits seguem Conventional Commits e a versão é calculada a partir deles. Veja [CONTRIBUTING.md](CONTRIBUTING.md) e [docs/versioning.md](docs/versioning.md).

A arquitetura está em [docs/CONTRACT.md](docs/CONTRACT.md) e o empacotamento em [docs/BUILD.md](docs/BUILD.md).

## Privacidade

O kapture não coleta telemetria nem envia dados do usuário. Leia a política completa em [PRIVACY.md](PRIVACY.md).

## Licença

Veja [LICENSE](LICENSE).
