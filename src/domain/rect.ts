// Cálculos puros de retângulo para o seletor de área.
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

/** Mínimo (px) para considerar uma seleção válida. */
export const MIN = 8

/** Seleção grande o bastante para gravar? */
export function big(r: Region): boolean {
  return r.w >= MIN && r.h >= MIN
}

/** Converte px CSS em px físicos (escala do monitor + origem da janela). */
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
