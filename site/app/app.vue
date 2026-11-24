<script setup lang="ts">
const config = useRuntimeConfig()
const base = String(config.public.siteUrl || '').replace(/\/$/, '')
const title = 'kapture, gravador de tela leve para Windows'
const description = 'Gravador de tela para Windows com menos de 2 MB: grave tela, janela ou área, com webcam e microfone, com atalho Ctrl+Shift+R.'
const ogImage = `${base}/og.png`

useHead({
  link: [{ rel: 'icon', type: 'image/png', href: `${config.app.baseURL}favicon.png` }],
  meta: [
    { name: 'theme-color', content: '#f6f7f9', media: '(prefers-color-scheme: light)' },
    { name: 'theme-color', content: '#15181e', media: '(prefers-color-scheme: dark)' }
  ]
})
useSeoMeta({
  title,
  description,
  ogType: 'website',
  ogLocale: 'pt_BR',
  ogSiteName: 'kapture',
  ogTitle: title,
  ogDescription: description,
  ogUrl: base || undefined,
  ogImage,
  ogImageAlt: 'Logo do kapture: obturador de câmera com ponto vermelho de gravação',
  twitterCard: 'summary',
  twitterImage: ogImage
})
</script>

<template>
  <UApp>
    <a class="skip-link" href="#conteudo">Pular para o conteúdo</a>
    <AppHeader />
    <UMain id="conteudo" tabindex="-1" class="outline-none">
      <NuxtPage />
    </UMain>
    <AppFooter />
  </UApp>
</template>
