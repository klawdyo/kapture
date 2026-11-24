<!-- Botão genérico: ícone opcional + conteúdo; variantes por props. -->
<script setup lang="ts">
import Icon from './Icon.vue'
import type { IconName } from './icons'

withDefaults(
  defineProps<{ icon?: IconName; kind?: 'flat' | 'rec'; on?: boolean; round?: boolean; col?: boolean; size?: number }>(),
  { kind: 'flat', size: 18 },
)
</script>

<template>
  <button class="btn" :class="[kind, { on, round, col }]" type="button">
    <Icon v-if="icon" :name="icon" :size="size" />
    <slot />
  </button>
</template>

<style scoped>
.btn {
  display: inline-flex; align-items: center; justify-content: center; gap: 6px;
  height: 32px; padding: 0 10px; border: 0; border-radius: 8px;
  background: transparent; color: var(--mut); font: inherit; cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.btn:hover { background: var(--hov); color: var(--fg); }
.btn:focus-visible { outline: 2px solid #93c5fd; outline-offset: 1px; }
.btn.on { background: var(--hov); color: var(--fg); }
.btn.round { width: 32px; padding: 0; border-radius: 50%; }
.btn.col { flex-direction: column; gap: 5px; height: auto; }
.btn.rec { width: 36px; height: 36px; background: var(--red); color: #fff; }
.btn.rec:hover { background: var(--red2); }
</style>
