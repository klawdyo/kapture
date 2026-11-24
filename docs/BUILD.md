# Build e distribuição

A distribuição é só o executável solto `kapture.exe` (decisão do dono): sem instalador, sem MSI, sem Velopack. Compile com `npx tauri build --no-bundle` (o `bundle.active` do `tauri.conf.json` está `false`); o ícone vem de `src-tauri/icons/icon.ico`, embutido no exe pelo `tauri-build`.

## Pré-requisitos (Windows)

- Rust estável com alvo `x86_64-pc-windows-msvc`
- Node 20 ou superior
- Visual Studio Build Tools (C++) e WebView2
- Para o MSIX: Windows SDK (traz o `MakeAppx.exe`)

## Nome do produto e nome interno dos crates

O produto se chama **kapture** (sempre com k minúsculo): `productName`, título das janelas, executável (`kapture.exe`, via `[[bin]]` e `mainBinaryName`), pasta de logs e pacote MSIX usam esse nome. O nome interno dos crates e módulos continua `cap` (`cap`, `cap-core`, `cap_lib`, `cap_core::`, `-p cap`). Isso não aparece para o usuário e foi mantido de propósito para não arriscar uma renomeação ampla do código Rust; por isso os comandos de teste abaixo ainda usam `-p cap-core`.

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

Em tempo de execução o app grava `%LOCALAPPDATA%\kapture\logs\kapture.log` (fallback: `%TEMP%\kapture\logs`), com rotação em `kapture.1.log` e `kapture.2.log` ao passar de 512 KB. A rotina é do `cap-core` (`log` e `logfile`, com 100% de cobertura) e o botão **Abrir logs**, no painel de configurações da barra, abre a pasta no Explorer. Para investigar um erro numa máquina real, peça o `kapture.log`: ele traz o HRESULT em hexadecimal de cada falha dos adaptadores Windows, os erros do WebView e, se o app abortar, a mensagem e o local (`arquivo:linha`) do pânico. O release usa `strip = true`, então o backtrace do pânico tem só endereços.

## Atualizações

Hoje o app só **avisa**; não troca o próprio exe.

1. **Porta e caso de uso.** Ao abrir a toolbar o frontend chama `update_start`; o caso de uso `Update` (cap-core, `app/upd.rs`) roda o ciclo uma única vez numa thread e publica o evento `update-state` (`checking`, `downloading` com `pct`, `ready`, `none`, `fallback`, `err`). O adaptador de produção é o `NoUpdater` (`src-tauri/src/adapters/noupd.rs`, em todos os SOs), com `supported() = false`: `check`, `download` e `apply_and_restart` devolvem erro em português.
2. **Fallback (aviso simples).** Com `supported() = false` o backend emite `fallback` e o frontend faz a consulta: `realUpdates` (`ports/real.ts`) com um `fetch` GET a `https://api.github.com/repos/klawdyo/kapture/releases/latest`, timeout de 5 s, e mostra "Atualização disponível" com **Baixar** (comando `open_url`, que só abre `https://github.com/klawdyo/kapture/releases/*`) e **Depois**.

A CSP de `tauri.conf.json` libera só `connect-src https://api.github.com` (o fallback). A versão instalada vem de `getVersion()` (permissão `core:app:allow-version`); o release precisa ter tag `vX.Y.Z` maior que ela. Nenhum dado do usuário é enviado e não há opção de desativar.

**Atualizador futuro:** basta outro tipo que implemente `cap_core::ports::Updater` (`supported`, `check`, `download` com progresso, `apply_and_restart`) e trocar a linha `updater:` em `plat()` de `src-tauri/src/lib.rs`. Núcleo, comandos e interface não mudam. O `FakeUpdater` cobre os testes.

**Validação no Linux:** o crate `ring` (dependência do Tauri/WebView) pode precisar do MSVC. Para só checar o código do alvo Windows sem as ferramentas, aponte `CC_x86_64_pc_windows_msvc` e `AR_x86_64_pc_windows_msvc` para um script que apenas cria os arquivos de saída e rode `cargo clippy -p cap --all-targets --target x86_64-pc-windows-msvc -- -D warnings`. No CI (Windows) o build é o real.

## MSIX

```
npx tauri build --no-bundle
./packaging/msix/build-msix.ps1 -Version 0.1.0 -Exe target/release/kapture.exe -Out dist/kapture.msix
```

O pacote não é assinado: serve para envio à Microsoft Store, que assina na certificação. Antes de enviar, troque os placeholders de `Identity/Name`, `Identity/Publisher` e `PublisherDisplayName` em `packaging/msix/AppxManifest.xml` pelos valores do Partner Center. O nome do executável no manifesto (`kapture.exe`) deve coincidir com o produzido pelo Tauri.

## Ícones

As logos oficiais são `assets/logo_light_transparent.png` e `assets/logo_dark_transparent.png` (800x800, RGBA, fundo transparente, com o arredondado próprio). Elas nunca são alteradas: tudo o mais é cópia gerada. (`assets/logo-light.png` e `assets/logo-dark.png` são as logos antigas, em JPEG com fundo opaco, mantidas só por histórico.) Todo o resto deriva das transparentes:

- `src-tauri/icons/`: `32x32.png`, `128x128.png`, `128x128@2x.png` (256), `icon.png` (512) e `icon.ico` (16, 24, 32, 48, 64, 128 e 256, PNG comprimido em cada entrada);
- `packaging/msix/Assets/`: `StoreLogo`, `Square44x44`, `Square71x71`, `Square150x150`, `Wide310x150` e `Square310x310`, com fundo transparente;
- `site/public/`: `logo-light*.png`, `logo-dark*.png` (64, 128 e 480 px), `favicon.png` e `og.png` (1200x630, logo sobre grafite);
- `src/assets/logo.png` (128 px, usada pela barra do app).

Os ícones do app, do MSIX e a barra usam a versão **clara**: ela mantém contraste sobre a barra de tarefas escura (padrão do Windows) e sobre a barra do app. O site e o README alternam as duas pelo tema.

Para regenerar tudo depois de trocar uma logo (requer Python 3 e Pillow):

```
pip install pillow
python scripts/make-icons.py
```

O script é idempotente: regrava sempre os mesmos arquivos a partir das duas logos. `assets/store/` (artes da listagem da Microsoft Store) e a imagem original gerada (`assets/Gemini_Generated_Image_*.png`, 3,3 MB) não são tocadas por ele. `docs/ui-reference.png` é só referência visual da interface.

## Release

O release é automático: o push na `main` (merge da `dev`) executa o `release.yml`, que calcula a versão pelos commits (veja [versioning.md](versioning.md)), compila o `kapture.exe` (`tauri build --no-bundle`, sem repetir os testes, que o PR para a `main` já rodou), cria a tag `vX.Y.Z` e publica só o `kapture.exe` e o `SHA256SUMS` (hash do exe) no GitHub Release com as notas geradas. A geração do MSIX está desligada no release (veja [msix-store.md](msix-store.md)). Sem commits `feat`, `fix` ou breaking desde a última tag, nada é publicado. Também pode ser disparado manualmente na `main`, e nesse caso os testes rodam antes.

## CI e site

O `ci.yml` roda só em pull request (para `main` ou `dev`) e quando o `release.yml` o chama. Não há CI por push: o PR já roda os testes e o push na `main` fica por conta do release.

- **PR para `dev`:** só o job Linux (fmt, clippy, cobertura de 100%, vitest) e o job `site` (confirma o `npm run generate` do Nuxt). É o caminho rápido.
- **PR para `main`:** os mesmos jobs mais o job Windows (`cargo test --workspace` e `cargo check` do Tauri).
- **Release (`release.yml`):** só compila o exe e publica, porque o PR para a `main` já rodou a suíte completa sobre o mesmo código. Os testes só rodam de novo no disparo manual (`workflow_dispatch`), que não tem PR por trás. Para rodá-los sempre, use o `if` comentado do job `tests`.
- **Desligado, mas mantido comentado em `ci.yml`:** o push na `dev`. Para voltar a usá-lo, basta descomentar o bloco.

O site (`site/`) é publicado pelo `pages.yml` em https://klawdyo.github.io/kapture/ quando a `main` recebe mudanças em `site/` (ou manualmente). O build usa `NUXT_APP_BASE_URL=/kapture/`. Em Settings > Pages do repositório, a fonte deve ser **GitHub Actions**.
