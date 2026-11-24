<script setup lang="ts">
import type { NuxtError } from '#app'

const props = defineProps<{ error: NuxtError }>()
const is404 = computed(() => props.error?.statusCode === 404)
useHead({ htmlAttrs: { lang: 'pt-BR' } })
useSeoMeta({ title: is404.value ? 'Página não encontrada, Cap' : 'Algo deu errado, Cap', robots: 'noindex' })
</script>

<template>
  <UApp>
    <AppHeader />
    <UMain id="conteudo">
      <UContainer class="py-24 sm:py-32 text-center">
        <p class="font-mono text-sm text-muted">{{ error.statusCode }}</p>
        <h1 class="mt-3 text-3xl sm:text-4xl font-bold tracking-tight text-highlighted text-balance">
          {{ is404 ? 'Esta página não existe' : 'Algo deu errado' }}
        </h1>
        <p class="mt-4 text-muted text-pretty">
          {{ is404 ? 'O endereço pode ter mudado ou estar digitado errado.' : 'Tente de novo em instantes.' }}
        </p>
        <UButton class="mt-8" size="xl" to="/" label="Voltar ao início" @click="clearError()" />
      </UContainer>
    </UMain>
    <AppFooter />
  </UApp>
</template>
