<!-- Bloco de dispositivo (câmera/microfone): ícone + rótulo (e slot, ex.: VU) sobre o select. -->
<script setup lang="ts">
import { computed } from 'vue'
import type { Dev } from '../domain/types'
import Icon from '../ui/Icon.vue'
import Select from '../ui/Select.vue'
import type { IconName } from '../ui/icons'

const props = defineProps<{ modelValue: string; devs: Dev[]; icon: IconName; label: string; none: string }>()
const emit = defineEmits<{ 'update:modelValue': [id: string] }>()

const opts = computed(() => [
  { value: '', label: props.none },
  ...props.devs.map((d) => ({ value: d.id, label: d.name })),
])
</script>

<template>
  <div class="dev">
    <div class="hd">
      <Icon :name="icon" :size="20" />
      <span class="lb">{{ label }}</span>
      <slot />
    </div>
    <Select :model-value="modelValue" :options="opts" :title="none" @update:model-value="emit('update:modelValue', $event)" />
  </div>
</template>

<style scoped>
.dev { display: flex; flex-direction: column; gap: 7px; flex: 1 1 0; min-width: 0; }
.hd { display: flex; align-items: center; gap: 8px; height: 22px; color: var(--fg); }
.hd svg { flex: none; }
.lb { font-weight: 600; font-size: 13px; }
</style>
