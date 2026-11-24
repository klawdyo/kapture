// Site estático do kapture. `npm run generate` gera .output/public.
// Variáveis NUXT_PUBLIC_* são lidas em tempo de build (o site é estático).
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: false },
  modules: ['@nuxt/ui'],
  css: ['~/assets/css/main.css'],
  ssr: true,
  telemetry: false,

  // Fontes de sistema apenas: nada é baixado em build nem em runtime.
  fonts: {
    families: [
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
    prerender: { crawlLinks: true, routes: ['/', '/404.html'] }
  }
})
