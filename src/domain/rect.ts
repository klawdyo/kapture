// Cálculos puros de retângulo para o seletor de área.
import type { Dim } from './size'
import type { Region } from './types'

/** Ponto em pixels. */
export interface Pt { x: number; y: number }

/** Normaliza um arrasto (qualquer direção) em retângulo com w/h >= 0. */
export function norm(a: Pt, b: Pt): Region {
  return {
    x: Math.min(a.x, b.x),
    y: Math.min(a.y, b.y),
    w: Math.abs(a.x - b.x),
    h: Math.abs(a.y - b.y),
  }
}

/** Mínimo (px) de largura e altura de uma seleção válida. */
export const MIN = 16

/** Seleção grande o bastante para gravar? */
export function big(r: Region): boolean {
  return r.w >= MIN && r.h >= MIN
}

/** Origem física do monitor na área de trabalho virtual e a escala dele (vindas da URL). */
export interface Org { ox: number; oy: number; s: number }

/** Lê `?ox=&oy=&s=` da janela do seletor; ausente ou inválido vira origem 0,0 e escala 1. */
export function org(search: string): Org {
  const q = new URLSearchParams(search)
  const num = (k: string, d: number): number => {
    const t = q.get(k)
    const v = t === null || t.trim() === '' ? NaN : Number(t)
    return Number.isFinite(v) ? v : d
  }
  const s = num('s', 1)
  return { ox: Math.round(num('ox', 0)), oy: Math.round(num('oy', 0)), s: s > 0 ? s : 1 }
}

/** Converte px CSS em px físicos (escala do monitor + origem do monitor, global e possivelmente negativa). */
export function phys(r: Region, dpr: number, ox: number, oy: number): Region {
  return {
    x: Math.round(r.x * dpr + ox),
    y: Math.round(r.y * dpr + oy),
    w: Math.round(r.w * dpr),
    h: Math.round(r.h * dpr),
  }
}

/** Etiqueta de resolução, ex.: "1280 × 720". */
export function label(r: Region): string {
  return `${r.w} × ${r.h}`
}

/** Alça de redimensionamento: pontos cardeais (n = topo, e = direita...). */
export type Handle = 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w'
/** Alvo de um clique: uma alça, o interior (mover) ou nada (desenhar outro). */
export type Hit = Handle | 'move' | null

/** Todas as alças, na ordem de desenho. */
export const HANDLES: Handle[] = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w']

/** Tolerância (px) do clique nos cantos, nos lados e ao longo do meio do lado. */
const CORNER = 12
const EDGE = 8
const SPAN = 18

const near = (a: number, b: number, t: number): boolean => Math.abs(a - b) <= t
const clamp = (v: number, lo: number, hi: number): number => Math.min(Math.max(v, lo), hi)

/** O que há sob o ponto: alça (cantos primeiro), interior ou nada. */
export function hit(r: Region, p: Pt): Hit {
  const x2 = r.x + r.w
  const y2 = r.y + r.h
  const cx = near(p.x, r.x, CORNER) ? 'w' : near(p.x, x2, CORNER) ? 'e' : ''
  const cy = near(p.y, r.y, CORNER) ? 'n' : near(p.y, y2, CORNER) ? 's' : ''
  if (cx && cy) return (cy + cx) as Handle
  if (near(p.x, r.x + r.w / 2, SPAN)) {
    if (near(p.y, r.y, EDGE)) return 'n'
    if (near(p.y, y2, EDGE)) return 's'
  }
  if (near(p.y, r.y + r.h / 2, SPAN)) {
    if (near(p.x, r.x, EDGE)) return 'w'
    if (near(p.x, x2, EDGE)) return 'e'
  }
  const inside = p.x > r.x && p.x < x2 && p.y > r.y && p.y < y2
  return inside ? 'move' : null
}

/** Cursor CSS para cada alvo. */
export function cursor(h: Hit): string {
  if (h === null) return 'crosshair'
  if (h === 'move') return 'move'
  if (h === 'n' || h === 's') return 'ns-resize'
  if (h === 'e' || h === 'w') return 'ew-resize'
  return h === 'nw' || h === 'se' ? 'nwse-resize' : 'nesw-resize'
}

/** Prende um ponto dentro dos limites da tela. */
export function pin(p: Pt, b: Dim): Pt {
  return { x: clamp(p.x, 0, b.w), y: clamp(p.y, 0, b.h) }
}

/** Move o retângulo (mesmo tamanho) sem sair da tela. */
export function shift(r: Region, dx: number, dy: number, b: Dim): Region {
  return { ...r, x: clamp(r.x + dx, 0, b.w - r.w), y: clamp(r.y + dy, 0, b.h - r.h) }
}

/** Arrasta uma alça por (dx, dy): respeita o mínimo, a tela e não inverte o lado. */
export function resize(r: Region, h: Handle, dx: number, dy: number, b: Dim): Region {
  let l = r.x
  let t = r.y
  let rt = r.x + r.w
  let bt = r.y + r.h
  if (h.includes('w')) l = clamp(l + dx, 0, rt - MIN)
  if (h.includes('e')) rt = clamp(rt + dx, l + MIN, b.w)
  if (h.includes('n')) t = clamp(t + dy, 0, bt - MIN)
  if (h.includes('s')) bt = clamp(bt + dy, t + MIN, b.h)
  return { x: l, y: t, w: rt - l, h: bt - t }
}

/** Sobra espaço abaixo do retângulo para a etiqueta? */
export function room(r: Region, b: Dim): boolean {
  return r.y + r.h + 56 <= b.h
}
