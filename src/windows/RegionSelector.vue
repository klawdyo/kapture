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
      <Box class="tag">{{ r.tag }}</Box>
    </Box>
    <Box v-if="r.err" class="err">{{ r.err }}</Box>
  </Box>
</template>

<style scoped>
.ov { position: fixed; inset: 0; cursor: crosshair; background: rgba(0, 0, 0, 0.35); }
.hint {
  position: absolute; top: 40%; left: 50%; transform: translateX(-50%); padding: 8px 14px;
  border-radius: 999px; background: var(--bg); color: var(--fg);
}
.sel { position: absolute; border: 1.5px dashed #fff; box-shadow: 0 0 0 100vmax rgba(0, 0, 0, 0.45); background: transparent; }
.cn { position: absolute; width: 14px; height: 14px; border: 0 solid var(--red); }
.tl { top: -2px; left: -2px; border-top-width: 3px; border-left-width: 3px; }
.tr { top: -2px; right: -2px; border-top-width: 3px; border-right-width: 3px; }
.bl { bottom: -2px; left: -2px; border-bottom-width: 3px; border-left-width: 3px; }
.br { bottom: -2px; right: -2px; border-bottom-width: 3px; border-right-width: 3px; }
.tag {
  position: absolute; left: 0; top: calc(100% + 6px); padding: 2px 8px; border-radius: 6px;
  background: var(--bg); color: var(--fg); font-variant-numeric: tabular-nums; white-space: nowrap;
}
.err { position: absolute; bottom: 24px; left: 50%; transform: translateX(-50%); color: var(--red); }
</style>
