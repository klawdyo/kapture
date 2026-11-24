<!-- Janela da webcam: moldura 1:1 chanfrada, borda clara, sombra, alça e marcas vermelhas. -->
<script setup lang="ts">
import { onBeforeUnmount, onMounted, reactive } from 'vue'
import { useWebcam } from '../composables/useWebcam'
import Box from '../ui/Box.vue'
import Video from '../ui/Video.vue'

const w = reactive(useWebcam())
onMounted(w.init)
onBeforeUnmount(w.dispose)
</script>

<template>
  <Box class="cam" data-tauri-drag-region>
    <Box class="edge" data-tauri-drag-region />
    <Box class="frame" data-tauri-drag-region />
    <Video class="vid" :src="w.stream" mirror />
    <Box v-if="w.err" class="err">{{ w.err }}</Box>
    <!-- Cantos em L (sobre o vídeo) -->
    <Box class="br tl" />
    <Box class="br tr" />
    <Box class="br bl" />
    <Box class="br rb" />
    <!-- Marcas vermelhas no meio de três lados -->
    <Box class="mk l" />
    <Box class="mk r" />
    <Box class="mk b" />
    <!-- Alça de arrasto (visível) -->
    <Box class="grip" data-tauri-drag-region><i /><i /></Box>
  </Box>
</template>

<style scoped>
/* A janela tem 10 px de folga transparente para a sombra; o quadro fica dentro. */
.cam {
  --m: 10px; --c: 12px; position: relative; width: 100vw; height: 100vh;
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.45));
}
.edge, .frame {
  position: absolute; inset: var(--m);
  clip-path: polygon(var(--c) 0, calc(100% - var(--c)) 0, 100% var(--c), 100% calc(100% - var(--c)),
    calc(100% - var(--c)) 100%, var(--c) 100%, 0 calc(100% - var(--c)), 0 var(--c));
}
.edge { background: rgba(226, 232, 240, 0.7); }
.frame {
  inset: calc(var(--m) + 1px); --c: 11px;
  background: linear-gradient(160deg, #36425c, #232b3d 60%, #2a3348);
}
.vid { position: absolute; top: calc(var(--m) + 12px); left: calc(var(--m) + 12px); width: calc(100% - 2 * (var(--m) + 12px)); height: calc(100% - 2 * (var(--m) + 12px)); border-radius: 3px; overflow: hidden; }
.err { position: absolute; inset: calc(var(--m) + 12px); display: grid; place-items: center; color: var(--mut); text-align: center; padding: 8px; font-size: 12px; }
.br { position: absolute; width: 22px; height: 22px; border: 0 solid #b6c0d0; pointer-events: none; filter: drop-shadow(0 1px 1px rgba(0,0,0,.35)); }
.tl { top: calc(var(--m) + 16px); left: calc(var(--m) + 16px); border-top-width: 6px; border-left-width: 6px; border-top-left-radius: 6px; }
.tr { top: calc(var(--m) + 16px); right: calc(var(--m) + 16px); border-top-width: 6px; border-right-width: 6px; border-top-right-radius: 6px; }
.bl { bottom: calc(var(--m) + 16px); left: calc(var(--m) + 16px); border-bottom-width: 6px; border-left-width: 6px; border-bottom-left-radius: 6px; }
.rb { bottom: calc(var(--m) + 16px); right: calc(var(--m) + 16px); border-bottom-width: 6px; border-right-width: 6px; border-bottom-right-radius: 6px; }
.mk { position: absolute; background: var(--red); border-radius: 2px; pointer-events: none; }
.l, .r { top: 50%; width: 4px; height: 30px; margin-top: -15px; }
.l { left: calc(var(--m) + 4px); }
.r { right: calc(var(--m) + 4px); }
.b { bottom: calc(var(--m) + 4px); left: 50%; width: 30px; height: 4px; margin-left: -15px; }
.grip {
  position: absolute; top: calc(var(--m) + 3px); left: 50%; width: 44px; height: 9px; margin-left: -22px;
  display: flex; flex-direction: column; justify-content: space-between; cursor: grab;
}
.grip i { display: block; height: 2px; border-radius: 1px; background: #cbd5e1; }
</style>
