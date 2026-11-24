<!-- Overlay de seleção de área: arraste, Enter confirma, Esc cancela. -->
<script setup lang="ts">
import { onBeforeUnmount, onMounted, reactive } from 'vue'
import { useRegion } from '../composables/useRegion'
import Box from '../ui/Box.vue'

const r = reactive(useRegion())
onMounted(r.init)
onBeforeUnmount(r.dispose)
</script>

<template>
  <Box
    class="ov"
    @mousedown="r.down($event.clientX, $event.clientY)"
    @mousemove="r.move($event.clientX, $event.clientY)"
    @mouseup="r.up()"
  >
    <Box v-if="!r.shown" class="hint">Arraste para escolher a área · Enter confirma · Esc cancela</Box>
    <Box
      v-else
      class="sel"
      :style="{ left: r.css.x + 'px', top: r.css.y + 'px', width: r.css.w + 'px', height: r.css.h + 'px' }"
    >
      <Box class="cn tl" />
      <Box class="cn tr" />
      <Box class="cn bl" />
      <Box class="cn br" />
      <Box class="hd t" />
      <Box class="hd b" />
      <Box class="hd l" />
      <Box class="hd r" />
      <Box class="tag">{{ r.tag }}</Box>
    </Box>
    <Box v-if="r.err" class="err">{{ r.err }}</Box>
  </Box>
</template>

<style scoped>
/* Fundo praticamente invisível (1%): não escurece a tela, mas captura o mouse. */
.ov { position: fixed; inset: 0; cursor: crosshair; background: rgba(0, 0, 0, 0.01); }
.hint {
  position: absolute; bottom: 72px; left: 50%; transform: translateX(-50%); padding: 6px 14px;
  border-radius: 8px; background: rgba(30, 38, 54, 0.88); color: var(--fg); font-size: 12px;
}
.sel { position: absolute; box-sizing: border-box; border: 2px dashed #475569; }
/* Cantos em L e alças: grafite com contorno branco (legíveis em fundo claro e escuro). */
.cn, .hd { position: absolute; border: 0 solid #334155; filter: drop-shadow(0 0 1px #fff) drop-shadow(0 0 1px #fff); }
.cn { width: 22px; height: 22px; }
.tl { top: -3px; left: -3px; border-top-width: 4px; border-left-width: 4px; border-top-left-radius: 9px; }
.tr { top: -3px; right: -3px; border-top-width: 4px; border-right-width: 4px; border-top-right-radius: 9px; }
.bl { bottom: -3px; left: -3px; border-bottom-width: 4px; border-left-width: 4px; border-bottom-left-radius: 9px; }
.br { bottom: -3px; right: -3px; border-bottom-width: 4px; border-right-width: 4px; border-bottom-right-radius: 9px; }
.hd { background: #334155; border-radius: 3px; }
.t, .b { left: 50%; width: 22px; height: 5px; margin-left: -11px; }
.t { top: -3px; }
.b { bottom: -3px; }
.l, .r { top: 50%; width: 5px; height: 22px; margin-top: -11px; }
.l { left: -3px; }
.r { right: -3px; }
.tag {
  position: absolute; left: 50%; top: calc(100% + 12px); transform: translateX(-50%); padding: 3px 10px;
  border-radius: 6px; background: #1e293b; color: #f1f5f9; font-size: 12px;
  font-variant-numeric: tabular-nums; white-space: nowrap;
}
.err { position: absolute; bottom: 24px; left: 50%; transform: translateX(-50%); padding: 4px 10px; border-radius: 6px; background: #1e293b; color: #fca5a5; }
</style>
