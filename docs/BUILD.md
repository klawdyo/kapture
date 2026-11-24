# Build e empacotamento

## Pré-requisitos (Windows)

- Rust estável com alvo `x86_64-pc-windows-msvc`
- Node 20 ou superior
- Visual Studio Build Tools (C++) e WebView2
- Para o MSI: o Tauri baixa o WiX Toolset automaticamente
- Para o MSIX: Windows SDK (traz o `MakeAppx.exe`)

## Testes locais

```
cargo fmt --all -- --check
cargo clippy -p cap-core --all-targets -- -D warnings
cargo llvm-cov -p cap-core --fail-under-lines 100
npm ci
npx vue-tsc --noEmit
npx vitest run --coverage
cargo test --workspace            # Windows
cargo check --manifest-path src-tauri/Cargo.toml
```

## MSI

```
npx tauri build --bundles msi
```

Saída em `target/release/bundle/msi/`.

## MSIX

```
npx tauri build --no-bundle
./packaging/msix/build-msix.ps1 -Version 0.1.0 -Exe target/release/cap.exe -Out dist/Cap.msix
```

O pacote não é assinado: serve para envio à Microsoft Store, que assina na certificação. Antes de enviar, troque os placeholders de `Identity/Name`, `Identity/Publisher` e `PublisherDisplayName` em `packaging/msix/AppxManifest.xml` pelos valores do Partner Center. O nome do executável no manifesto (`cap.exe`) deve coincidir com o produzido pelo Tauri.

## Ícones

`python3 assets/make_logo.py` (requer Pillow) regenera os SVGs em `assets/`, os ícones em `src-tauri/icons/` e os PNGs do MSIX em `packaging/msix/Assets/`.

## Release

Criar uma tag `v*` (ex.: `git tag v0.1.0 && git push --tags`) ou rodar o workflow manualmente. O `release.yml` executa os testes do `ci.yml`, gera MSI e MSIX, imprime os tamanhos e publica `.msi`, `.msix` e `SHA256SUMS` no GitHub Release. A versão vem da tag (sem o `v`); em execução manual, do `package.json`.
