<!-- Seletor de dispositivo (câmera/microfone) com opção "nenhum". -->
<script setup lang="ts">
import { computed } from 'vue'
import type { Dev } from '../domain/types'
import Select from '../ui/Select.vue'
import type { IconName } from '../ui/icons'

const props = defineProps<{ modelValue: string; devs: Dev[]; icon: IconName; none: string }>()
const emit = defineEmits<{ 'update:modelValue': [id: string] }>()

const opts = computed(() => [
  { value: '', label: props.none },
  ...props.devs.map((d) => ({ value: d.id, label: d.name })),
])
</script>

<template>
  <Select :model-value="modelValue" :options="opts" :icon="icon" :title="none" @update:model-value="emit('update:modelValue', $event)" />
</template>
