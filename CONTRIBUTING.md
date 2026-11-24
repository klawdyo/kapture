# Como contribuir

## Branches

- Trabalhe a partir da `dev` e abra PRs para ela.
- A `main` só recebe releases, por merge da `dev`. O push na `main` dispara o release (tag, versão e instaladores).

## Commits semânticos

Todo commit segue Conventional Commits: `tipo(escopo): descrição`.

```
feat(webcam): espelhar a imagem
fix: corrigir travamento ao parar a gravação
docs: atualizar o README
```

`feat` sobe o minor, `fix` sobe o patch, os demais tipos não geram versão. Use `!` (`feat!: ...`) para mudança incompatível. O CI recusa PRs com commits fora do padrão. Valide localmente com `node scripts/version.mjs check origin/dev..HEAD`.

As regras completas estão em [docs/versioning.md](docs/versioning.md).

## Testes

`npm test` (front-end e script de versão) e `cargo test -p cap-core`, ambos com 100% de cobertura.
