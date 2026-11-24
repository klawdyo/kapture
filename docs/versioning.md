# Versionamento e releases

A versão do Cap é calculada automaticamente a partir dos commits. Ninguém edita número de versão à mão. A implementação está em `scripts/version.mjs` (testes em `scripts/version.test.mjs`).

## Commits (Conventional Commits)

Todo commit segue `tipo(escopo)!: descrição`, com escopo e `!` opcionais.

Tipos aceitos: `feat`, `fix`, `perf`, `refactor`, `docs`, `style`, `test`, `build`, `ci`, `chore`, `revert`.

## Regra de cálculo

1. Base: a última tag `vX.Y.Z` alcançável a partir de `HEAD` (a maior versão). Sem nenhuma tag, a base é `0.0.0` e todo o histórico é considerado.
2. Percorre-se os commits desde a base em ordem cronológica, do mais antigo ao mais novo.
3. Cada commit conta individualmente e o efeito é somado em sequência:
   - `feat`: sobe o minor e **zera o patch**.
   - `fix`: sobe o patch.
   - Qualquer outro tipo (`docs`, `chore`, `ci`, `test`, `refactor`...): não muda a versão.
4. Não vale o maior bump: vale a sequência. `fix` depois de `feat` sobe o patch (`1.1.1`); `feat` depois de `fix` zera de novo (`1.1.0`).

### Casos especiais

- **Merge commits** (mais de um pai, ou assunto começando com `Merge `): são ignorados no cálculo, no changelog e na validação. Contam os commits reais que vieram no merge.
- **Escopo** (`feat(ui): ...`): não altera o cálculo. Aparece em negrito no changelog.
- **`!` ou `BREAKING CHANGE:` no corpo**: como a regra só define `feat` e `fix`, a decisão é tratar como **minor** (zera o patch), igual a um `feat`, e destacar o item na seção "Mudanças incompatíveis". Um commit com `!` conta uma vez (não soma o bump do seu tipo). Configurável em uma linha: a constante `BREAKING` em `scripts/version.mjs` aceita `'minor'` (padrão), `'major'` (sobe major e zera minor e patch) ou `'none'` (ignora o marcador).
- **Commits fora do padrão**: a validação (`check`) falha no PR e no CI, e no cálculo eles são ignorados (não contam e não entram nas notas). Reverts gerados pelo git (`Revert "..."`) também precisam ser reescritos como `revert: ...`.
- **Sem commits `feat`/`fix` desde a última tag**: a versão calculada é igual à base e **nenhum release é publicado**.

### Exemplo

Última tag `v1.4.2`. Commits, do mais antigo ao mais novo:

| Commit | Efeito | Versão |
| --- | --- | --- |
| `fix: corrige crash` | patch | 1.4.3 |
| `fix: ajusta volume` | patch | 1.4.4 |
| `feat: grava em área` | minor, zera patch | 1.5.0 |
| `fix: borda da área` | patch | 1.5.1 |
| `docs: atualiza README` | nenhum | 1.5.1 |
| `Merge pull request #7 ...` | ignorado | 1.5.1 |
| `feat(webcam): espelhar` | minor, zera patch | 1.6.0 |
| `chore: deps` | nenhum | 1.6.0 |

Próxima versão: **1.6.0**.

## Fluxo de branches

- `dev` concentra o desenvolvimento. Os PRs entram nela (e em `main` só via `dev`).
- `main` só recebe releases finais: o merge de `dev` em `main`.
- Quando a `main` recebe o push, o workflow `release.yml` calcula a versão, aplica em `tauri.conf.json`, `package.json` e nos `Cargo.toml` (só dentro do build, sem commit), gera MSI e MSIX, cria a tag `vX.Y.Z` e o GitHub Release. A tag é criada no release porque o ambiente de desenvolvimento não consegue criar tags na `main`.
- Os números de versão guardados no repositório são apenas placeholders. A fonte da verdade são as tags.
- O release também pode ser disparado manualmente, a partir da `main`.

## Notas do release (changelog)

O release traz as notas do que mudou desde a última tag, com o título `## vX.Y.Z (data)` e seções por tipo: Mudanças incompatíveis, Novidades (`feat`), Correções (`fix`), Desempenho, Refatoração, Documentação, Testes, Build, CI, Estilo, Manutenção e Reversões. Seções vazias são omitidas.

## Comandos

```
node scripts/version.mjs next            # próxima versão
node scripts/version.mjs base            # versão da última tag (0.0.0 sem tag)
node scripts/version.mjs notes           # changelog em markdown
node scripts/version.mjs check <range>   # valida commits, ex.: origin/dev..HEAD
node scripts/version.mjs apply X.Y.Z     # grava a versão nos arquivos do projeto
```

O cálculo precisa do histórico completo e das tags (`git fetch --tags --unshallow`, ou `fetch-depth: 0` no CI).
