<!-- Lista suspensa genérica com ícone opcional (v-model). -->
<script setup lang="ts">
import Icon from './Icon.vue'
import type { IconName, UiOpt } from './icons'

defineProps<{ modelValue: string; options: UiOpt[]; icon?: IconName; title?: string }>()
const emit = defineEmits<{ 'update:modelValue': [v: string] }>()
</script>

<template>
  <label class="sel" :title="title">
    <Icon v-if="icon" :name="icon" :size="16" />
    <select :value="modelValue" @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)">
      <option v-for="o in options" :key="o.value" :value="o.value">{{ o.label }}</option>
    </select>
  </label>
</template>

<style scoped>
.sel { display: inline-flex; align-items: center; gap: 6px; color: var(--mut); }
select {
  max-width: 130px; height: 28px; border: 0; border-radius: 8px; padding: 0 6px;
  background: rgba(255, 255, 255, 0.06); color: var(--fg); font: inherit; outline: none;
}
select option { background: #27272a; color: var(--fg); }
</style>
