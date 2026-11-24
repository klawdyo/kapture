<script setup lang="ts">
const config = useRuntimeConfig()
const items = [
  { label: 'Recursos', to: '/#recursos' },
  { label: 'Por que é leve', to: '/#leve' },
  { label: 'Instalação', to: '/#instalacao' }
]
</script>

<template>
  <UHeader to="/" :toggle="{ 'aria-label': 'Abrir menu' }">
    <template #title>
      <span class="flex items-center gap-2.5 font-bold tracking-tight">
        <BrandLogo :size="32" />
        <span>Cap</span>
      </span>
    </template>

    <UNavigationMenu :items="items" variant="link" aria-label="Principal" />

    <template #right>
      <UButton class="hidden sm:inline-flex" label="Baixar MSI" :to="config.public.downloadUrl" external />
      <UColorModeButton aria-label="Alternar tema claro e escuro" />
    </template>

    <template #body>
      <UNavigationMenu :items="items" orientation="vertical" aria-label="Principal (móvel)" class="-mx-2.5" />
      <UButton class="mt-4" block label="Baixar MSI" :to="config.public.downloadUrl" external />
    </template>
  </UHeader>
</template>
