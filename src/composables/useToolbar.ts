// Lógica da janela da barra de ferramentas (dados e ações próprios).
import { computed, ref, watch } from 'vue'
import { buildCfg, targetOpts } from '../domain/cfg'
import { msg } from '../domain/err'
import { idle, isActive, isPaused } from '../domain/rec'
import { barSize } from '../domain/size'
import { fmt } from '../domain/time'
import type { Devices, Mode, RecState, Region, Targets } from '../domain/types'
import { ports } from '../ports'
import type { Off } from '../ports/types'

export function useToolbar() {
  const b = ports().backend
  const mode = ref<Mode>('screen')
  const cam = ref('') // '' = sem câmera
  const mic = ref('') // '' = sem microfone
  const tid = ref('') // alvo (monitor/janela) selecionado
  const sys = ref(true) // áudio do sistema
  const fps = ref('30')
  const devs = ref<Devices>({ cams: [], mics: [] })
  const tg = ref<Targets>({ monitors: [], wins: [] })
  const level = ref(0)
  const rec = ref<RecState>(idle)
  const err = ref('')
  const saved = ref('') // caminho do último MP4
  const cfg = ref(false) // painel de configurações aberto
  let offs: Off[] = []

  const active = computed(() => isActive(rec.value))
  const paused = computed(() => isPaused(rec.value))
  const clock = computed(() => fmt(rec.value.secs))
  const opts = computed(() => targetOpts(mode.value, tg.value))
  // Linha extra: configurações abertas ou escolha de alvo (várias telas/janelas).
  const extra = computed(() => cfg.value || (mode.value !== 'area' && opts.value.length > 1))
  const size = computed(() => barSize(active.value, extra.value))
  // O medidor só mostra nível quando há microfone escolhido.
  const lvl = computed(() => (mic.value ? level.value : 0))

  // A janela acompanha o conteúdo (pílula, barra, barra com linha extra).
  watch(size, (d) => {
    b.fitBar(d.w, d.h).catch((e: unknown) => { err.value = msg(e) })
  })

  /** Executa uma ação capturando o erro para exibição. */
  async function run(fn: () => Promise<void>): Promise<void> {
    err.value = ''
    try {
      await fn()
    } catch (e) {
      err.value = msg(e)
    }
  }

  /** Seleciona o primeiro alvo disponível (ou nenhum). */
  function pick(): void {
    tid.value = opts.value[0]?.value ?? ''
  }

  async function refresh(): Promise<void> {
    rec.value = await b.state()
  }

  async function start(): Promise<void> {
    await run(async () => {
      let region: Region | undefined
      if (mode.value === 'area') {
        const r = await b.pickRegion()
        if (!r) return // cancelado no seletor
        region = r
      }
      saved.value = ''
      await b.start(
        buildCfg({ mode: mode.value, tid: tid.value, mic: mic.value, sys: sys.value, fps: Number(fps.value), region }),
      )
      await refresh()
    })
  }

  async function stop(): Promise<void> {
    await run(async () => {
      saved.value = await b.stop()
      await refresh()
    })
  }

  /** Alterna pausa/retomada. */
  async function pause(): Promise<void> {
    await run(async () => {
      await (paused.value ? b.resume() : b.pause())
      await refresh()
    })
  }

  /** Alterna gravação (botão e atalho global). */
  function toggle(): Promise<void> {
    return active.value ? stop() : start()
  }

  function setMode(m: Mode): void {
    mode.value = m
    pick()
  }

  /** Troca a câmera e mostra/oculta a janela da webcam. */
  function setCam(id: string): Promise<void> {
    cam.value = id
    return run(() => b.toggleCam(id !== '', id || undefined))
  }

  /** Liga o medidor ao escolher/alterar o microfone; '' o desliga. */
  function setMic(id: string): Promise<void> {
    mic.value = id
    return run(() => (id === '' ? b.micStop() : b.micStart(id)))
  }

  /** Abre a pasta de logs no Explorer. */
  const openLogs = (): Promise<void> => run(() => b.openLogs())

  /** Minimiza a janela da barra. */
  const minimize = (): Promise<void> => run(() => b.minimize())
  /** Fecha a barra (encerra o app). */
  const close = (): Promise<void> => run(() => b.close())

  const toggleSys = (): void => { sys.value = !sys.value }
  const toggleCfg = (): void => { cfg.value = !cfg.value }

  async function init(): Promise<void> {
    await run(async () => {
      ;[devs.value, tg.value] = await Promise.all([b.devices(), b.targets()])
      pick()
      await refresh()
    })
    offs = await Promise.all([
      b.onMic((l) => { level.value = l }),
      b.onState((s) => { rec.value = s }),
      b.onHotkey(() => { void toggle() }),
    ])
  }

  function dispose(): void {
    offs.forEach((f) => f())
    offs = []
    void b.micStop().catch(() => {}) // descarta: desliga o medidor
  }

  return {
    mode, cam, mic, tid, sys, fps, devs, level, rec, err, saved, cfg,
    active, paused, clock, opts, lvl, extra, size,
    init, dispose, start, stop, pause, toggle, setMode, setCam, toggleSys, toggleCfg, setMic, openLogs, minimize, close,
  }
}
