<script setup lang="ts">
// Página inicial do kapture: hero integrado com a demonstração viva do app, recursos e instalação.
const config = useRuntimeConfig()
const downloadUrl = config.public.downloadUrl

// Recursos em linguagem direta, sem jargão técnico.
const features = [
  {
    title: 'Tela, janela ou área',
    detail: 'Grave o monitor inteiro, uma janela específica ou apenas um retângulo escolhido por você.'
  },
  {
    title: 'Webcam arrastável',
    detail: 'Mostre sua câmera em um quadrado limpo e arraste para o canto que preferir.'
  },
  {
    title: 'Microfone com medidor',
    detail: 'Confira se o som está chegando antes de começar e enquanto grava.'
  },
  {
    title: 'Atalho Ctrl+Shift+R',
    detail: 'Comece e termine a gravação de qualquer janela, sem tirar as mãos do teclado.'
  },
  {
    title: 'Leve de verdade',
    detail: 'Instalador com menos de 2 MB, abre rápido e não pesa enquanto você grava.'
  }
]

// Passos objetivos para baixar e usar.
const installSteps = [
  {
    title: 'Baixe o instalador',
    body: 'Faça o download do kapture-win-Setup.exe.'
  },
  {
    title: 'Instale em um clique',
    body: 'A instalação roda só para o seu usuário, sem pedir administrador.'
  },
  {
    title: 'Comece a gravar',
    body: 'Clique em Gravar ou pressione Ctrl+Shift+R.'
  }
]
</script>

<template>
  <div>
    <!-- Hero com grade sutil da marca, pôster em telas grandes e a barra interativa integrada logo abaixo. -->
    <section class="brand-grid border-b border-[var(--line-hairline)]">
      <div class="mx-auto max-w-6xl px-5 pb-14 pt-12 sm:px-8 md:pb-20 md:pt-16 lg:pb-24 lg:pt-20">
        <div class="grid items-center gap-12 lg:grid-cols-[1.15fr_0.85fr] lg:gap-16">
          <div class="max-w-xl">
            <h1 class="font-display text-4xl font-bold leading-[1.06] tracking-[-0.03em] text-balance text-[var(--ink-primary)] sm:text-5xl lg:text-[3.5rem]">
              Grave a tela.<br>Só isso.
            </h1>

            <p class="mt-5 max-w-[44ch] text-lg leading-relaxed text-pretty text-[var(--ink-secondary)]">
              Gravador de tela para Windows com menos de 2 MB.
            </p>

            <div class="mt-8 flex flex-wrap items-center gap-3.5">
              <a
                :href="downloadUrl"
                rel="noopener"
                class="focus-ring inline-flex items-center gap-2.5 rounded-xl bg-[#14181f] px-6 py-3.5 text-base font-semibold text-[#f6f7f9] shadow-md shadow-black/15 transition-colors hover:bg-[#232a35] dark:bg-[#f2f4f8] dark:text-[#15181e] dark:hover:bg-[#dde2eb]"
              >
                <UIcon name="i-lucide-download" class="size-5" aria-hidden="true" />
                <span>Baixar instalador</span>
              </a>

              <a
                href="#recursos"
                class="focus-ring inline-flex items-center gap-2 rounded-xl border border-[var(--line-hairline)] bg-[var(--surface-elevated)] px-5 py-3.5 text-base font-medium text-[var(--ink-primary)] transition-colors hover:border-[var(--ink-muted)]"
              >
                <span>Ver recursos</span>
              </a>
            </div>

            <div class="mt-8 flex flex-wrap items-center gap-x-5 gap-y-3 border-t border-[var(--line-hairline)] pt-5 text-sm text-[var(--ink-muted)]">
              <span>Windows 10 e 11 · gratuito · licença MIT</span>
              <span class="inline-flex items-center gap-1.5" aria-label="Atalho: Ctrl, Shift e R">
                <kbd class="keycap">Ctrl</kbd>
                <span aria-hidden="true">+</span>
                <kbd class="keycap">Shift</kbd>
                <span aria-hidden="true">+</span>
                <kbd class="keycap">R</kbd>
              </span>
            </div>
          </div>

          <!-- Pôster oficial apenas em telas grandes, evitando espaço vertical desperdiçado no celular. -->
          <div class="ml-auto hidden w-full max-w-[340px] lg:block">
            <div class="poster-elevation overflow-hidden rounded-2xl">
              <ThemeArt
                light="media/poster-light.png"
                dark="media/poster-dark.png"
                :width="720"
                :height="1080"
                priority
                alt="Pôster do kapture com obturador de câmera e ponto vermelho de gravação"
              />
            </div>
          </div>
        </div>

        <!-- Continuação natural do hero: a própria barra real do app, mudando junto com o tema do site. -->
        <div class="mt-12 border-t border-[var(--line-hairline)] pt-10 lg:mt-16 lg:pt-12">
          <AppPreview />
        </div>
      </div>
    </section>

    <!-- Lista enxuta de recursos em linguagem simples. -->
    <section id="recursos" class="mx-auto max-w-5xl px-5 py-16 sm:px-8 lg:py-24">
      <h2 class="font-display text-2xl font-bold tracking-[-0.02em] text-[var(--ink-primary)] sm:text-3xl">
        Recursos
      </h2>

      <ul class="mt-8 divide-y divide-[var(--line-hairline)] border-y border-[var(--line-hairline)]">
        <li
          v-for="item in features"
          :key="item.title"
          class="grid gap-1.5 py-5 sm:grid-cols-[minmax(0,15rem)_1fr] sm:gap-10"
        >
          <h3 class="font-display text-base font-bold text-[var(--ink-primary)]">
            {{ item.title }}
          </h3>
          <p class="text-sm leading-relaxed text-[var(--ink-secondary)] sm:text-base">
            {{ item.detail }}
          </p>
        </li>
      </ul>
    </section>

    <!-- Instalação direta sobre a mesma grade sutil da marca. -->
    <section id="instalar" class="brand-grid border-t border-[var(--line-hairline)]">
      <div class="mx-auto max-w-5xl px-5 py-16 sm:px-8 lg:py-24">
        <h2 class="font-display text-2xl font-bold tracking-[-0.02em] text-[var(--ink-primary)] sm:text-3xl">
          Instalação
        </h2>
        <p class="mt-2 text-base text-[var(--ink-secondary)]">
          Windows 10 ou 11, 64 bits.
        </p>

        <ol class="mt-8 grid gap-6 border-y border-[var(--line-hairline)] py-7 sm:grid-cols-3 sm:gap-8">
          <li v-for="(step, idx) in installSteps" :key="step.title">
            <p class="font-mono text-xs font-semibold tabular-nums text-[var(--ink-muted)]">
              Passo {{ idx + 1 }}
            </p>
            <h3 class="font-display mt-1.5 text-base font-bold text-[var(--ink-primary)]">
              {{ step.title }}
            </h3>
            <p class="mt-1.5 text-sm leading-relaxed text-[var(--ink-secondary)]">
              {{ step.body }}
            </p>
          </li>
        </ol>

        <div class="mt-8">
          <a
            :href="downloadUrl"
            rel="noopener"
            class="focus-ring inline-flex items-center gap-2.5 rounded-xl bg-[#14181f] px-6 py-3.5 text-base font-semibold text-[#f6f7f9] shadow-md shadow-black/15 transition-colors hover:bg-[#232a35] dark:bg-[#f2f4f8] dark:text-[#15181e] dark:hover:bg-[#dde2eb]"
          >
            <UIcon name="i-lucide-download" class="size-5" aria-hidden="true" />
            <span>Baixar instalador</span>
          </a>
        </div>

        <p class="mt-6 max-w-[62ch] text-xs leading-relaxed text-[var(--ink-muted)]">
          <strong class="font-semibold text-[var(--ink-secondary)]">Aviso do SmartScreen:</strong>
          como o instalador ainda não é assinado digitalmente, o Windows pode mostrar “O Windows protegeu o seu computador”. Clique em <em>Mais informações</em> e depois em <em>Executar assim mesmo</em>.
        </p>
      </div>
    </section>
  </div>
</template>
