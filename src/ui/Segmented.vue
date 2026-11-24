<!-- Controle segmentado genérico (v-model), compõe Btn. -->
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
      :icon="o.icon"
      :on="o.value === modelValue"
      @click="emit('update:modelValue', o.value)"
    >{{ o.label }}</Btn>
  </div>
</template>

<style scoped>
.seg { display: inline-flex; gap: 2px; padding: 2px; border-radius: 12px; background: rgba(255, 255, 255, 0.05); }
</style>
