<script setup lang="ts">
const config = useRuntimeConfig()
const base = String(config.public.siteUrl || '').replace(/\/$/, '')
const title = 'Cap, gravador de tela leve para Windows'
const description = 'Grave tela, janela ou área, com webcam e microfone, em um instalador de cerca de 1,5 MB. Atalho Ctrl+Shift+R e encoder nativo do Windows, sem FFmpeg.'
const ogImage = `${base}/og.png`

useHead({
  meta: [
    { name: 'theme-color', content: '#fafafa', media: '(prefers-color-scheme: light)' },
    { name: 'theme-color', content: '#09090b', media: '(prefers-color-scheme: dark)' }
  ]
})
useSeoMeta({
  title,
  description,
  ogType: 'website',
  ogLocale: 'pt_BR',
  ogSiteName: 'Cap',
  ogTitle: title,
  ogDescription: description,
  ogUrl: base || undefined,
  ogImage,
  ogImageAlt: 'Logo do Cap: obturador de câmera com ponto vermelho de gravação',
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
