<!-- VU meter do microfone (nível 0..1), 8 tracinhos. -->
<script setup lang="ts">
import Meter from '../ui/Meter.vue'

defineProps<{ value: number }>()
</script>

<template>
  <Meter data-tauri-drag-region :value="value" :segs="8" :hot="0.85" />
</template>
