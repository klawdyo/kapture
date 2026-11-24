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

## Onde ficam os logs

Em tempo de execução o app grava `%LOCALAPPDATA%\Cap\logs\cap.log` (fallback: `%TEMP%\Cap\logs`), com rotação em `cap.1.log` e `cap.2.log` ao passar de 512 KB. A rotina é do `cap-core` (`log` e `logfile`, com 100% de cobertura) e o botão **Abrir logs**, no painel de configurações da barra, abre a pasta no Explorer. Para investigar um erro numa máquina real, peça o `cap.log`: ele traz o HRESULT em hexadecimal de cada falha dos adaptadores Windows, os erros do WebView e, se o app abortar, a mensagem e o local (`arquivo:linha`) do pânico. O release usa `strip = true`, então o backtrace do pânico tem só endereços.

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

A logo oficial (raster 380x380, fundo opaco com grade, recortada da Figura 1 da especificação) está em `assets/logo-light.png` e `assets/logo-dark.png`. Os ícones em `src-tauri/icons/`, os PNGs do MSIX em `packaging/msix/Assets/`, as imagens do site e `src/assets/logo.png` derivam da versão escura (cantos arredondados aplicados). `docs/ui-reference.png` é só referência visual da interface.

## Release

Criar uma tag `v*` (ex.: `git tag v0.1.0 && git push --tags`) ou rodar o workflow manualmente. O `release.yml` executa os testes do `ci.yml`, gera MSI e MSIX, imprime os tamanhos e publica `.msi`, `.msix` e `SHA256SUMS` no GitHub Release. A versão vem da tag (sem o `v`); em execução manual, do `package.json`.
