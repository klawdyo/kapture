<!-- Botão com aparência de texto fino (cinza, 11 px); foco visível. O conteúdo vem pelo slot. -->
<script setup lang="ts">
defineEmits<{ click: [] }>()
</script>

<template>
  <button class="tb" type="button" @click="$emit('click')"><slot /></button>
</template>

<style scoped>
.tb {
  display: inline-flex; align-items: center; gap: 5px; flex: none; height: 24px; padding: 0 6px; border: 0; border-radius: 6px;
  background: transparent; color: var(--mut); font: inherit; font-size: 11px; font-weight: 400; line-height: 1; white-space: nowrap; cursor: pointer;
}
.tb:hover { background: var(--hov); color: var(--fg); }
.tb:focus-visible { outline: 2px solid #93c5fd; outline-offset: 1px; }
</style>
