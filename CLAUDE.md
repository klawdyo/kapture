# kapture: instruções para agentes (IA ou pessoas)

Gravador de tela leve para Windows (Tauri v2 + Vue 3 + Rust). Leia isto antes de mexer em qualquer coisa.

## Fontes de verdade

1. **Visual atual:** `docs/design/` (README e prints). O PDF `docs/especificacao.pdf` e `docs/ui-reference.png` são **históricos**: não os use como modelo de interface.
2. **Contrato entre frontend e backend:** `docs/CONTRACT.md` (comandos, eventos, tamanhos de janela).
3. **Build, CI, release, atualização:** `docs/BUILD.md`. **Versionamento:** `docs/versioning.md`.
4. Em conflito, vale o que está em `docs/design/` e em `CLAUDE.md` sobre o PDF.

## Decisões do dono que não se desfazem sem ordem expressa

- Nome do produto: **kapture** (k minúsculo), com o nome na cor da bolinha da logo. O nome interno dos crates continua `cap` e `cap-core`.
- Nunca altere as imagens originais em `assets/`. Gere cópias (`scripts/make-icons.py`).
- Pílula de gravação sem botão de pausa. Barra e seletor de área arrastáveis.
- **Distribuição só pelo `kapture.exe` solto, sem instalador, sem MSI, sem Velopack.** Sem instalação: o usuário baixa e executa. O MSIX da Store (`packaging/msix`) fica, desligado. O app existe para cumprir a obrigação e deve ser o mais leve possível: não adicione dependências pesadas (nada de reqwest, plugins do Tauri sem necessidade, FFmpeg).
- Tamanho exibido no site: manter sincronizado como **"menos de 2 MB"** (hero, recursos e meta). Sempre que o tamanho real do `kapture.exe` compilado ultrapassar 2 MB, atualizar esse valor no site.
- Assinatura digital (SmartScreen): o dono decidiu manter o `kapture.exe` sem assinatura digital por enquanto, mantendo o aviso discreto do SmartScreen no site e no README.
- Commits breaking (`!`) são proibidos sem ordem explícita do dono.

## Convenções de código (do dono)

- Comentários em **tudo**, em português. Textos de tela e mensagens ao usuário em português.
- Identificadores (arquivos, variáveis, tipos, componentes) em **inglês**, nomes **curtos** (`price`, não `salePrice`).
- Clean Architecture: toda capacidade externa atrás de uma **porta** (trait ou interface) com implementação **real** e **fake**. Domínio e casos de uso não conhecem Tauri nem Windows.
- Vue: nenhum componente com lógica de negócio. Cada tela tem seu composable com seus dados. Widgets **genéricos** e centralizados em `src/ui/`; os especializados do domínio ficam em `src/components/` e compõem os genéricos. Nenhuma tela usa elemento cru fora de `src/ui/`.
- Cobertura: `cap-core` com 100% das linhas (`cargo llvm-cov -p cap-core --fail-under-lines 100`; apague `target/llvm-cov-target` antes de medir, e use o Rust stable atual, que é o do CI) e frontend com 100% (`npx vitest run --coverage`).
- Sem travessões como aposto nos textos: use vírgulas ou dois-pontos.

## Fluxo de trabalho e CI (o dono não quer gastar Actions à toa)

- **Só envie ao remoto com algo pronto, funcionando e verificado localmente.** Nada de push parcial.
- Fluxo: trabalho na `dev`, PR da `dev` para a `main`, o merge na `main` dispara o release. Push em `dev` não roda CI.
- PR para `dev`: só Linux e site. PR para `main`: também Windows e o job `installer`. O release não repete os testes.
- Tipo de commit decide a versão (`feat` sobe o minor, `fix` o patch). Use `chore`, `style`, `refactor`, `docs`, `ci`, `test` quando não for funcionalidade.
- Antes de abrir PR, rode: `cargo fmt --all --check`, `cargo clippy -p cap-core --all-targets -- -D warnings`, a cobertura do core, `npx vitest run --coverage`, `npx vue-tsc --noEmit`, `npm run build` e, em `site/`, `npm run generate`.

## O que nunca foi validado em Windows real

Captura, microfone, webcam, gravação em MP4, mica, sombra, `fit_toolbar`, atalho global, ícone embutido no exe sem bundle, CSP e CORS da verificação de atualização. Não afirme que funcionam sem teste numa máquina Windows.

## Pendências pedidas pelo dono (ainda não implementadas)

1. **Seletor de área, proporção:** opções abaixo do retângulo para travar a proporção: 16:9, 4:3, 3:4, 9:16 ou livre. Com uma proporção fixa, redimensionar mantém a proporção (lógica pura em `src/domain/rect.ts`, com testes).
2. **Seletor de área, indicador de tamanho:** manter o formato parecido com a etiqueta atual de dimensões (ex.: 1280 × 720), mas com pequenas diferenças visuais entre a **etiqueta informativa** e os **botões de proporção**, para ninguém confundir badge com botão.
3. **Webcam:** simplificar para um **círculo com borda branca fina** (sem moldura chanfrada nem marcas). Com o mouse em cima, a **roda do mouse** aumenta ou diminui o tamanho, inclusive durante a gravação. Manter 1:1 e os limites mínimo e máximo de `cam_side`.
4. **Atualização automática trocando o próprio exe:** baixar o `kapture.exe` novo do release para um arquivo ao lado, renomear o exe em uso, mover o novo para o lugar e reiniciar; usar WinHTTP via crate `windows` (sem reqwest/ureq). Hoje o adaptador de produção é o `NoUpdater` (só avisa e abre a página do release).
5. **Regras de envio:** **não envie ao remoto nem abra PR sem o dono mandar.**
