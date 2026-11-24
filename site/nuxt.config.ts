// Site estático do kapture. `npm run generate` gera .output/public.
// Variáveis NUXT_PUBLIC_* são lidas em tempo de build (o site é estático).
import { fileURLToPath } from 'node:url'

const appSrc = fileURLToPath(new URL('../src', import.meta.url))

export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: false },
  modules: ['@nuxt/ui'],
  css: ['~/assets/css/main.css'],
  ssr: true,
  telemetry: false,

  // Compartilha os componentes Vue puros do app (src/ui e src/components) com o site.
  alias: {
    '@app': appSrc
  },
  vite: {
    server: {
      fs: {
        allow: [appSrc]
      }
    }
  },

  // Fontes locais e de sistema: nada é baixado em build nem em runtime.
  fonts: {
    families: [
      { name: 'Rubik', provider: 'none' },
      { name: 'Segoe UI', provider: 'none' },
      { name: 'Roboto', provider: 'none' },
      { name: 'Helvetica Neue', provider: 'none' },
      { name: 'Arial', provider: 'none' },
      { name: 'Cascadia Mono', provider: 'none' },
      { name: 'SF Mono', provider: 'none' },
      { name: 'Consolas', provider: 'none' }
    ]
  },

  // Ícones embutidos no bundle (sem chamadas à API do Iconify em runtime).
  icon: { provider: 'none', clientBundle: { scan: true, sizeLimitKb: 64 }, fallbackToApi: false },

  colorMode: { preference: 'system', fallback: 'light' },

  runtimeConfig: {
    public: {
      // NUXT_PUBLIC_DOWNLOAD_URL: o repositório é público, então o padrão abaixo
      // leva ao último release. Pode apontar para outra URL do .msi se quiser.
      downloadUrl: 'https://github.com/klawdyo/kapture/releases/latest',
      // NUXT_PUBLIC_SITE_URL (opcional): URL pública do site, usada nas tags og:*.
      siteUrl: '',
      repoUrl: 'https://github.com/klawdyo/kapture'
    }
  },

  app: {
    head: {
      htmlAttrs: { lang: 'pt-BR' }
    }
  },

  nitro: {
    prerender: { crawlLinks: true, routes: ['/', '/privacidade', '/404.html'] }
  }
})
