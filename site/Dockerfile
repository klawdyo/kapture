# syntax=docker/dockerfile:1

# Etapa 1: gera o site estático usando o contexto da raiz do repositório para
# compartilhar os componentes Vue do app em src/ com site/.
FROM node:22-alpine AS build
WORKDIR /repo

ARG NUXT_PUBLIC_DOWNLOAD_URL=https://github.com/klawdyo/kapture/releases/latest
ARG NUXT_PUBLIC_SITE_URL=
ENV NUXT_PUBLIC_DOWNLOAD_URL=$NUXT_PUBLIC_DOWNLOAD_URL \
    NUXT_PUBLIC_SITE_URL=$NUXT_PUBLIC_SITE_URL \
    NUXT_TELEMETRY_DISABLED=1

COPY site/package.json site/package-lock.json ./site/
RUN cd site && npm ci
COPY src ./src
COPY site ./site
RUN cd site && npm run generate

# Etapa 2: nginx servindo só os arquivos gerados.
FROM nginx:alpine AS runtime
COPY site/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /repo/site/.output/public /usr/share/nginx/html
EXPOSE 80
HEALTHCHECK --interval=30s --timeout=3s CMD wget -qO /dev/null http://127.0.0.1/ || exit 1
