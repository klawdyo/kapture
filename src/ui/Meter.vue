<!-- Medidor segmentado genérico (valor 0..1); passa de "hot" fica vermelho. -->
<script setup lang="ts">
withDefaults(defineProps<{ value: number; segs?: number; hot?: number }>(), { segs: 10, hot: 0.75 })
</script>

<template>
  <div class="meter" role="meter" :aria-valuenow="value">
    <i v-for="i in segs" :key="i" :class="{ on: i <= Math.round(value * segs), hot: i / segs > hot }" />
  </div>
</template>

<style scoped>
.meter { display: inline-flex; align-items: center; gap: 2px; height: 12px; }
i { pointer-events: none; width: 3px; height: 100%; border-radius: 1px; background: rgba(255, 255, 255, 0.2); }
i.on { background: var(--ok); }
i.on.hot { background: var(--red); }
</style>
