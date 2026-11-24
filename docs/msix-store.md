# Publicar o Cap na Microsoft Store (MSIX)

Este guia cobre o cadastro no Partner Center, de onde vem cada valor do manifesto e o que é público ou sensível. Os nomes de campos do Partner Center podem mudar de lugar na interface; o que não pude verificar daqui está marcado como "conferir".

## Estado atual

A geração do MSIX está **desligada** no `.github/workflows/release.yml` (passos comentados), até a identidade da Store e as imagens/logo estarem prontas. O release publica só o MSI.

Para religar: descomente, no job de build, os passos "Gerar exe para o MSIX" e "Gerar MSIX (MakeAppx)" e inclua a linha `dist/*.msix` na lista `files` do passo de publicação. Depois configure as variáveis `MSIX_IDENTITY_NAME`, `MSIX_PUBLISHER` e `MSIX_PUBLISHER_DISPLAY` (seção 4).

## 1. Passo a passo no Partner Center

1. Crie a conta de desenvolvedor em partner.microsoft.com (tipo Individual ou Empresa; há taxa única de registro, valor a conferir na página de inscrição).
2. Em **Apps e jogos**, escolha **Novo produto** > **App MSIX ou PWA** e reserve o nome do app ("Cap", se estiver livre). A reserva vale por tempo limitado se você não enviar um pacote (conferir o prazo exibido).
3. Abra o produto e vá em **Gerenciamento de produto** > **Identidade do produto** (em inglês, *Product identity*; em versões antigas, *App identity*). Ali ficam os valores do manifesto (seção 2).
4. Ajuste o manifesto `packaging/msix/AppxManifest.xml` via variáveis (seção 4) e gere o pacote com `build-msix.ps1`.
5. Crie uma **Submissão**: preços e disponibilidade, propriedades (categoria, classificação etária), listagem (descrição, capturas de tela, ícone) e **Pacotes** (envie o `.msix` não assinado).
6. Responda ao questionário de classificação etária e justifique a capability restrita `runFullTrust` (o app é Win32/Tauri) e o uso de webcam e microfone. Informe na certificação que a captura de tela é feita só localmente.
7. Envie para certificação. A Store assina o pacote com o certificado dela; por isso não é preciso `.pfx` para publicar.

## 2. De onde vem cada valor

| Valor no manifesto | Onde pegar no Partner Center |
|---|---|
| `Identity/@Name` | Identidade do produto > **Package/Identity/Name** (algo como `12345Fulano.Cap`) |
| `Identity/@Publisher` | Identidade do produto > **Package/Identity/Publisher** (formato `CN=GUID`) |
| `PublisherDisplayName` | Identidade do produto > **Package/Properties/PublisherDisplayName**. É o nome de publicador da conta. |
| Package Family Name (PFN) | Identidade do produto > **Package Family Name** (também aparece em Gerenciamento do produto). Formato `Name_hash`. Não vai no manifesto. |
| Store ID (Product ID) | Visão geral do produto, ID de 12 caracteres (ex.: `9NBLGGH4XXXX`). Usado na URL `https://apps.microsoft.com/detail/<StoreID>` e em `ms-windows-store://pdp/?productid=<StoreID>`. |
| Assinatura | Nada a fazer para a Store: ela reassina. O `Publisher` do manifesto precisa coincidir exatamente com o da identidade reservada, senão o envio é rejeitado. |

Conferir sempre: o texto do `Publisher` copiado do Partner Center, sem espaços extras, e o `Name` idêntico ao reservado.

## 3. Análise: público, privado e onde guardar

| Informação | Visibilidade | Destino recomendado | Motivo |
|---|---|---|---|
| Identity Name | Pública | GitHub **Variable** (`MSIX_IDENTITY_NAME`), ou commitada | Está no `AppxManifest.xml` dentro do `.msix` distribuído e é o prefixo do PFN, visível na Store. |
| Publisher (`CN=GUID`) | Pública | GitHub **Variable** (`MSIX_PUBLISHER`), ou commitada | Idem: vai em claro no manifesto e no certificado de assinatura do pacote. Não é credencial. |
| PublisherDisplayName | Pública | GitHub **Variable** (`MSIX_PUBLISHER_DISPLAY`), ou commitada | Aparece na página da Store e no manifesto. |
| Package Family Name | Pública | Variable opcional, ou commitada (doc) | Derivado de Name + Publisher; aparece em caminhos do sistema do usuário (`%LOCALAPPDATA%\Packages\<PFN>`). |
| Store ID / Product ID | Público | Variable opcional, ou commitado | Está na URL pública da listagem. |
| Certificado `.pfx` (sideload/assinatura própria) | Privada | **Secret** (base64) | Dá poder de assinar como você. Não é necessário para a Store. |
| Senha do `.pfx` | Privada | **Secret** | Idem. |
| Tenant ID, Client ID, Client Secret da API de submissão da Store (Azure AD) | Privadas | **Secrets** | Só se for automatizar o envio. O Client Secret é credencial; tenant e client id sozinhos não dão acesso, mas é prática comum tratá-los juntos como secret. |
| Seller ID / Application ID de submissão | Semi-privados | **Secret** por cautela | Usados na automação; não são segredos fortes, mas não há ganho em expô-los. |

O que tenho certeza: Name, Publisher e PublisherDisplayName ficam legíveis em qualquer `.msix` baixado (é um zip com o manifesto em texto), então esconder em Secret não protege nada, e ainda mascara os valores nos logs, o que atrapalha a depuração. `.pfx`, senha e client secret são credenciais. O que conferir no Partner Center: se a sua conta exige alguma etapa extra de assinatura (não é o caso para MSIX enviado pela Store), e como a sua conta cria o app registration da API de submissão (Configurações da conta > Gerenciamento de usuários > Aplicativos do Azure AD).

Observação: GitHub Variables não são mascaradas nos logs; se preferir não deixar o GUID evidente no repositório público, a Variable não resolve isso (ela é visível a quem tem acesso de leitura), e de todo modo o valor já é público no pacote.

## 4. Integração com o workflow

`build-msix.ps1` lê estes parâmetros, com fallback para variáveis de ambiente de mesmo sentido:

| Parâmetro | Variável de ambiente | Origem no GitHub |
|---|---|---|
| `-IdentityName` | `MSIX_IDENTITY_NAME` | `vars.MSIX_IDENTITY_NAME` |
| `-Publisher` | `MSIX_PUBLISHER` | `vars.MSIX_PUBLISHER` |
| `-PublisherDisplayName` | `MSIX_PUBLISHER_DISPLAY` | `vars.MSIX_PUBLISHER_DISPLAY` |

Sem as variáveis, o script usa placeholders e avisa; o pacote resultante serve só para testes locais.

## 5. Versão

A Store exige `Major.Minor.Build.0`. O script força o 4º segmento em `0`; com a versão `0.1.0`, o MSIX sai `0.1.0.0`. Toda nova submissão precisa de versão maior que a anterior.

## 6. Assets e capabilities

- Assets atuais: `Square44x44`, `Square150x150`, `Wide310x150`, `StoreLogo` (escala única). Funciona, mas a Store e o Windows preferem variantes de escala (`.scale-100/125/150/200/400`) e `Square44x44Logo.targetsize-*`; recomenda-se gerá-las a partir da logo antes de publicar.
- `webcam` e `microphone`: usadas por Media Foundation, WASAPI e `getUserMedia`.
- `runFullTrust`: é obrigatória para qualquer app desktop Win32/Tauri empacotado em MSIX, porque o exe roda fora do sandbox UWP. Não dá para removê-la sem reescrever o app como UWP. É uma capability restrita: a Store pede uma justificativa no envio. Texto sugerido para o campo:

  - Português: "O Cap é um aplicativo desktop Win32 (Tauri) de gravação de tela. Ele precisa de runFullTrust para executar como processo de desktop e acessar as APIs de captura de tela e áudio do Windows. A captura e o processamento são feitos apenas localmente, sem envio de dados."
  - English: "Cap is a Win32 desktop (Tauri) screen recording app. It requires runFullTrust to run as a full desktop process and use the Windows screen and audio capture APIs. Capture and processing happen locally only, with no data uploaded."
- `graphicsCapture` não foi declarada: o app usa Windows.Graphics.Capture por monitor/janela em processo de desktop e o loopback WASAPI, que não exigem capability. Se a certificação ou um teste em máquina limpa mostrar bloqueio de captura, reavalie (não pude testar o MSIX instalado neste ambiente Linux).
