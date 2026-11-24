<!-- Gravar (anel vermelho + rótulo) ou, durante a gravação, Parar (quadrado arredondado). -->
<script setup lang="ts">
import Btn from '../ui/Btn.vue'

defineProps<{ rec: boolean }>()
</script>

<template>
  <Btn v-if="rec" kind="rec" round icon="stop" :size="24" title="Parar" />
  <Btn v-else col class="go" title="Gravar">
    <span class="ring"><span class="core" /></span>
    <span class="lb"><i />Gravar</span>
  </Btn>
</template>

<style scoped>
.go { height: auto; padding: 0 4px; gap: 6px; color: var(--fg); }
.go:hover { background: transparent; }
.ring {
  display: grid; place-items: center; width: 42px; height: 42px; border-radius: 50%;
  background: radial-gradient(circle at 50% 35%, #ff7676, #dc2626 70%);
  box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.55), 0 0 14px rgba(239, 68, 68, 0.45), inset 0 0 0 3px rgba(255, 255, 255, 0.22);
}
.go:hover .ring { filter: brightness(1.12); }
.core { width: 16px; height: 16px; border-radius: 50%; background: #ef4444; box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.35); }
.lb { display: inline-flex; align-items: center; gap: 5px; font-size: 12px; font-weight: 600; }
.lb i { width: 8px; height: 8px; border-radius: 50%; background: var(--red); box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.35); }
</style>
