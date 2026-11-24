// Formatação de tempo do cronômetro.

const pad = (n: number): string => String(n).padStart(2, '0')

/** Formata segundos como mm:ss (ou h:mm:ss a partir de 1h). */
export function fmt(secs: number): string {
  const t = Math.max(0, Math.floor(secs))
  const h = Math.floor(t / 3600)
  const m = Math.floor((t % 3600) / 60)
  const s = t % 60
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${pad(m)}:${pad(s)}`
}
