// Regra do quadro da webcam: sempre 1:1.

/** Lado quadrado a aplicar após redimensionar, ou null se já é quadrado. */
export function fit(w: number, h: number): number | null {
  return w === h ? null : Math.max(w, h)
}

/** Tamanho lógico (px CSS) de uma janela. */
export interface Dim { w: number; h: number }

/** Barra completa (igual ao `tauri.conf.json`). */
export const BAR: Dim = { w: 720, h: 136 }
/** Altura da linha extra (alvo e configurações). */
export const EXTRA_H = 48
/** Pílula do modo gravando. */
export const PILL: Dim = { w: 150, h: 48 }

/** Tamanho da janela da barra conforme o modo e a linha extra. */
export function barSize(active: boolean, extra: boolean): Dim {
  if (active) return PILL
  return extra ? { w: BAR.w, h: BAR.h + EXTRA_H } : BAR
}
