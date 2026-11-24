<!-- Seletor de modo de captura: Tela / Janela / Área. -->
<script setup lang="ts">
import type { Mode } from '../domain/types'
import Segmented from '../ui/Segmented.vue'
import type { UiOpt } from '../ui/icons'

defineProps<{ modelValue: Mode }>()
const emit = defineEmits<{ 'update:modelValue': [m: Mode] }>()

const OPTS: UiOpt[] = [
  { value: 'screen', label: 'Tela', icon: 'screen' },
  { value: 'window', label: 'Janela', icon: 'window' },
  { value: 'area', label: 'Área', icon: 'area' },
]
</script>

<template>
  <Segmented :model-value="modelValue" :options="OPTS" @update:model-value="emit('update:modelValue', $event as Mode)" />
</template>
