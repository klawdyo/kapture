<!-- Mensagem flutuante genérica (diálogo pequeno) ancorada ao canto superior direito do contêiner posicionado. -->
<script setup lang="ts">
defineProps<{ label: string }>()
const emit = defineEmits<{ close: [] }>()
</script>

<template>
  <div class="pop" role="dialog" :aria-label="label" @keydown.esc="emit('close')">
    <slot />
  </div>
</template>

<style scoped>
.pop {
  position: absolute; top: 32px; right: 10px; z-index: 10; box-sizing: border-box; width: 290px;
  padding: 8px 12px; border-radius: 10px; background: #1e2638; border: 1px solid var(--bd);
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.45);
}
</style>
