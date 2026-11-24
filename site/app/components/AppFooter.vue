<script setup lang="ts">
const config = useRuntimeConfig()
</script>

<template>
  <UFooter>
    <template #left>
      <p class="flex flex-wrap items-baseline gap-x-3 text-sm text-muted">
        <span class="wordmark">kapture</span>
        <span>Licença MIT · Cláudio Medeiros</span>
      </p>
    </template>
    <template #right>
      <a class="link text-sm text-muted hover:text-highlighted inline-flex items-center min-h-11" :href="config.public.repoUrl" rel="noopener">
        Repositório no GitHub
      </a>
    </template>
  </UFooter>
</template>
