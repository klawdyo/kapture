// Catálogo de ícones (viewBox 24x24). f = preenchido; senão só contorno.
export const ICONS = {
  screen: { f: false, d: 'M3 5h18v11H3zM8 20h8M12 16v4' },
  window: { f: false, d: 'M3 5h18v14H3zM3 9h18' },
  area: { f: false, d: 'M4 8V4h4M16 4h4v4M20 16v4h-4M8 20H4v-4' },
  cam: { f: false, d: 'M3 7h12v10H3zM15 11l6-3v8l-6-3' },
  mic: { f: false, d: 'M12 3a3 3 0 013 3v6a3 3 0 01-6 0V6a3 3 0 013-3zM5 11a7 7 0 0014 0M12 18v3' },
  sys: { f: false, d: 'M4 9h4l5-4v14l-5-4H4zM16 9a4 4 0 010 6' },
  gear: { f: false, d: 'M12 9a3 3 0 100 6 3 3 0 000-6zM12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M19 5l-2 2M7 17l-2 2' },
  rec: { f: true, d: 'M12 6a6 6 0 100 12 6 6 0 000-12z' },
  stop: { f: true, d: 'M7 7h10v10H7z' },
  pause: { f: true, d: 'M8 6h3v12H8zM13 6h3v12h-3z' },
  play: { f: true, d: 'M8 5l11 7-11 7z' },
} as const

/** Nome de ícone válido. */
export type IconName = keyof typeof ICONS

/** Opção de lista com ícone opcional. */
export interface UiOpt { value: string; label: string; icon?: IconName }
