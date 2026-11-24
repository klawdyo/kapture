<script setup lang="ts">
// Duas versões da logo; o tema escolhe via classe .dark (sem troca no cliente, sem flash).
withDefaults(defineProps<{ size?: 32 | 64 | 320; alt?: string; priority?: boolean }>(), { size: 32, alt: '', priority: false })
// Arquivos existentes: 64 px (cabeçalho, exibido a 32) e 320 px (hero).
const px = (s: number) => (s === 320 ? '' : '-64')
// baseURL termina com barra ('/' ou '/kapture/'): vale para qualquer hospedagem.
const root = useRuntimeConfig().app.baseURL
</script>

<template>
  <span class="inline-flex shrink-0">
    <img :src="`${root}logo-light${px(size)}.png`" :width="size" :height="size" :alt="alt" class="dark:hidden rounded-[22%]" :loading="priority ? 'eager' : 'lazy'" :fetchpriority="priority ? 'high' : 'auto'" decoding="async">
    <img :src="`${root}logo-dark${px(size)}.png`" :width="size" :height="size" alt="" aria-hidden="true" class="hidden dark:block rounded-[22%]" :loading="priority ? 'eager' : 'lazy'" decoding="async">
  </span>
</template>
