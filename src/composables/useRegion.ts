// Lógica do seletor de área: arrastar, confirmar (Enter) e cancelar (Esc).
import { computed, ref } from 'vue'
import { msg } from '../domain/err'
import { big, label, norm, phys, type Pt } from '../domain/rect'
import type { Region } from '../domain/types'
import { ports } from '../ports'

export function useRegion() {
  const b = ports().backend
  const a = ref<Pt | null>(null) // ponto inicial do arrasto
  const c = ref<Pt>({ x: 0, y: 0 }) // ponto atual
  const drag = ref(false)
  const err = ref('')

  /** Retângulo em px CSS (zerado sem seleção). */
  const css = computed<Region>(() => (a.value ? norm(a.value, c.value) : { x: 0, y: 0, w: 0, h: 0 }))
  const ok = computed(() => big(css.value))
  /** Retângulo em px físicos, o que vai para o backend. */
  const reg = computed(() => phys(css.value, window.devicePixelRatio, window.screenX, window.screenY))
  const shown = computed(() => drag.value || ok.value)
  const tag = computed(() => label(reg.value))

  function down(x: number, y: number): void {
    a.value = { x, y }
    c.value = { x, y }
    drag.value = true
  }

  function move(x: number, y: number): void {
    if (drag.value) c.value = { x, y }
  }

  function up(): void {
    drag.value = false
    if (!ok.value) a.value = null // seleção minúscula: descarta
  }

  async function submit(r: Region | null): Promise<void> {
    try {
      await b.submitRegion(r)
    } catch (e) {
      err.value = msg(e)
    }
  }

  /** Enter confirma (se válida); Esc cancela. */
  function key(e: KeyboardEvent): void {
    if (e.key === 'Escape') void submit(null)
    else if (e.key === 'Enter' && ok.value) void submit(reg.value)
  }

  const init = (): void => window.addEventListener('keydown', key)
  const dispose = (): void => window.removeEventListener('keydown', key)

  return { css, ok, shown, tag, err, down, move, up, key, init, dispose }
}
