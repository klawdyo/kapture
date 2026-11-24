<!-- Lista suspensa genérica (v-model): largura total, seta e dica com o nome completo. -->
<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { UiOpt } from './icons'

const props = defineProps<{ modelValue: string; options: UiOpt[]; title?: string }>()
const emit = defineEmits<{ 'update:modelValue': [v: string] }>()

/** Nome da opção atual: vira tooltip quando o texto não cabe. */
const cur = computed(() => props.options.find((o) => o.value === props.modelValue)?.label ?? '')
</script>

<template>
  <label class="sel">
    <select
      :value="modelValue"
      :aria-label="title"
      :title="cur"
      @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
    >
      <option v-for="o in options" :key="o.value" :value="o.value">{{ o.label }}</option>
    </select>
    <Icon class="arr" name="chev" :size="16" />
  </label>
</template>

<style scoped>
.sel { position: relative; display: flex; min-width: 0; color: var(--fg); }
select {
  appearance: none; -webkit-appearance: none; width: 100%; min-width: 0; height: 30px;
  border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 7px; padding: 0 26px 0 9px;
  background: var(--fld); color: var(--fg); font: inherit; font-size: 12.5px; outline: none;
  text-overflow: ellipsis; cursor: pointer;
}
select:hover { background: var(--hov); }
select:focus-visible { outline: 2px solid #93c5fd; outline-offset: 1px; }
select option { background: #232c3d; color: var(--fg); }
.arr { position: absolute; right: 7px; top: 7px; pointer-events: none; color: var(--fg); }
</style>
