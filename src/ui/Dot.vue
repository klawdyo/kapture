<!-- Bolinha genérica: clicável (botão com foco visível) ou, com `inert`, só decorativa dentro de outro botão. Cor e piscar configuráveis. -->
<script setup lang="ts">
withDefaults(
  defineProps<{ label?: string; blink?: boolean; size?: number; expanded?: boolean; inert?: boolean }>(),
  { label: '', size: 10 },
)
</script>

<template>
  <span v-if="inert" class="dot inert" :class="{ blink }" aria-hidden="true" :style="{ '--sz': `${size}px` }"><i /></span>
  <button
    v-else
    class="dot"
    :class="{ blink }"
    type="button"
    :aria-label="label"
    :title="label"
    aria-haspopup="dialog"
    :aria-expanded="expanded ? 'true' : 'false'"
    :style="{ '--sz': `${size}px` }"
  ><i /></button>
</template>

<style scoped>
/* Área de clique maior que a bolinha (24 px), sem aumentar o desenho. */
.dot {
  display: inline-flex; align-items: center; justify-content: center; flex: none;
  width: 24px; height: 24px; padding: 0; border: 0; border-radius: 50%;
  background: transparent; cursor: pointer; color: var(--red);
}
.dot i { display: block; width: var(--sz); height: var(--sz); border-radius: 50%; background: currentColor; }
.dot:hover { background: var(--hov); }
/* Decorativa: tamanho do desenho, sem área de clique própria. */
.dot.inert { width: var(--sz); height: var(--sz); cursor: inherit; }
.dot.inert:hover { background: transparent; }
.dot:focus-visible { outline: 2px solid #93c5fd; outline-offset: 1px; }
.dot.blink i { animation: dot-blink 1.1s ease-in-out infinite; }
@keyframes dot-blink { 50% { opacity: 0.25; } }
/* Sem animação para quem prefere menos movimento: bolinha estática. */
@media (prefers-reduced-motion: reduce) { .dot.blink i { animation: none; } }
</style>
