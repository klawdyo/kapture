<!-- Status discreto da atualização na barra de título. Sem lógica: fase e texto vêm prontos por props. -->
<script setup lang="ts">
import type { Phase } from '../domain/upd'
import type { Upd } from '../domain/types'
import Dot from '../ui/Dot.vue'
import Txt from '../ui/Txt.vue'
import TxtBtn from '../ui/TxtBtn.vue'
import UpdateDot from './UpdateDot.vue'

defineProps<{ phase: Phase; text: string; upd: Upd | null; cur: string; open: boolean }>()
defineEmits<{ apply: []; toggle: []; download: []; later: [] }>()
</script>

<template>
  <span class="us" data-tauri-drag-region>
    <!-- Região sempre presente: o leitor de tela anuncia cada troca de texto. -->
    <Txt live data-tauri-drag-region>
      <TxtBtn v-if="phase === 'ready'" @click="$emit('apply')">
        <Dot inert blink :size="8" />{{ text }}
      </TxtBtn>
      <template v-else>{{ text }}</template>
    </Txt>
    <!-- Fallback (sem instalador): bolinha com a mensagem Baixar/Depois. -->
    <UpdateDot v-if="phase === 'available' && upd" :upd="upd" :cur="cur" :open="open" @toggle="$emit('toggle')" @download="$emit('download')" @later="$emit('later')" />
  </span>
</template>

<style scoped>
.us { display: inline-flex; align-items: center; gap: 2px; flex: none; }
</style>
