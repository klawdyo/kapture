<!-- Controle segmentado genérico (v-model): ícone em cima, rótulo embaixo. -->
<script setup lang="ts">
import Btn from './Btn.vue'
import type { UiOpt } from './icons'

defineProps<{ modelValue: string; options: UiOpt[] }>()
const emit = defineEmits<{ 'update:modelValue': [v: string] }>()
</script>

<template>
  <div class="seg" role="group">
    <Btn
      v-for="o in options"
      :key="o.value"
      col
      :icon="o.icon"
      :size="22"
      :on="o.value === modelValue"
      @click="emit('update:modelValue', o.value)"
    >{{ o.label }}</Btn>
  </div>
</template>

<style scoped>
.seg {
  display: inline-flex; gap: 2px; padding: 3px; border-radius: 10px;
  background: rgba(255, 255, 255, 0.06); border: 1px solid rgba(255, 255, 255, 0.08);
}
.seg :deep(.btn) { width: 48px; height: 56px; padding: 0; font-size: 12px; }
.seg :deep(.btn.on) { background: rgba(255, 255, 255, 0.16); box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.12); }
</style>
