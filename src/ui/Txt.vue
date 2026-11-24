<!-- Texto fino e discreto (cinza, 11 px, sem negrito). Com `live`, é uma região anunciada pelo leitor de tela sem interromper. -->
<script setup lang="ts">
defineProps<{ live?: boolean }>()
</script>

<template>
  <span class="txt" :role="live ? 'status' : undefined" :aria-live="live ? 'polite' : undefined"><slot /></span>
</template>

<style scoped>
.txt { display: inline-flex; align-items: center; gap: 4px; flex: none; font-size: 11px; font-weight: 400; line-height: 1; color: var(--mut); white-space: nowrap; font-variant-numeric: tabular-nums; }
</style>
