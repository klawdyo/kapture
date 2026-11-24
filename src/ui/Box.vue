<!-- Contêiner genérico (div por padrão); troca a tag via prop. -->
<script setup lang="ts">
withDefaults(defineProps<{ tag?: string }>(), { tag: 'div' })
</script>

<template>
  <component :is="tag"><slot /></component>
</template>
