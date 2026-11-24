<!-- Vídeo genérico ligado a um MediaStream; opcionalmente espelhado. -->
<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'

const props = defineProps<{ src: MediaStream | null; mirror?: boolean }>()
const el = ref<HTMLVideoElement>()
const set = (): void => { el.value!.srcObject = props.src }
onMounted(set)
watch(() => props.src, set)
</script>

<template>
  <video ref="el" autoplay muted playsinline :class="{ mirror }" />
</template>

<style scoped>
video { width: 100%; height: 100%; object-fit: cover; display: block; }
.mirror { transform: scaleX(-1); }
</style>
