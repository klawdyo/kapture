// Regra do quadro da webcam: sempre 1:1.

/** Lado quadrado a aplicar após redimensionar, ou null se já é quadrado. */
export function fit(w: number, h: number): number | null {
  return w === h ? null : Math.max(w, h)
}
