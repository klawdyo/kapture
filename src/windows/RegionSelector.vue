<!-- Overlay de seleção de área: desenhe, mova, redimensione; Enter confirma, Esc cancela. -->
<script setup lang="ts">
import { onBeforeUnmount, onMounted, reactive } from 'vue'
import { useRegion } from '../composables/useRegion'
import { HANDLES } from '../domain/rect'
import Box from '../ui/Box.vue'
import Btn from '../ui/Btn.vue'

const r = reactive(useRegion())
onMounted(r.init)
onBeforeUnmount(r.dispose)
</script>

<template>
  <Box
    class="ov"
    :style="{ cursor: r.cur }"
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
      <Box v-for="h in HANDLES" :key="h" class="hh" :class="[h, h.length === 2 ? 'cn' : 'hd', { act: r.act === h }]" />
      <Box class="low" :class="{ in: !r.below }" @mousedown.stop>
        <!-- Etiqueta informativa (só mostra o tamanho) e, ao lado, os botões de ação. -->
        <Box class="tag">{{ r.tag }}</Box>
        <Btn class="act go" @click="r.go()">Gravar nesta área</Btn>
        <Btn class="act" @click="r.cancel()">Cancelar</Btn>
      </Box>
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
.nw { top: -3px; left: -3px; border-top-width: 4px; border-left-width: 4px; border-top-left-radius: 9px; }
.ne { top: -3px; right: -3px; border-top-width: 4px; border-right-width: 4px; border-top-right-radius: 9px; }
.sw { bottom: -3px; left: -3px; border-bottom-width: 4px; border-left-width: 4px; border-bottom-left-radius: 9px; }
.se { bottom: -3px; right: -3px; border-bottom-width: 4px; border-right-width: 4px; border-bottom-right-radius: 9px; }
.hd { background: #334155; border-radius: 3px; }
.n, .s { left: 50%; width: 22px; height: 5px; margin-left: -11px; }
.n { top: -3px; }
.s { bottom: -3px; }
.w, .e { top: 50%; width: 5px; height: 22px; margin-top: -11px; }
.w { left: -3px; }
.e { right: -3px; }
/* Alça em uso: vermelho vivo (indicador ativo da marca), para ficar claro qual lado está sendo ajustado. */
.cn.act { border-color: var(--red); }
.hd.act { background: var(--red); }
/* Faixa abaixo do retângulo: etiqueta de tamanho e botões lado a lado. */
.low {
  position: absolute; left: 50%; top: calc(100% + 12px); transform: translateX(-50%);
  display: flex; align-items: center; gap: 8px; white-space: nowrap; cursor: default;
}
/* Sem espaço abaixo (retângulo na borda da tela): a faixa entra pelo lado de dentro. */
.low.in { top: auto; bottom: 12px; }
/* Etiqueta informativa: chapada, sem borda nem hover, para não parecer botão. */
.tag {
  padding: 3px 10px; border-radius: 6px; background: #1e293b; color: #f1f5f9; font-size: 12px;
  font-variant-numeric: tabular-nums; pointer-events: none;
}
/* Botões de ação: borda clara, fundo mais claro que a etiqueta e hover (o Btn genérico traz o resto). */
.act { height: 30px; padding: 0 14px; border: 1px solid rgba(255, 255, 255, 0.35); background: #334155; color: #f8fafc; font-size: 12.5px; box-shadow: 0 2px 6px rgba(0, 0, 0, 0.35); }
.act:hover { background: #475569; }
.act.go { background: var(--red); border-color: var(--red); color: #fff; font-weight: 600; }
.act.go:hover { background: var(--red2); }
.err { position: absolute; bottom: 24px; left: 50%; transform: translateX(-50%); padding: 4px 10px; border-radius: 6px; background: #1e293b; color: #fca5a5; }
</style>
