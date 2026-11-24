<!-- Janela da webcam: quadro 1:1 com cantos chanfrados e marcas vermelhas. -->
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
  <Box class="cam">
    <Video class="vid" :src="w.stream" mirror />
    <Box v-if="w.err" class="err">{{ w.err }}</Box>
    <!-- Marcas de canto -->
    <Box class="mk tl" />
    <Box class="mk tr" />
    <Box class="mk bl" />
    <Box class="mk br" />
    <!-- Alça de arrasto -->
    <Box class="grip" data-tauri-drag-region />
  </Box>
</template>

<style scoped>
.cam {
  --c: 16px; position: relative; width: 100vw; height: 100vh; background: #18181b;
  clip-path: polygon(var(--c) 0, calc(100% - var(--c)) 0, 100% var(--c), 100% calc(100% - var(--c)),
    calc(100% - var(--c)) 100%, var(--c) 100%, 0 calc(100% - var(--c)), 0 var(--c));
}
.vid { position: absolute; inset: 0; }
.err { position: absolute; inset: 0; display: grid; place-items: center; color: var(--mut); text-align: center; padding: 16px; }
.mk { position: absolute; width: 18px; height: 18px; border: 0 solid var(--red); pointer-events: none; }
.tl { top: 6px; left: 6px; border-top-width: 3px; border-left-width: 3px; }
.tr { top: 6px; right: 6px; border-top-width: 3px; border-right-width: 3px; }
.bl { bottom: 6px; left: 6px; border-bottom-width: 3px; border-left-width: 3px; }
.br { bottom: 6px; right: 6px; border-bottom-width: 3px; border-right-width: 3px; }
.grip {
  position: absolute; top: 0; left: 0; right: 0; height: 26px; cursor: grab;
  background: linear-gradient(rgba(0, 0, 0, 0.5), transparent); opacity: 0; transition: opacity 0.15s;
}
.cam:hover .grip { opacity: 1; }
</style>
