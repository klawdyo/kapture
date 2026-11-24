<!-- Aviso de atualização: bolinha piscando que abre uma mensagem com Baixar/Depois. Sem lógica: tudo vem por props e eventos. -->
<script setup lang="ts">
import type { Upd } from '../domain/types'
import Box from '../ui/Box.vue'
import Btn from '../ui/Btn.vue'
import Dot from '../ui/Dot.vue'
import Pop from '../ui/Pop.vue'

defineProps<{ upd: Upd; cur: string; open: boolean }>()
defineEmits<{ toggle: []; download: []; later: [] }>()
</script>

<template>
  <Dot label="Atualização disponível" blink :expanded="open" @click="$emit('toggle')" />
  <Pop v-if="open" label="Atualização disponível" @close="$emit('later')">
    <Box class="ttl">Atualização disponível</Box>
    <Box class="ver">
      <span>Instalada: <b>v{{ cur }}</b></span>
      <span>Nova: <b>v{{ upd.version }}</b></span>
    </Box>
    <Box class="act">
      <Btn class="go" @click="$emit('download')">Baixar</Btn>
      <Btn @click="$emit('later')">Depois</Btn>
    </Box>
  </Pop>
</template>

<style scoped>
.ttl { font-weight: 600; font-size: 13px; margin-bottom: 4px; }
.ver { display: flex; gap: 16px; font-size: 12px; color: var(--mut); line-height: 1.4; }
.ver b { color: var(--fg); font-weight: 600; }
.act { display: flex; gap: 8px; justify-content: flex-end; margin-top: 6px; }
.act :deep(.btn) { height: 28px; border: 1px solid rgba(255, 255, 255, 0.12); background: var(--fld); color: var(--fg); }
.act :deep(.btn:hover) { background: var(--hov); }
.act :deep(.btn.go) { background: var(--fg); border-color: var(--fg); color: #1a2130; font-weight: 600; }
.act :deep(.btn.go:hover) { background: var(--mut); border-color: var(--mut); color: #1a2130; }
</style>
