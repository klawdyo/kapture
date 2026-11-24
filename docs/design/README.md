# Referências de design do kapture

Estas imagens são a **fonte de verdade do visual atual**, aprovada pelo dono do projeto em 2026-11-24.

> **O protótipo do PDF (`docs/ui-reference.png` e `docs/especificacao.pdf`) é HISTÓRICO.** Ele serviu de ponto de partida e foi superado por estas imagens. Não refaça a interface para parecer com o PDF, e não reintroduza o que foi retirado (veja "Decisões" abaixo). Se uma tela mudar de propósito, atualize a imagem correspondente neste diretório no mesmo commit.

Como foram geradas: Chromium (Playwright, 2x) rodando o frontend com o backend FAKE. A tipografia no Windows usa Segoe UI. O que vale aqui é a estrutura, as proporções, os textos e as cores.

## Telas

| Arquivo | O que mostra |
| --- | --- |
| `01-barra-repouso.png` | Painel principal: barra de título, logo, Tela/Janela/Área, Câmera, Microfone, Gravar, configurações |
| `02-barra-configuracoes.png` | Painel com a linha extra de configurações aberta, sem estourar a largura |
| `03-pilula-gravando.png` | Pílula durante a gravação: ponto, cronômetro e botão de parar |
| `04-pilula-transparencia.png` | A pílula semitransparente sobre conteúdo, para não atrapalhar |
| `05-webcam.png` | Janela da webcam: moldura chanfrada, alça de arrasto, marcas vermelhas no meio dos lados |
| `06-seletor-de-area.png` | Seletor de área: sem escurecer a tela, cantos em L, 8 alças, etiqueta centralizada |
| `07` a `09` | Atualização: "Verificando atualizações…", "Atualizando… 42%", bolinha + "Reinicie para atualizar" |
| `10-atualizacao-aviso-baixar.png` | Aviso (sem atualização automática) com Baixar e Depois |
| `11-site-claro.png`, `12-site-escuro.png` | Site de apresentação (Nuxt UI) |

## Decisões que valem para qualquer sessão

- **Nome:** `kapture`, sempre com **k minúsculo**, mesmo no início de frase. O nome da marca aparece na cor da bolinha da logo (`#F93847`, token `--brand`), em texto grande no site e na barra de título do app. Nunca use "Cap" nem "Kapture" voltado ao usuário. (Os crates internos continuam `cap` e `cap-core`, de propósito.)
- **Logos:** as originais ficam em `assets/` (`logo_*_transparent.png`, `store/*`, imagem do Gemini) e **nunca são alteradas**. Qualquer derivado é uma cópia gerada por `scripts/make-icons.py`.
- **Cores:** grafite azulado nas superfícies. O vermelho vivo (`--red`, `--red2`) só aparece em Gravar, indicadores ativos (ponto de gravação, alça em uso, bolinha de atualização) e detalhes de enquadramento. Nada de azul novo.
- **Pílula de gravação:** só ponto, cronômetro e **parar**. **Sem botão de pausa** (retirado a pedido do dono; `pause`/`resume` continuam no backend, sem uso na UI). Compacta, arrastável por todo o corpo (menos o botão de parar) e semitransparente em repouso.
- **Painel e janelas:** tudo arrastável pelas áreas sem controle. A barra fica fora do vídeo (`set_content_protected`) só enquanto grava ou pausado (regra pura `protected` do cap-core, aplicada ao emitir `rec-state`); ocioso, o Print Screen a captura. O seletor de área não é protegido.
- **Seletor de área:** não escurece o fundo; o retângulo é movível e redimensionável pelas 8 alças.
- **Atualização:** o app verifica ao abrir e avisa com a bolinha (Baixar abre a página do release). A troca automática do exe é pendência (CLAUDE.md). Nada disso aparece durante a gravação.
- **Distribuição:** só o `kapture.exe` solto, sem instalador, sem MSI, sem Velopack (decisão do dono). Tamanho é prioridade.
