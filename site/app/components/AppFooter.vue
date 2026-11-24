<script setup lang="ts">
const config = useRuntimeConfig()
</script>

<template>
  <UFooter>
    <template #left>
      <p class="text-sm text-muted">Licença MIT · Cláudio Medeiros</p>
    </template>
    <template #right>
      <a class="link text-sm text-muted hover:text-highlighted inline-flex items-center min-h-11" :href="config.public.repoUrl" rel="noopener">
        Repositório no GitHub (privado)
      </a>
    </template>
  </UFooter>
</template>
