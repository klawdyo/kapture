# syntax=docker/dockerfile:1

# Etapa 1: gera o site estático. O site é estático, então a URL de download é
# fixada AQUI, em tempo de build (--build-arg NUXT_PUBLIC_DOWNLOAD_URL=...).
FROM node:22-alpine AS build
WORKDIR /app

ARG NUXT_PUBLIC_DOWNLOAD_URL=https://github.com/klawdyo/kapture/releases/latest
ARG NUXT_PUBLIC_SITE_URL=
ENV NUXT_PUBLIC_DOWNLOAD_URL=$NUXT_PUBLIC_DOWNLOAD_URL \
    NUXT_PUBLIC_SITE_URL=$NUXT_PUBLIC_SITE_URL \
    NUXT_TELEMETRY_DISABLED=1

COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run generate

# Etapa 2: nginx servindo só os arquivos gerados.
FROM nginx:alpine AS runtime
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/.output/public /usr/share/nginx/html
EXPOSE 80
HEALTHCHECK --interval=30s --timeout=3s CMD wget -qO /dev/null http://127.0.0.1/ || exit 1
