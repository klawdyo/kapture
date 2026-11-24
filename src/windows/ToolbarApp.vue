<!-- Barra de ferramentas (pílula). Sem lógica: tudo vem de useToolbar. -->
<script setup lang="ts">
import { onBeforeUnmount, onMounted, reactive } from 'vue'
import DevSelect from '../components/DevSelect.vue'
import ModeSwitch from '../components/ModeSwitch.vue'
import RecBtn from '../components/RecBtn.vue'
import VuMeter from '../components/VuMeter.vue'
import { useToolbar } from '../composables/useToolbar'
import { FPS } from '../domain/cfg'
import Box from '../ui/Box.vue'
import Btn from '../ui/Btn.vue'
import Icon from '../ui/Icon.vue'
import Select from '../ui/Select.vue'

const t = reactive(useToolbar())
onMounted(t.init)
onBeforeUnmount(t.dispose)
</script>

<template>
  <Box class="bar" data-tauri-drag-region>
    <!-- Modo compacto durante a gravação -->
    <template v-if="t.active">
      <Box class="dot" :class="{ off: t.paused }" />
      <Box tag="span" class="clock">{{ t.clock }}</Box>
      <Btn round :icon="t.paused ? 'play' : 'pause'" :title="t.paused ? 'Retomar' : 'Pausar'" @click="t.pause()" />
      <RecBtn :rec="true" @click="t.toggle()" />
    </template>
    <template v-else>
      <Icon name="logo" :size="22" class="logo" />
      <ModeSwitch :model-value="t.mode" @update:model-value="t.setMode" />
      <Select v-if="t.opts.length" v-model="t.tid" :options="t.opts" title="Alvo" />
      <DevSelect :model-value="t.cam" :devs="t.devs.cams" icon="cam" none="Sem câmera" @update:model-value="t.setCam" />
      <DevSelect :model-value="t.mic" :devs="t.devs.mics" icon="mic" none="Sem microfone" @update:model-value="t.setMic" />
      <VuMeter :value="t.lvl" />
      <template v-if="t.cfg">
        <Btn icon="sys" :on="t.sys" title="Áudio do sistema" @click="t.toggleSys()" />
        <Select v-model="t.fps" :options="FPS" title="Quadros por segundo" />
      </template>
      <RecBtn :rec="false" @click="t.toggle()" />
      <Btn round icon="gear" :on="t.cfg" title="Configurações" @click="t.toggleCfg()" />
    </template>
    <Box v-if="t.err" tag="span" class="err">{{ t.err }}</Box>
    <Box v-else-if="t.saved" tag="span" class="ok" :title="t.saved">Salvo</Box>
  </Box>
</template>

<style scoped>
.bar {
  display: flex; align-items: center; gap: 8px; width: max-content; margin: 6px auto;
  padding: 6px 12px; border-radius: 999px; background: var(--bg); border: 1px solid var(--bd);
  backdrop-filter: blur(14px); box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
}
.logo { color: var(--red); }
.dot { width: 10px; height: 10px; border-radius: 50%; background: var(--red); animation: blink 1.2s infinite; }
.dot.off { animation: none; background: var(--mut); }
.clock { min-width: 48px; font-variant-numeric: tabular-nums; font-weight: 600; }
.err { color: var(--red); max-width: 180px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ok { color: var(--mut); }
@keyframes blink { 50% { opacity: 0.3; } }
</style>
