<!-- Botão genérico: ícone opcional + conteúdo; variantes por props. -->
<script setup lang="ts">
import Icon from './Icon.vue'
import type { IconName } from './icons'

withDefaults(
  defineProps<{ icon?: IconName; kind?: 'flat' | 'rec'; on?: boolean; round?: boolean }>(),
  { kind: 'flat' },
)
</script>

<template>
  <button class="btn" :class="[kind, { on, round }]" type="button">
    <Icon v-if="icon" :name="icon" />
    <slot />
  </button>
</template>

<style scoped>
.btn {
  display: inline-flex; align-items: center; justify-content: center; gap: 6px;
  height: 32px; padding: 0 10px; border: 0; border-radius: 10px;
  background: transparent; color: var(--mut); font: inherit; cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.btn:hover { background: var(--hov); color: var(--fg); }
.btn.on { background: var(--hov); color: var(--fg); }
.btn.round { width: 32px; padding: 0; border-radius: 50%; }
.btn.rec { width: 38px; height: 38px; background: var(--red); color: #fff; }
.btn.rec:hover { background: var(--red2); }
</style>
