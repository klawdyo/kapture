# Site do Cap

Site de apresentação do Cap (gravador de tela leve para Windows). É um projeto
[Nuxt](https://nuxt.com) 4 com [Nuxt UI](https://ui.nuxt.com) 4 e TypeScript, gerado como
**site estático** (`nuxt generate`) e servido por nginx em um container.

## Variáveis de ambiente

O site é estático: as variáveis são lidas **no build** e ficam gravadas nos arquivos.
Para mudar o valor, é preciso gerar o site de novo.

| Variável | Padrão | Uso |
| --- | --- | --- |
| `NUXT_PUBLIC_DOWNLOAD_URL` | `https://github.com/klawdyo/kapture/releases/latest` | Destino dos botões "Baixar MSI" |
| `NUXT_PUBLIC_SITE_URL` | vazio | URL pública do site, usada em `og:url` e `og:image` |

O repositório `klawdyo/kapture` é **público**, então o link padrão leva direto ao último
release, onde qualquer pessoa pode baixar o `.msi`.

## GitHub Pages

O site é publicado em https://klawdyo.github.io/kapture/ pelo workflow `.github/workflows/pages.yml`,
que roda a cada mudança em `site/` na `main`. Ele define `NUXT_APP_BASE_URL=/kapture/`, porque a
página de projeto fica sob o prefixo `/kapture/`.

Ativação, uma única vez: em **Settings > Pages**, em **Build and deployment**, escolha
**Source: GitHub Actions**. Deixe "Custom domain" vazio para usar o domínio `github.io`.

Para hospedar na raiz de um domínio (por exemplo, com o Docker abaixo), não defina
`NUXT_APP_BASE_URL`: o padrão é `/`.

## Desenvolvimento

```bash
cd site
npm ci
npm run dev        # http://localhost:3000
npm run generate   # gera .output/public
npx serve .output/public   # ou qualquer servidor estático, para conferir
```

Requer Node 22.

## Docker

```bash
cd site
docker build -t cap-site \
  --build-arg NUXT_PUBLIC_DOWNLOAD_URL=https://exemplo.com/downloads/Cap.msi \
  --build-arg NUXT_PUBLIC_SITE_URL=https://cap.exemplo.com .
docker run -d --name cap-site -p 8080:80 --restart unless-stopped cap-site
curl -I http://localhost:8080/
```

Com Compose (porta e URLs parametrizáveis; veja `.env.example`):

```bash
cp .env.example .env   # edite os valores
docker compose up -d --build
```

O `Dockerfile` tem duas etapas: `node:22-alpine` roda `npm ci && npm run generate`, e
`nginx:alpine` serve `.output/public` com a configuração de `nginx.conf` (gzip, cache longo e
imutável em `/_nuxt/*`, `404.html` como página de erro, cabeçalhos de segurança básicos e
`server_tokens off`).

## Deploy na VPS

1. Instale Docker e o plugin Compose na VPS e clone o repositório (ou copie a pasta `site/`).
2. Em `site/`, crie o `.env` com `PORT`, `NUXT_PUBLIC_DOWNLOAD_URL` e `NUXT_PUBLIC_SITE_URL`.
3. `docker compose up -d --build`.
4. Coloque um proxy reverso com HTTPS na frente (Caddy, Traefik ou nginx do host) apontando para
   `127.0.0.1:PORT`. O container só fala HTTP na porta 80.
5. Para atualizar: `git pull && docker compose up -d --build`. Para trocar só a URL de download,
   altere o `.env` e rode o mesmo comando (o build é refeito).

A política `Content-Security-Policy` do `nginx.conf` permite apenas recursos do próprio domínio
(mais scripts e estilos inline que o Nuxt gera). Se você adicionar analytics ou fontes externas,
ajuste-a.

## Estrutura

- `app/pages/index.vue`: conteúdo da página.
- `app/components/`: cabeçalho, rodapé, logo (versões clara e escura) e a pílula de gravação.
- `app/assets/css/main.css`: tema (grafite; vermelho `#EF4444` só como marca de gravação).
- `public/`: logos, favicon e `og.png`.
