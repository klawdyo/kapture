<script setup lang="ts">
const config = useRuntimeConfig()
const downloadUrl = config.public.downloadUrl

const LEFT = { title: 'text-start', description: 'text-start max-w-2xl', header: 'max-w-3xl' }

const heroLinks = [
  { label: 'Baixar instalador', to: downloadUrl, external: true, size: 'xl' as const, icon: 'i-lucide-download' },
  { label: 'Ver recursos', to: '#recursos', color: 'neutral' as const, variant: 'outline' as const, size: 'xl' as const }
]

const features = [
  { icon: 'i-lucide-monitor', title: 'Tela, janela ou área', description: 'Grave o monitor inteiro, uma janela específica ou um retângulo desenhado por você.', modes: ['Tela', 'Janela', 'Área'] },
  { icon: 'i-lucide-camera', title: 'Webcam arrastável 1:1', description: 'Uma bolha quadrada com sua câmera, que você posiciona onde quiser sobre a gravação.' },
  { icon: 'i-lucide-mic', title: 'Microfone com medidor', description: 'Veja o nível do áudio em tempo real, antes e durante a gravação.' },
  { icon: 'i-lucide-keyboard', title: 'Atalho global', description: 'Inicie e pare a gravação de qualquer lugar, sem tirar as mãos do teclado.', kbd: true },
  { icon: 'i-lucide-file-text', title: 'Logs em disco', description: 'Cada sessão registra o que aconteceu em arquivo, para você diagnosticar qualquer problema.' }
]

const spec = [
  { k: 'Instalação', v: 'por usuário, sem admin' },
  { k: 'Encoder', v: 'nativo do Windows' },
  { k: 'FFmpeg', v: 'não usa' }
]

const steps = [
  'Baixe o kapture-win-Setup.exe.',
  'Abra o instalador: ele instala em um clique, só para o seu usuário, sem pedir administrador.',
  'Pressione Ctrl+Shift+R para começar a gravar.'
]
</script>

<template>
  <div>
    <UPageHero
      title="Grave a tela. Só isso."
      description="kapture é um gravador de tela leve para Windows: um instalador minúsculo e uma pílula discreta no canto da tela."
      orientation="horizontal"
      :ui="{ title: 'text-balance', description: 'text-pretty' }"
    >
      <template #links>
        <div class="flex flex-col gap-4">
          <div class="flex flex-wrap gap-3">
            <UButton v-for="l in heroLinks" :key="l.label" v-bind="l" />
          </div>
          <p class="text-sm text-muted">Windows 10 e 11 · gratuito · licença MIT</p>
        </div>
      </template>
      <div class="relative mx-auto grid aspect-square w-full max-w-sm place-items-center sm:max-w-md">
        <div aria-hidden="true" class="absolute inset-[6%] rounded-full border border-default" />
        <BrandLogo :size="320" priority alt="Logo do kapture: obturador de câmera com ponto vermelho de gravação" class="w-[72%] [&_img]:size-full [&_img]:drop-shadow-2xl" />
        <RecPill class="absolute bottom-[3%] left-1/2 -translate-x-1/2" />
      </div>
    </UPageHero>

    <UPageSection id="recursos" title="Tudo o que você precisa, nada além" description="Cinco recursos pensados para gravar e mostrar o que importa." :ui="LEFT">
      <ul class="w-full divide-y divide-default border-y border-default">
        <li v-for="f in features" :key="f.title" class="grid gap-2 py-6 md:grid-cols-[minmax(0,18rem)_1fr] md:gap-10">
          <h3 class="flex items-center gap-3 font-semibold text-highlighted">
            <UIcon :name="f.icon" class="size-5 shrink-0 text-muted" aria-hidden="true" />
            {{ f.title }}
          </h3>
          <div class="text-muted">
            <p class="text-pretty">{{ f.description }}</p>
            <p v-if="f.kbd" class="mt-3 flex flex-wrap items-center gap-1.5" aria-label="Atalho: Ctrl, Shift e R">
              <UKbd value="Ctrl" size="lg" /><span aria-hidden="true" class="text-sm">+</span><UKbd value="Shift" size="lg" /><span aria-hidden="true" class="text-sm">+</span><UKbd value="R" size="lg" />
            </p>
            <p v-if="f.modes" class="mt-3 flex flex-wrap gap-2">
              <UBadge v-for="m in f.modes" :key="m" :label="m" color="neutral" variant="outline" size="lg" />
            </p>
          </div>
        </li>
      </ul>
    </UPageSection>

    <UPageSection id="leve" title="Por que é leve" description="kapture usa o encoder nativo do Windows (Media Foundation) para gerar o vídeo. Não embute FFmpeg nem outros binários pesados, e a interface reaproveita o WebView do sistema. O resultado é um instalador pequeno, abertura rápida e gravações que não disputam recursos com o que você está mostrando." orientation="horizontal" :ui="{ description: 'text-pretty' }">
      <dl class="rounded-2xl bg-zinc-900 p-2 text-zinc-50 shadow-xl shadow-zinc-950/15 ring-1 ring-white/10 dark:bg-black dark:ring-white/20">
        <div v-for="r in spec" :key="r.k" class="flex items-baseline justify-between gap-6 px-5 py-4 [&:not(:last-child)]:border-b [&:not(:last-child)]:border-white/10">
          <dt class="text-zinc-300">{{ r.k }}</dt>
          <dd class="font-mono font-semibold tabular-nums">{{ r.v }}</dd>
        </div>
      </dl>
    </UPageSection>

    <UPageSection id="instalacao" title="Instalação" description="Windows 10 ou 11, 64 bits." :ui="LEFT">
      <div class="grid w-full max-w-3xl gap-8">
        <ol class="grid gap-3">
          <li v-for="(text, i) in steps" :key="text" class="flex items-center gap-4 rounded-xl border border-default px-5 py-4">
            <span class="grid size-8 shrink-0 place-items-center rounded-full bg-zinc-900 font-mono text-sm font-semibold text-zinc-50 dark:bg-black dark:ring-1 dark:ring-white/20" aria-hidden="true">{{ i + 1 }}</span>
            <span><span class="sr-only">Passo {{ i + 1 }}: </span>{{ text }}</span>
          </li>
        </ol>
        <UAlert
          color="neutral"
          variant="outline"
          icon="i-lucide-shield-alert"
          title="Aviso do SmartScreen"
          description="O instalador ainda não é assinado digitalmente, então o Windows pode exibir “O Windows protegeu o seu computador”. Clique em Mais informações e depois em Executar assim mesmo."
        />
        <div>
          <UButton size="xl" label="Baixar instalador" :to="downloadUrl" external icon="i-lucide-download" />
        </div>
      </div>
    </UPageSection>
  </div>
</template>
