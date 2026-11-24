// Lógica do seletor de área: desenhar, mover, redimensionar (8 alças), teclado, confirmar e cancelar.
import { computed, ref } from 'vue'
import { msg } from '../domain/err'
import {
  big, cursor, hit, label, norm, phys, pin, resize, room, shift, type Handle, type Hit, type Pt,
} from '../domain/rect'
import type { Dim } from '../domain/size'
import type { Region } from '../domain/types'
import { ports } from '../ports'

/** Gesto em andamento: desenhar um novo retângulo, mover ou arrastar uma alça. */
type Mode = 'draw' | 'move' | Handle

/** Setas do teclado: deslocamento unitário. */
const ARROWS: Record<string, Pt> = {
  ArrowLeft: { x: -1, y: 0 }, ArrowRight: { x: 1, y: 0 }, ArrowUp: { x: 0, y: -1 }, ArrowDown: { x: 0, y: 1 },
}

export function useRegion() {
  const b = ports().backend
  const rect = ref<Region | null>(null) // seleção atual em px CSS
  const mode = ref<Mode | null>(null) // gesto em andamento
  const org = ref<Region>({ x: 0, y: 0, w: 0, h: 0 }) // retângulo no início do gesto
  const from = ref<Pt>({ x: 0, y: 0 }) // ponto do mouse no início do gesto
  const over = ref<Hit>(null) // alvo sob o mouse (define o cursor)
  const err = ref('')

  /** Limites da tela (a janela do seletor cobre o monitor inteiro). */
  const lim = (): Dim => ({ w: window.innerWidth, h: window.innerHeight })

  /** Retângulo em px CSS (zerado sem seleção). */
  const css = computed<Region>(() => rect.value ?? { x: 0, y: 0, w: 0, h: 0 })
  const ok = computed(() => big(css.value))
  /** Retângulo em px físicos, o que vai para o backend. */
  const reg = computed(() => phys(css.value, window.devicePixelRatio, window.screenX, window.screenY))
  const drag = computed(() => mode.value !== null)
  const shown = computed(() => drag.value || ok.value)
  const tag = computed(() => label(reg.value))
  /** Cursor atual: o do gesto em andamento ou o do alvo sob o mouse. */
  const cur = computed(() => cursor(mode.value === null ? over.value : mode.value === 'draw' ? null : mode.value))
  /** Alça ativa (destacada) durante o arrasto. */
  const act = computed<Hit>(() => (mode.value === null || mode.value === 'draw' ? null : mode.value))
  /** Etiqueta abaixo do retângulo, ou dentro dele quando não há espaço. */
  const below = computed(() => room(css.value, lim()))

  function down(x: number, y: number): void {
    const p = pin({ x, y }, lim())
    const h = ok.value ? hit(css.value, p) : null
    from.value = p
    org.value = css.value
    mode.value = h ?? 'draw'
    if (h === null) rect.value = { x: p.x, y: p.y, w: 0, h: 0 }
  }

  function move(x: number, y: number): void {
    const p = pin({ x, y }, lim())
    const m = mode.value
    if (m === null) {
      over.value = ok.value ? hit(css.value, p) : null
      return
    }
    const dx = p.x - from.value.x
    const dy = p.y - from.value.y
    if (m === 'draw') rect.value = norm(from.value, p)
    else if (m === 'move') rect.value = shift(org.value, dx, dy, lim())
    else rect.value = resize(org.value, m, dx, dy, lim())
  }

  function up(): void {
    mode.value = null
    if (!ok.value) rect.value = null // seleção minúscula: descarta
  }

  async function submit(r: Region | null): Promise<void> {
    try {
      await b.submitRegion(r)
    } catch (e) {
      err.value = msg(e)
    }
  }

  /** Botão "Gravar nesta área": confirma a seleção válida. */
  function go(): void {
    if (ok.value) void submit(reg.value)
  }

  /** Botão "Cancelar": devolve null, sem erro. */
  function cancel(): void {
    void submit(null)
  }

  /** Setas movem 1 px (Shift: 10 px); Alt + setas redimensionam. */
  function nudge(e: KeyboardEvent): void {
    const a = ARROWS[e.key]
    const n = e.shiftKey ? 10 : 1
    const dx = a.x * n
    const dy = a.y * n
    rect.value = e.altKey ? resize(css.value, 'se', dx, dy, lim()) : shift(css.value, dx, dy, lim())
    e.preventDefault()
  }

  /** Enter confirma (se válida); Esc cancela; setas ajustam a seleção. */
  function key(e: KeyboardEvent): void {
    if (e.key === 'Escape') cancel()
    else if (e.key === 'Enter') go()
    else if (Object.hasOwn(ARROWS, e.key) && ok.value) nudge(e)
  }

  const init = (): void => window.addEventListener('keydown', key)
  const dispose = (): void => window.removeEventListener('keydown', key)

  return { css, ok, shown, tag, err, cur, act, below, down, move, up, go, cancel, key, init, dispose }
}
