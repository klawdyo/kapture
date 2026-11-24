// Resolução do deviceId de câmera a partir do nome amigável do backend.

/** Dispositivo de vídeo como o navegador o descreve. */
export interface VideoDev { deviceId: string; label: string }

/**
 * Acha o deviceId cujo label casa com o nome recebido: igualdade primeiro,
 * depois "contém" (nos dois sentidos), sem diferenciar maiúsculas.
 * Sem nome ou sem casamento devolve undefined (câmera padrão).
 */
export function pickCam(list: VideoDev[], name?: string): string | undefined {
  const n = name?.trim().toLowerCase()
  if (!n) return undefined
  const low = list.map((d) => ({ id: d.deviceId, l: d.label.toLowerCase() }))
  const hit =
    low.find((d) => d.l === n) ?? low.find((d) => d.l !== '' && (d.l.includes(n) || n.includes(d.l)))
  return hit?.id
}
