<!-- VU meter do microfone (nível 0..1). -->
<script setup lang="ts">
import Meter from '../ui/Meter.vue'

defineProps<{ value: number }>()
</script>

<template>
  <Meter :value="value" :segs="10" :hot="0.75" />
</template>
