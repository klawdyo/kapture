<script setup lang="ts">
// Exibe a arte clara ou escura conforme a classe .dark, sem piscar no carregamento.
withDefaults(
  defineProps<{
    light: string
    dark: string
    alt: string
    width: number
    height: number
    priority?: boolean
    imgClass?: string
  }>(),
  { priority: false, imgClass: '' }
)

// Prefixo baseURL garante caminhos válidos em qualquer subdiretório de hospedagem.
const root = useRuntimeConfig().app.baseURL
</script>

<template>
  <span class="block">
    <img
      :src="`${root}${light}`"
      :width="width"
      :height="height"
      :alt="alt"
      :class="['block w-full h-auto dark:hidden', imgClass]"
      :loading="priority ? 'eager' : 'lazy'"
      :fetchpriority="priority ? 'high' : 'auto'"
      decoding="async"
    >
    <img
      :src="`${root}${dark}`"
      :width="width"
      :height="height"
      alt=""
      aria-hidden="true"
      :class="['hidden w-full h-auto dark:block', imgClass]"
      :loading="priority ? 'eager' : 'lazy'"
      :fetchpriority="priority ? 'high' : 'auto'"
      decoding="async"
    >
  </span>
</template>
